use sv_lib::model::{RoutingGraph, Node, Edge};
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


// This algorithm applies to an undirected graph
#[allow(dead_code)]
pub fn dijkstra(
    graph: &RoutingGraph, 
    source_id: String,
    target_id: String,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    
    // check source and target nodes
    let _source = graph.nodes.iter().find(|x| x.id == source_id)
        .ok_or_else(|| format!("Source node ({}) does not exist", source_id))?;
    let _target = graph.nodes.iter().find(|x| x.id == target_id)
        .ok_or_else(|| format!("Target node ({}) does not exist", target_id))?;

     
    let mut working_set: HashSet<String> = graph.nodes.iter()
        .filter(|x| x.id != source_id) // get all nodes except source
        .map(|x| x.id.clone())
        .collect();
    
    let mut visited: HashSet<String> = HashSet::from([source_id.clone()]);

    let mut distances: HashMap<String, f64> = HashMap::new();
    let mut paths: HashMap<String, Vec<String>> = HashMap::new();
    
    // initialise distances and paths
    for u in &working_set {
        let edge = graph.edges.iter()
            .find(|e| (e.source == *u && e.target == source_id) || (e.source == source_id && e.target == *u));
            
        if let Some(e) = edge {
            distances.insert(u.clone(), e.weight); 
            paths.insert(u.clone(), vec![source_id.clone(), u.clone()]);
        } else {
            distances.insert(u.clone(), f64::MAX);
            paths.insert(u.clone(), vec![]);
        } 
    }

    // Iterate through working_set until target is found
    while !visited.contains(&target_id) {
        // find node u in working set with minimum distance
        let u_opt = working_set.iter()
                .filter_map(|id| distances.get(id).filter(|&d| *d < f64::MAX).map(|&d| (id, d)))
                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                .map(|(id, _)| id.clone());

        let u = match u_opt {
            Some(node) => node,
            None => return Err("Cannot reach the target in the graph".into()),
        };
        let dist_u = distances.get(&u).cloned().unwrap_or(f64::MAX); 
        let path_u = paths.get(&u).cloned().unwrap_or_else(|| vec![]);

        working_set.remove(&u);
        visited.insert(u.clone());

        // update neighbours
        for v in &working_set {
            let edge_opt = graph.edges.iter()
                .find(|e| (e.source == u && e.target == *v) || (e.source == *v && e.target == u));
            
            if let Some(e) = edge_opt {
                // update distance and path 
                // if coming from u has a lower cost
                let new_cost = dist_u + e.weight; 
                let old_cost = distances.get(v).cloned().unwrap_or(f64::MAX);

                if new_cost < old_cost {
                    distances.insert(v.clone(), new_cost);
                    
                    let mut new_path = path_u.clone();
                    new_path.push(v.clone());
                    paths.insert(v.clone(), new_path);
                }
            }
        }
    }

    paths.remove(&target_id)
        .ok_or_else(|| format!("No path found to target {}", target_id).into())
}



#[allow(dead_code)]
pub fn a_star(
    graph: &RoutingGraph, 
    source_id: String,
    target_id: String,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {

    // Ensure source and target exist
    let _source = graph.nodes.iter().find(|x| x.id == source_id)
        .ok_or_else(|| format!("Source node ({}) does not exist", source_id))?;
    let _target = graph.nodes.iter().find(|x| x.id == target_id)
        .ok_or_else(|| format!("Target node ({}) does not exist", target_id))?;

    // initialise cost maps
    let mut f: HashMap<String, f64> = HashMap::new();
    let mut g: HashMap<String, f64> = HashMap::new();
    let mut h: HashMap<String, f64> = HashMap::new();

    for node in &graph.nodes {
        let id = node.id.clone();
        f.insert(id.clone(), if id == source_id { 0.0 } else { f64::MAX });
        g.insert(id.clone(), 0.0);
        h.insert(id.clone(), manhattan_distance(&node, &_target)? as f64);
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
            let successor_g = g[&q] + weight;
            let successor_h = h[&this];
            let successor_f = successor_g + successor_h;

            // ignore worse paths
            if open_list.contains(&this) && f[&this] <= successor_f {
                continue;
            }

            if closed_list.contains(&this) && f[&this] <= successor_f {
                continue;
            }

            // add to the open list, update scores and predecessors
            open_list.insert(this.clone());
            g.insert(this.clone(), successor_g);
            f.insert(this.clone(), successor_f);
            predecessors.insert(this.clone(), q.clone());
        }
    }

    return Err("Cannot reach the target in the graph".into())
}   




