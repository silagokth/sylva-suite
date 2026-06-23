use sv_lib::model::{TransporterISA}; 
use std::collections::{HashMap, HashSet, BTreeMap};
use crate::transporter::utils;
use std::cmp::min;

fn _def(inst: &TransporterISA) -> HashSet<u32> {
    let mut set = HashSet::new();

    match inst {
        TransporterISA::LDI { r0, .. } => {
            set.insert(*r0);
        }

        TransporterISA::CAL { r0, .. }
        | TransporterISA::CALI { r0, .. } => {
            set.insert(*r0);
        }

        _ => {}
    }

    // r0 is a special register - not to be included
    set.remove(&0);

    set
}

fn _use(inst: &TransporterISA) -> HashSet<u32> {
    let mut set = HashSet::new();

    match inst {
        TransporterISA::MOV { r0, r1, .. } => {
            set.insert(*r0);
            set.insert(*r1);
        }

        TransporterISA::MOVC { r0, r1, r2, .. } => {
            set.insert(*r0);
            set.insert(*r1);
            set.insert(*r2);
        }

        TransporterISA::CAL { r1, r2, .. }
        | TransporterISA::CALI { r1, r2, .. } => {
            set.insert(*r1);
            set.insert(*r2);
        }

        _ => {}
    }

    // r0 is a special register - not to be included
    set.remove(&0);
    
    set
}


fn _succ(
    ir: &BTreeMap<i32, TransporterISA>,
    index: i32,
) -> Option<i32> {
    ir.range((index + 1)..)
        .next()
        .map(|(&next_index, _)| next_index)
}


pub fn liveness(
    ir: &BTreeMap<i32, TransporterISA>,
) -> (Vec<HashSet<u32>>, Vec<HashSet<u32>>) {
    // linearize instructions
    let vertices: Vec<i32> = ir.keys().cloned().collect();
    let n = vertices.len();

    // map time index -> position
    let mut pos: HashMap<i32, usize> = HashMap::new();
    for (i, &idx) in vertices.iter().enumerate() {
        pos.insert(idx, i);
    }

    // successor list (by position)
    let succ: Vec<Option<usize>> = vertices
        .iter()
        .map(|&idx| _succ(ir, idx).and_then(|s| pos.get(&s).copied()))
        .collect();

    // live-in / live-out sets
    let mut live_in: Vec<HashSet<u32>> = vec![HashSet::new(); n];
    let mut live_out: Vec<HashSet<u32>> = vec![HashSet::new(); n];

    loop {
        let old_in = live_in.clone();
        let old_out = live_out.clone();

        for i in (0..n).rev() {
            // live_out[i] = union of live_in[succ]
            // TODO: merge union if more than one successor
            live_out[i].clear();
            if let Some(s) = succ[i] {
                live_out[i].extend(live_in[s].iter().copied());
            }

            // live_in[i] = use[i] ∪ (live_out[i] − def[i])
            let inst = &ir[&vertices[i]];

            let mut new_in = _use(inst);
            let def = _def(inst);
            
            for r in live_out[i].iter() {
                if !def.contains(r) {
                    new_in.insert(*r);
                }
            }

            live_in[i] = new_in;
        }

        if live_in == old_in && live_out == old_out {
            break;
        }
    }
    
    (live_in, live_out)
}

    


pub fn interference_graph_construction(
    ir: &BTreeMap<i32, TransporterISA>,
    live_out: &[HashSet<u32>],  
) -> (Vec<u32>, HashSet<(u32, u32)>) {
    let nodes: Vec<u32> = ir
        .values() 
        .filter_map(|inst| {
            if let TransporterISA::LDI { r0, .. } = inst {
                Some(*r0)
            } else {
                None
            }
        })
        .collect();

    let mut edges: HashSet<(u32, u32)> = HashSet::new();

    let vertices: Vec<i32> = ir.keys().cloned().collect();

    for (i, idx) in vertices.iter().enumerate() {
        let inst = &ir[idx];
        let defs = _def(inst);
        
        for &a in &defs {
            for &b in &live_out[i] {
                if a != b {
                    let edge = if a < b { (a, b) } else { (b, a) };
                    edges.insert(edge);
                }
            }
        }
    }
    
    (nodes, edges)
}
   


type Node = u32;

#[allow(dead_code)]
struct StackEntry {
    node: Node,
    optimistic: bool, 
}

