use crate::model::{DataBase, RoutingGraph, Node, Edge};
use std::collections::{HashMap, HashSet};

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
pub fn dijkstra(
    graph: &RoutingGraph, 
    source_id: String,
    target_id: String,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    
    // check source and target nodes
    let _source = graph.nodes.iter().find(|x| x.id == source_id)
        .ok_or_else(|| format!("Cannot find node {}", source_id))?;
    let _target = graph.nodes.iter().find(|x| x.id == target_id)
        .ok_or_else(|| format!("Cannot find node {}", target_id))?;

     
    let mut working_set: HashSet<String> = graph.nodes.iter()
        .filter(|x| x.id != source_id) // get all nodes except source
        .map(|x| x.id.clone())
        .collect();
    
    let mut visited: HashSet<String> = HashSet::from([source_id.clone()]);

    let mut distances: HashMap<String, i32> = HashMap::new();
    let mut paths: HashMap<String, Vec<String>> = HashMap::new();
    
    // initialise distances and paths
    for u in &working_set {
        let edge = graph.edges.iter()
            .find(|e| (e.source == *u && e.target == source_id) || (e.source == source_id && e.target == *u));
            
        if let Some(e) = edge {
            distances.insert(u.clone(), e.weight as i32); 
            paths.insert(u.clone(), vec![source_id.clone(), u.clone()]);
        } else {
            distances.insert(u.clone(), i32::MAX);
            paths.insert(u.clone(), vec![]);
        } 
    }

    // Iterate through working_set until target is found
    while !visited.contains(&target_id) {
        // find node u in working set with minimum distance
        let u_opt = working_set.iter()
                .min_by_key(|id| distances.get(*id).unwrap_or(&i32::MAX))
                .cloned();

        let u = match u_opt {
            Some(node) => node,
            None => return Err("Cannot reach the target in the graph".into()),
        };
        let dist_u = distances.get(&u).cloned().unwrap_or(i32::MAX); 
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
                let new_cost = dist_u.saturating_add(e.weight as i32); // avoid overflow
                let old_cost = *distances.get(v).unwrap_or(&i32::MAX);

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
        result = 0;
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
 
        let mut expected = vec!["v1", "v2", "v4", "v3", "v5", "v6"];
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

        expected = vec![];
        match dijkstra(&graph, "v1".to_string(), "v7".to_string()) {
            Ok(x) => assert_eq!(x, expected),
            Err(e) => panic!("dijkstra fails 3 with {}", e),
        }

    }
}

