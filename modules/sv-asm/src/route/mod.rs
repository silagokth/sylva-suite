use sv_lib::model::{DataBase, RoutingGraph};
use log::{info, error, debug};


fn _port_id_to_node_id(
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    id: String,
) -> Result<String, Box<dyn std::error::Error>> {
    let parts: Vec<&str> = id.split('_').collect();    
    if parts.len() != 2 {
        return Err("ID format is incorrect".into())
    }
    
    let dir = parts[0];
    let i: i32 = parts[1].parse()?;

    match dir {
        "N" | "n" => return Ok(format!("{}_{}_{}", x + i, y + height - 1, 1)),
        "S" | "s" => return Ok(format!("{}_{}_{}", x + i, y - 1, 1)),
        _ => return Err("Invalid direction in port ID".into()),
    }
}


fn _barrier_id_to_node_id(
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    id: String,
) -> Result<String, Box<dyn std::error::Error>> {
    let parts: Vec<&str> = port_id.split('_').collect();    
    if parts.len() != 2 {
        return Err("ID format is incorrect".into())
    }
    
    let dir = parts[0];
    let i: i32 = parts[1].parse()?;

    match dir {
        "N" | "n" => return Ok(format!("{}_{}_{}", x + i - 1, y + height, 0)),
        "S" | "s" => return Ok(format!("{}_{}_{}", x + i - 1, y - 1, 0)),
        _ => return Err("Invalid direction in ID".into()),
    }
}



fn _node_id_to_coordinates(
    node_id: String,
) -> Result<(i32, i32, i32), Box<dyn std::error::Error>> {
    
    let parts: Vec<&str> = node_id.split('_').collect();    
    if parts.len() != 3 {
        return Err("Node ID format is incorrect".into())
    }
    
    let i: i32 = parts[0].parse()?;
    let j: i32 = parts[1].parse()?;
    let k: i32 = parts[2].parse()?;
    Ok((i, j, k))
}


