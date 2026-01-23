use std::collections::{HashMap, HashSet};

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
    nodes: &Vec<u32>, 
    edges: &HashSet<(u32, u32)>, 
    k: u32,
) -> Option<HashMap<u32, u32>> { // virtual registers to physical registers

    // ---------- Build graph ----------
    let mut graph: HashMap<u32, HashSet<u32>> = HashMap::new();

    for &n in nodes {
        graph.insert(n, HashSet::new());
    }

    for (a, b) in edges {
        if a != b {
            graph.get_mut(&a)?.insert(*b);
            graph.get_mut(&b)?.insert(*a);
        }
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
            // optimistic removal: pick highest-degree node
            let (&node, _) = graph
                .iter()
                .max_by_key(|(_, neigh)| neigh.len())
                .unwrap();

            stack.push(StackEntry {
                node,
                optimistic: true,
            });

            remove_node(&mut graph, node);
        }
    }

    // ---------- Assign colors ----------
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
                // optimistic failed → spill required
                return None;
            }
        }
    }

    Some(colouring)
}