fn remove_node(
    graph: &mut HashMap<u32, HashSet<u32>>,
    node: u32,
) {
    if let Some(neighs) = graph.remove(&node) {
        for n in neighs {
            if let Some(s) = graph.get_mut(&n) {
                s.remove(&node);
            }
        }
    }
}


fn neighbours(
    node: u32,
    edges: &HashSet<(u32, u32)>,
) -> HashSet<u32> {
    edges
        .iter()
        .filter_map(|&(a, b)| {
            if a == node {
                Some(b)
            } else if b == node {
                Some(a)
            } else {
                None
            }
        })
        .collect()
}



fn spilling_analysis(
    ir: &BTreeMap<i32, TransporterISA>,
    number: u32,
) -> Result<(Vec<i32>, Vec<u32>), utils::RegAllocError> {

    let mov_indices = utils::list_all_mov_indices(ir);

    let use_times = 
        utils::list_mov_indices_to_reg(ir, &mov_indices, number)
        .map_err(|_| utils::RegAllocError::InvalidGraph)?;
    
    if use_times.len() < 2 {
        return Err(utils::RegAllocError::SpillingFail);
    }

    let mut load_times: Vec<i32> = Vec::new();
    let mut scores: Vec<u32> = Vec::new();

    for i in 0..use_times.len() - 1 {
        let interval = use_times[i + 1] - use_times[i];

        let occupied = ir
            .keys()
            .filter(|&&idx| use_times[i] < idx && idx < use_times[i + 1])
            .count() as i32;

        let free = interval - occupied;
        
        if free > 0 {
            let time_slots = utils::find_free_times_before(ir, use_times[i + 1], 1);
            
            let reload_time = time_slots[0];
            load_times.push(reload_time);

            let mov_occupied = ir
                .iter()
                .filter(|(idx, inst)| {
                    let idx = **idx;
                    use_times[i] < idx && idx < reload_time &&
                    matches!(inst,
                        TransporterISA::MOV { .. } |
                        TransporterISA::MOVC { .. } 
                    )
                })
                .count() as u32;
            
            scores.push(mov_occupied);
        } else {
            load_times.push(-1);
            scores.push(0);
        }
    }

    Ok((load_times, scores))
}



pub fn graph_colouring(
    ir: &BTreeMap<i32, TransporterISA>,
    nodes: &Vec<u32>, 
    edges: &HashSet<(u32, u32)>, 
    k: u32,
) -> Result<HashMap<u32, u32>, utils::RegAllocError> { // virtual registers to physical registers

    let alpha = 10;
    let beta = 1;
    let gamma = 2;
    let delta = 4;

    // ---------- Build graph ----------
    let mut graph: HashMap<u32, HashSet<u32>> = HashMap::new();

    for &n in nodes {
        graph.insert(n, HashSet::new());
    }

    for (a, b) in edges {
        if a != b {
            graph.get_mut(&a).ok_or(utils::RegAllocError::InvalidGraph)?.insert(*b);
            graph.get_mut(&b).ok_or(utils::RegAllocError::InvalidGraph)?.insert(*a);
        }
    }

    fn degree(graph: &HashMap<u32, HashSet<u32>>, n: u32) -> i32 {
        graph.get(&n).map(|neigh| neigh.len() as i32).unwrap_or(0)
    }

    // ---------- Simplification ----------
    let mut stack: Vec<StackEntry> = Vec::new();

    while !graph.is_empty() {
        // try to find a node with degree < K
        if let Some((&node, _)) = graph
            .iter()
            .find(|(_, neigh)| neigh.len() < k as usize)
        {
            stack.push(StackEntry {
                node,
                optimistic: false,
            });

            remove_node(&mut graph, node);
        } else {
            // optimistic removal
            // instead of picking the highest degree,
            // we use spill_score to choose the register because 
            // this compiler is time-sensitive

            // Spill heuristic
            let mut best_node = None;
            let mut best_score = i32::MIN;

            for &n in graph.keys() {
                let base_score: i32 = match spilling_analysis(&ir, n) {
                    Ok((_, scores)) => scores.iter().map(|&s| s as i32).sum(),
                    Err(_) => 0,
                };

                
                let score = match base_score {
                    0 => 0,
                    _ => base_score * alpha
                        + utils::live_range(&ir, n)? * beta
                        + degree(&graph, n) * gamma
                        - (utils::use_count(&ir, n)? as i32) * delta,
                };
                                    
                if score > best_score {
                    best_score = score;
                    best_node = Some(n);
                }
            }

            let node = best_node.ok_or(utils::RegAllocError::InvalidGraph)?;
   
            stack.push(StackEntry {
                node,
                optimistic: true,
            });

            remove_node(&mut graph, node);
        }
    }

    // ---------- Assign colours ----------
    let mut colouring: HashMap<u32, u32> = HashMap::new();
    let physical_regs: Vec<u32> = (0..k).collect();

    while let Some(entry) = stack.pop() {
        let node = entry.node;

        // collect used colors of neighbors
        let mut used: HashSet<u32> = HashSet::new();
        for neighbour in neighbours(node, &edges) {
            if let Some(&c) = colouring.get(&neighbour) {
                used.insert(c);
            }
        }

        let available = physical_regs
            .iter()
            .find(|r| !used.contains(r))
            .copied();

        match available {
            Some(colour) => {
                colouring.insert(node, colour);
            }
            None => {
                // optimistic failed → required spilling
                return Err(utils::RegAllocError::SpilledRegister(node));
            }
        }
    }

    Ok(colouring)
}