fn _update_weight(
    graph: &mut RoutingGraph,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    input_: Vec<String>, 
) -> Result<(), Box<dyn std::error::Error>> {
 



fn _add_obstacle(
    graph: &mut RoutingGraph,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    excludes: Vec<String>, 
    includes: Vec<String>, 
) -> Result<(), Box<dyn std::error::Error>> {
    
    let mut include_ids = HashSet::new();
    for id in includes {
        let node_id = _barrier_id_to_node_id(x, y, width, height, id)?;
        include_ids.insert(node_id);
    }
    
    let mut exclude_ids = HashSet::new();
    for id in excludes {
        let node_id = _port_id_to_node_id(x, y, width, height, id)?;
        exclude_ids.insert(node_id);
    }

    // delete nodes inside the placement
    // true -> keep
    // false -> delete
    graph.nodes.retain(|node| {
        // exclude the ports
        if exclude_ids.contains(&node.id) {
            return true;
        }
    
        // include spaces between ports for routing
        if include_ids.contains(&node.id) {
            return false;
        }
        
        let (i, j, k) = match _node_id_to_coordinates(node.id.clone()) {
            Ok(coords) => coords,
            Err(_) => return false,
        };
    
        if i < x - 1 || i >= x + width || j < y - 1 || j >= y + height {
            return true;
        } else if i == x - 1 && k != 0 {
            return true;
        } else if j == y - 1 && k != 1 {
            return true;
        }
    
        false
    });
   
    // collect remaining node IDs for referencing
    let nodes: HashSet<String> = graph.nodes
        .iter()
        .map(|n| n.id.clone())
        .collect();

    // delete invalid edges (source/target has no valid node)  
    graph.edges.retain(|edge| {
        // exclude if both source/target are in exclusive list
        // ports are not connected to each other 
        if exclude_ids.contains(&edge.source) && exclude_ids.contains(&edge.target) {
            return false;
        }
    
        nodes.contains(&edge.source) && nodes.contains(&edge.target)
    });
    
    Ok(())
}



fn create_full_graph(
    width: i32, 
    height: i32,
) -> RoutingGraph {
    
    // helper function to create node IDs
    // analogy: z -> (dx, dy) 
    let node_id = |x: i32, y: i32, z: i32| -> String {
        format!("{}_{}_{}", x, y, z)
    };

    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    // add nodes
    for x in 0..width {
        for y in 0..height {
            if x < width - 1 {
                nodes.push(Node { id: node_id(x, y, 0), weight: 0.0 });
            }
            if y < height - 1 {
                nodes.push(Node { id: node_id(x, y, 1), weight: 0.0 }); 
            }
        }
    }

    // add edges: horizontal
    for x in 0..width-2 {
        for y in 0..height {
            edges.push( Edge {
                source: node_id(x, y, 0),
                target: node_id(x + 1, y, 0),
                weight: 1.0,
            });
        }
    }

    // add edges: vertical
    for x in 0..width {
        for y in 0..height-2 {
            edges.push( Edge {
                source: node_id(x, y, 1),
                target: node_id(x, y + 1, 1),
                weight: 1.0,
            });
        }
    }

    for x in 0..width-1 {
        for y in 0..height-1 {
            // add edges: right down diagonal connection
            edges.push( Edge {
                source: node_id(x, y, 1),
                target: node_id(x, y, 0),
                weight: 1.2,
            });
            edges.push( Edge {
                source: node_id(x, y + 1, 0),
                target: node_id(x + 1, y, 1),
                weight: 1.2,
            });
            // add edges: left down diagonal connection
            edges.push( Edge {
                source: node_id(x, y + 1, 0),
                target: node_id(x, y, 1),
                weight: 1.2,
            });
            edges.push( Edge {
                source: node_id(x + 1, y, 1),
                target: node_id(x, y, 0),
                weight: 1.2,
            });
        }
    }

    RoutingGraph {
        nodes: nodes,
        edges: edges,
        channels: Vec::new(),
    }
}



fn create_routing_graph(
    db: &mut DataBase,
) -> Result<RoutingGraph, Box<dyn std::error::Error>> {

    let mut graph = create_full_graph(
        db.synthesized_information.max_width,
        db.synthesized_information.max_height,
    );
    let mut node_maps: HashMap<String, (i32, i32, i32, i32)> = HashMap::new();

    for node in &db.app_graph.nodes {
        let input_space = if node.input_ports.len() > 0 { 1 } else { 0 };
        let output_space = if node.output_ports.len() > 0 { 2 } else { 0 };
        
        // get x, y coordinates of the node placement
        let (x, y) = db.synthesized_information.placements.iter()
            .find(|place| place.app_node_id == node.id)
            .map(|place| (place.x, place.y - input_space))
            .ok_or_else(|| {
                Box::<dyn std::error::Error>::from("Cannot find placement")
            })?;
        
        // get width and height of the alimp
        let (width, height) = db.synthesized_information.alimp_bindings.iter()
            .find(|b| b.app_node_id == node.id)
            .map(|b| (b.alimp_instance.width, b.alimp_instance.height + input_space + output_space))
            .ok_or_else(|| {
                Box::<dyn std::error::Error>::from("Cannot find binding")
            })?;

        // add these in the maps
        node_maps.insert(
            node.id.clone(), 
            (
                x.clone(), 
                y.clone(), 
                width.clone(), 
                height.clone(),
            )
        );
            
        // remove the nodes covered by the placement from the graph
        let mut exclude_lists: Vec<String> = Vec::new();
        // add the nodes for spliting edges, making sure that every route 
        // can cross the first layer on top of the transporter platform.
        let mut include_lists: Vec<String> = Vec::new();

        let output_memories: Vec<Vec<_>> = &db.synthesized_information.memory_synthesis
            .iter()
            .find(|m| m.app_node_id == node.id && m.memory_direction == "out")
            .map(|m| m.memory_structure)
            .collect();

        let mut last_position = 0;
    
        for (i, edge_memory) in output_memories.iter().enumerate() {
            if i > 0 {
                include_lists.push(format!("n_{}", last_position));
            }

            let first_position = edge_memory[0].output_channels
                .iter()
                .copied()
                .min()
                .unwrap_or(0);
            last_position = edge_memory[edge_memory.len() - 1].output_channels
                .iter()
                .copied()
                .max()
                .unwrap_or(0);
            
            exclude_lists.push(format!("n_{}", first_out_position));

            if first_out_position != last_out_position {
                exclude_lists.push(format!("n_{}", last_out_position));
            }
        }   

        let input_memories: Vec<Vec<_>> = &db.synthesized_information.memory_synthesis
            .iter()
            .find(|m| m.app_node_id == node.id && m.memory_direction == "in")
            .map(|m| m.memory_structure)
            .collect();

        for (i, edge_memory) in input_memories.iter().enumerate() {
            if i > 0 {
                include_lists.push(format!("s_{}", last_position));
            }

            let first_position = edge_memory[0].input_channels
                .iter()
                .copied()
                .min()
                .unwrap_or(0);
            
            last_position = edge_memory[edge_memory.len() - 1].input_channels
                .iter()
                .copied()
                .max()
                .unwrap_or(0);
            
            exclude_lists.push(format!("s_{}", first_out_position));

            if first_out_position != last_out_position {
                exclude_lists.push(format!("s_{}", last_out_position));
            }
        }

        _add_obstacle(&mut graph, x, y, width, height, exclude_lists, include_lists)?;
    }

    // add routing channels
    let mut channels: Vec<Channel> = Vec::new(); 

    for edge in &db.app_graph.edges {
        let &(source_x, source_y, source_width, source_height) = node_maps
            .get(&edge.source_node)
            .clone()
            .ok_or(format!("Missing source_node {} in node_maps", edge.source_node))?;
        let &(target_x, target_y, target_width, target_height) = node_maps
            .get(&edge.target_node)
            .ok_or(format!("Missing target_node {} in node_maps", edge.target_node))?;
        
        // get source/target port index
        let source_port_index = db.app_graph.nodes.iter()
            .find(|node| node.id == edge.source_node)
            .and_then(|node| {
                node.output_ports.iter().enumerate()
                    .find(|(_, port)| port.id == edge.source_port)
                    .map(|(i, _)| i)
            })
            .ok_or_else(|| {
                Box::<dyn std::error::Error>::from("Cannot find source port in the edges")
            })?;

        let target_port_index = db.app_graph.nodes.iter()
            .find(|node| node.id == edge.target_node)
            .and_then(|node| {
                node.input_ports.iter().enumerate()
                    .find(|(_, port)| port.id == edge.target_port)
                    .map(|(i, _)| i)
            })
            .ok_or_else(|| {
                Box::<dyn std::error::Error>::from("Cannot find target port in the edges")
            })?;
        
        // get the node coordinates
        let source_port = format!("n_{}", source_port_index);
        let target_port = format!("s_{}", target_port_index);
        let source = _port_id_to_node_id(source_x, source_y, source_width, source_height, source_port)?;
        let target = _port_id_to_node_id(target_x, target_y, target_width, target_height, target_port)?;

        channels.push(
            Channel {
                app_edge_id: edge.id.clone(),
                source: source,
                target: target,
                traffic: edge.token_size as f64,
                path: Vec::new(),
            }
        );
    }

    graph.channels = channels;
    Ok(graph)
}




#[allow(unused_variables)]
pub fn run(
    db: &mut DataBase, 
    dir: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Start: precise routing");
    let module_dir = format!("{}/precise-route", dir);
    match std::fs::create_dir_all(&module_dir) {
        Ok(_) => (),
        Err(e) => {
            error!("Failed to create {} with {}", module_dir, e);
            return Err(Box::new(e));
        },
    };

    // TODO: For optimisation
    // 1. Retain some of the previous routes where their start positions haven't changed
 
    // 2. Re-route the rest of the routing paths given their approximated lengths
    
    

    // 3. Add routing paths for all output channels


    info!("Finish: precise routing");
    Ok(())
}
