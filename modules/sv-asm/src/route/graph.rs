use sv_lib::model::{RoutingGraph, Node};
use std::collections::{HashMap, HashSet};


#[allow(dead_code)]
pub fn manhattan_distance(
    n1: &Node,
    n2: &Node,
) -> Result<i32, Box<dyn std::error::Error>> {
    let pos1: Vec<&str> = n1.id.split('_').collect();
    let pos2: Vec<&str> = n2.id.split('_').collect();

    if pos1.len() < 2 || pos2.len() < 2 {
        return Err("Invalid node ID format".into());
    }

    let x1: i32 = pos1[0].parse()?;
    let y1: i32 = pos1[1].parse()?;
    let x2: i32 = pos2[0].parse()?;
    let y2: i32 = pos2[1].parse()?;

    Ok((x1 - x2).abs() + (y1 - y2).abs())
}


#[allow(dead_code)]
pub fn modified_a_star(
    graph: &RoutingGraph, 
    source_id: String,
    target_id: String,
    lambda: f64,
    target_length: i32,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {

    // Ensure source and target exist
    let _source = graph.nodes.iter().find(|x| x.id == source_id)
        .ok_or_else(|| format!("Source node ({}) does not exist", source_id))?;
    let _target = graph.nodes.iter().find(|x| x.id == target_id)
        .ok_or_else(|| format!("Target node ({}) does not exist", target_id))?;

    // initialise cost maps
    let mut f: HashMap<String, f64> = HashMap::new();
    let mut g_w: HashMap<String, f64> = HashMap::new();
    let mut g_h: HashMap<String, f64> = HashMap::new(); 
    let mut h_w: HashMap<String, f64> = HashMap::new();
    let mut h_h: HashMap<String, f64> = HashMap::new();
    let avg_weight = graph.edges.iter()
        .map(|e| e.weight).sum::<f64>() 
        / graph.edges.len() as f64;

    for node in &graph.nodes {
        let id = node.id.clone();
        f.insert(id.clone(), if id == source_id { 0.0 } else { f64::MAX }); 
        g_w.insert(id.clone(), if id == source_id { 0.0 } else { f64::MAX });
        g_h.insert(id.clone(), if id == source_id { 0.0 } else { f64::MAX });
        h_w.insert(id.clone(), manhattan_distance(&node, &_target)? as f64 * avg_weight);
        h_h.insert(id.clone(), manhattan_distance(&node, &_target)? as f64);
    }
    
    let mut open_list: HashSet<String> = HashSet::from([source_id.clone()]);
    let mut closed_list: HashSet<String> = HashSet::new();
    let mut predecessors: HashMap<String, String> = HashMap::new();

    while !open_list.is_empty() {
        // find node with lowest f 
        let q = open_list.iter()
            .filter_map(|id| f.get(id).map(|&d| (id, d)))
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(id, _)| id.clone())
            .unwrap();

        open_list.remove(&q);
        closed_list.insert(q.clone());
        
        // get all neighbours to q 
        let neighbours: Vec<(String, f64)> = graph.edges.iter()
            .filter(|e| e.source == q || e.target == q)
            .map(|e| { 
                if e.source == q {
                    (e.target.clone(), e.weight.clone())
                } else {
                    (e.source.clone(), e.weight.clone())
                }
            })
            .collect();
    
        // iterate through the neighbours
        for (this, weight) in neighbours { 
            // If we reach the goal, backtrace the path 
            if this == target_id {
                let mut path = vec![this.clone()];
                let mut current = q.clone();
                path.push(current.clone());
                while let Some(prev) = predecessors.get(&current) {
                    current = prev.clone();
                    path.push(current.clone());
                }
                path.reverse();
                return Ok(path); 
            }

            // compute scores
            let successor_h_h = h_h[&this];
            let successor_g_h = g_h[&q] + 1.0; // count hop
            let successor_penalty = ((successor_g_h + successor_h_h) - target_length as f64).abs();
            let successor_h_w = h_w[&this];
            let successor_g_w = g_w[&q] + weight;
            let successor_f = successor_g_w + successor_h_w + lambda * successor_penalty;

            // ignore worse paths
            if open_list.contains(&this) && f[&this] <= successor_f {
                continue;
            }

            if closed_list.contains(&this) && f[&this] <= successor_f {
                continue;
            }

            // add to the open list, update scores and predecessors
            open_list.insert(this.clone());
            g_w.insert(this.clone(), successor_g_w);
            g_h.insert(this.clone(), successor_g_h);
            f.insert(this.clone(), successor_f);
            predecessors.insert(this.clone(), q.clone());
        }
    }

    return Ok(vec![]);
}   