#[allow(dead_code)]
fn test_make_10x10_graph() -> RoutingGraph {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut node_set = HashSet::new();
    
    // Define some obstacles (positions that should not be nodes)
    let obstacles = HashSet::from([
        (0, 2), (1, 0), (1, 3), (2, 1), (3, 2),
        (2, 7), (2, 8), (2, 9),                         
        (3, 3), (3, 4), (3, 5), (4, 5), (5, 5), (6, 5), 
        (4, 7), (4, 8), (5, 7), (5, 8), (6, 8), (6, 9),
        (7, 6), (8, 7), (8, 8)
    ]);
    
    // Helper function to create node IDs
    let node_id = |x: usize, y: usize| -> String {
        format!("{}_{}_n", x, y)
    };

    // Create nodes
    for y in 0..10 {
        for x in 0..10 {
            if obstacles.contains(&(x, y)) {
                continue;
            }
            nodes.push(Node { id: node_id(x, y), weight: 0.0 });
            node_set.insert((x, y));
        }
    }
    
    // Create edges (4-directional)
    for y in 0..10 {
        for x in 0..10 {
            if !node_set.contains(&(x, y)) {
                continue;
            }
            let from = node_id(x, y);
    
            // Try connect to right
            if x + 1 < 10 && node_set.contains(&(x + 1, y)) {
                let to = node_id(x + 1, y);
                edges.push(Edge { source: from.clone(), target: to.clone(), weight: 1.0 });
                edges.push(Edge { source: to.clone(), target: from.clone(), weight: 1.0 }); // undirected
            }
            // Try connect to bottom
            if y + 1 < 10 && node_set.contains(&(x, y + 1)) {
                let to = node_id(x, y + 1);
                edges.push(Edge { source: from.clone(), target: to.clone(), weight: 1.0 });
                edges.push(Edge { source: to.clone(), target: from.clone(), weight: 1.0 }); // undirected
            }
        }
    }
    
    RoutingGraph {
        nodes: nodes,
        edges: edges,
        channels: vec![],
    }
}




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manhattan_distance() {
        let mut n1 = Node{
            id: String::new(), 
            weight: 0.0,
        };
        let mut n2 = Node{
            id: String::new(), 
            weight: 0.0,
        };
        let mut result = 0;

        n1.id = "1_1_hello".to_string();
        n2.id = "1_1_world".to_string();
        match manhattan_distance(&n1, &n2) {
            Ok(x) => assert_eq!(x, result),
            _ => panic!("manhattan_distance fails 1"),
        };

        n1.id = "10_50_hello".to_string();
        n2.id = "1_1_world".to_string();
        result = 9 + 49;
        match manhattan_distance(&n1, &n2) {
            Ok(x) => assert_eq!(x, result),
            _ => panic!("manhattan_distance fails 2"),
        };

        n1.id = "45_300_hello".to_string();
        n2.id = "100_299_world".to_string();
        result = 55 + 1;
        match manhattan_distance(&n1, &n2) {
            Ok(x) => assert_eq!(x, result),
            _ => panic!("manhattan_distance fails 3"),
        };
        
        n1.id = "-45_300_hello".to_string();
        n2.id = "10_299_world".to_string();
        result = 55 + 1;
        match manhattan_distance(&n1, &n2) {
            Ok(x) => assert_eq!(x, result),
            _ => panic!("manhattan_distance fails 4"),
        };

        n1.id = "str_300_hello".to_string();
        n2.id = "10_299_world".to_string();
        match manhattan_distance(&n1, &n2) {
            Ok(_) => panic!("manhattan_distance fails 5"),
            Err(_) => (),
        };

        n1.id = "".to_string();
        n2.id = "10_299_world".to_string();
        match manhattan_distance(&n1, &n2) {
            Ok(_) => panic!("manhattan_distance fails 6"),
            Err(_) => (),
        };
    }

    #[test]
    fn test_dijkstra() {
        let mut graph = RoutingGraph {
            nodes: vec![
                Node {
                    id: "v1".to_string(),
                    weight: 0.0,
                },
                Node {
                    id: "v2".to_string(),
                    weight: 0.0,
                },
                Node {
                    id: "v3".to_string(),
                    weight: 0.0,
                },
                Node {
                    id: "v4".to_string(),
                    weight: 0.0,
                },
                Node {
                    id: "v5".to_string(),
                    weight: 0.0,
                },
                Node {
                    id: "v6".to_string(),
                    weight: 0.0,
                },
            ],
            edges: vec![
                Edge {
                    source: "v1".to_string(),
                    target: "v2".to_string(),
                    weight: 1.0,
                },
                Edge {
                    source: "v1".to_string(),
                    target: "v3".to_string(),
                    weight: 10.0,
                },
                Edge {
                    source: "v1".to_string(),
                    target: "v4".to_string(),
                    weight: 4.0,
                },
                Edge {
                    source: "v2".to_string(),
                    target: "v3".to_string(),
                    weight: 3.0,
                },
                Edge {
                    source: "v2".to_string(),
                    target: "v4".to_string(),
                    weight: 1.0,
                },
                Edge {
                    source: "v3".to_string(),
                    target: "v4".to_string(),
                    weight: 1.0,
                },
                Edge {
                    source: "v3".to_string(),
                    target: "v5".to_string(),
                    weight: 1.0,
                },
                Edge {
                    source: "v4".to_string(),
                    target: "v6".to_string(),
                    weight: 4.0,
                },
                Edge {
                    source: "v5".to_string(),
                    target: "v6".to_string(),
                    weight: 1.0,
                },
            ],
            channels: vec![],
        };
 
        let expected = vec!["v1", "v2", "v4", "v3", "v5", "v6"];
        match dijkstra(&graph, "v1".to_string(), "v6".to_string()) {
            Ok(x) => assert_eq!(x, expected),
            Err(e) => panic!("dijkstra fails 1 with {}", e),
        }

        match dijkstra(&graph, "v1".to_string(), "v7".to_string()) {
            Err(_) => (),
            Ok(_) => panic!("dijkstra fails 2"),
        }
        
        graph.nodes.push(
            Node {
                    id: "v7".to_string(),
                    weight: 0.0,
                }
        );

        match dijkstra(&graph, "v1".to_string(), "v7".to_string()) {
            Err(_) => (),
            Ok(_) => panic!("dijkstra fails 3"),
        }

    }

    
    #[test]
    fn test_a_star() {
        let graph = test_make_10x10_graph();
        let start = "0_0_n".to_string();
        let end = "9_9_n".to_string();
        let path: Vec<String> = vec![
            "0_0_n", "0_1_n", "1_1_n", "1_2_n", "2_2_n", "2_3_n", "2_4_n", "2_5_n",
            "2_6_n", "3_6_n", "4_6_n", "5_6_n", "6_6_n", "6_7_n", "7_7_n", "7_8_n", 
            "7_9_n", "8_9_n", "9_9_n",
        ]
            .into_iter()
            .map(String::from)
            .collect();

        match a_star(&graph, start, end) {
            Ok(x) => assert_eq!(x, path),
            Err(_) => panic!("a_star fails 1"),
        }
    }

}