pub fn transforming_ir(
    ir: &mut BTreeMap<i32, TransporterISA>,
    spilled_register: u32,
) -> Result<(), utils::RegAllocError> {

    let (reload_times, scores) = spilling_analysis(&ir, spilled_register)?;
    
    // no viable spill point
    if scores.iter().all(|&s| s == 0) {
        return Err(utils::RegAllocError::SpillingFail);
    }

    // pick best reload point 
    let (best_idx, _) = scores
        .iter()
        .enumerate()
        .max_by_key(|(_, s)| *s)
        .ok_or(utils::RegAllocError::SpillingFail)?;
    
    let reload_time = reload_times[best_idx];
    
    // MOVs that use this register
    let all_mov_indices = utils::list_all_mov_indices(ir);
    let mut mov_indices = utils::list_mov_indices_to_reg(ir, &all_mov_indices, spilled_register)
        .map_err(|_| utils::RegAllocError::InvalidGraph)?;

    // only those after reload
    mov_indices.retain(|idx| *idx > reload_time);
    
    if mov_indices.is_empty() {
        return Err(utils::RegAllocError::SpillingFail);
    }

    let new_register_number = utils::find_unused_register(ir);
    
    let (_, value) = utils::find_load_register_with_number(ir, spilled_register)
        .ok_or(utils::RegAllocError::InvalidGraph)?;
    
    // insert reload
    ir.insert(
        reload_time,
        TransporterISA::LDI { 
            r0: new_register_number, 
            immediate: value,
        },
    );

    // rewrite uses after reload
    for (&idx, inst) in ir.iter_mut() {
        if idx > reload_time {
            match inst {
                TransporterISA::MOV { r0, r1, .. } => {
                    if *r0 == spilled_register {
                        *r0 = new_register_number;
                    }
                    if *r1 == spilled_register {
                        *r1 = new_register_number;
                    }
                }
                TransporterISA::MOVC { r0, r1, r2, .. } => {
                    if *r0 == spilled_register {
                        *r0 = new_register_number;
                    }
                    if *r1 == spilled_register {
                        *r1 = new_register_number;
                    }
                    if *r2 == spilled_register {
                        *r2 = new_register_number;
                    }
                }
                TransporterISA::LDI { r0, .. } => {
                    if *r0 == spilled_register {
                        return Err(utils::RegAllocError::InvalidGraph);
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}



pub fn retiming_ir(
    ir: &mut BTreeMap<i32, TransporterISA>,
) -> Result<(), utils::RegAllocError> {

    // move all LDIs to before zero time 
    let ldis: Vec<(i32, TransporterISA)> = ir.iter()
        .filter(|(_t, inst)| {
            matches!(
                inst,
                TransporterISA::LDI { .. }
            )
        })
        .map(|(t, inst)| (*t, inst.clone()))
        .collect();

    if ldis.is_empty() {
        return Err(utils::RegAllocError::InvalidGraph);
    }

    for (t, _) in &ldis {
        ir.remove(t);
    }

    let earliest_time = ir.keys().min().copied().unwrap_or(0);
    let before_time = min(0, earliest_time);
    let time_slots = utils::find_free_times_before(ir, before_time, ldis.len());
    
    for ((_, inst), new_t) in ldis.into_iter().zip(time_slots.into_iter()) {
        ir.insert(new_t, inst);
    }
 
    utils::tighten_ir_timing(ir)
        .map_err(|_| utils::RegAllocError::InvalidGraph)?;
    
    Ok(())
}

