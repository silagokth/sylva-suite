use sv_lib::model::{TransporterISA}; 
use std::collections::{HashMap, HashSet, BTreeMap};


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
    
