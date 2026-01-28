use sv_lib::model::{TransporterISA}; 
use std::collections::{HashMap, HashSet, BTreeMap};
use crate::transporter::utils;

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


pub fn graph_colouring(
    ir: &BTreeMap<i32, TransporterISA>,
    nodes: &Vec<u32>, 
    edges: &HashSet<(u32, u32)>, 
    k: u32,
) -> Result<HashMap<u32, u32>, utils::RegAllocError> { // virtual registers to physical registers

    let alpha = 1;
    let beta = 2;
    let gamma = 4;

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
                let score =
                    utils::live_range(&ir, n)? * alpha
                  + degree(&graph, n) * beta
                  - (utils::use_count(&ir, n)? as i32) * gamma;

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

