use sv_lib::model::{DataBase, Node, Edge, Channel, 
                    RoutingGraph, RoutingPath, Coordinate};
use log::{info, error, debug};
use std::collections::{HashMap, HashSet};
use plotters::prelude::*;


#[allow(unused_variables)]
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


#[allow(unused_variables)]
fn _barrier_id_to_node_id(
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
        "N" | "n" => return Ok(format!("{}_{}_{}", x + i, y + height, 0)),
        "S" | "s" => return Ok(format!("{}_{}_{}", x + i, y - 1, 0)),
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



fn _add_obstacle(
    graph: &mut RoutingGraph,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    output_positions: HashMap<i32, i32>, 
    input_positions: HashMap<i32, i32>, 
) -> Result<(), Box<dyn std::error::Error>> {
 
    // work out which nodes must be 
    // deleted apart from the alimp block itself
    let mut port_ids = HashSet::new();
    let mut barrier_ids = HashSet::new();
    
    // output 
    for (pos, len) in output_positions.iter() {
        assert!(*pos >= 0);
        assert!(*len >= 1);

        // add first port
        let mut id = format!("n_{}", pos);
        let mut node_id = _port_id_to_node_id(x, y, width, height, id)?;
        
        port_ids.insert(node_id);
       
        // add last port, if any
        if *len > 1 {
            id = format!("n_{}", pos + len - 1);
            node_id = _port_id_to_node_id(x, y, width, height, id)?;
            
            port_ids.insert(node_id);
        }

        // delete every node in between
        for i in 0..len-1 {
            id = format!("n_{}", pos + i);
            node_id = _barrier_id_to_node_id(x, y, width, height, id)?;
            
            barrier_ids.insert(node_id);
        }
    }

    // input 
    for (pos, len) in input_positions.iter() {
        assert!(*pos >= 0);
        assert!(*len >= 1);

        // add first port
        let mut id = format!("s_{}", pos);
        let mut node_id = _port_id_to_node_id(x, y, width, height, id)?;
        
        port_ids.insert(node_id);
        
        // add last port, if any
        if *len > 1 {
            id = format!("s_{}", pos + len - 1);
            node_id = _port_id_to_node_id(x, y, width, height, id)?;
            
            port_ids.insert(node_id);
        }

        // delete every node in between
        for i in 0..len-1 {
            id = format!("s_{}", pos + i);
            node_id = _barrier_id_to_node_id(x, y, width, height, id)?;
            
            barrier_ids.insert(node_id);
        }
    }

    // delete nodes inside the placement
    // true -> keep
    // false -> delete
    graph.nodes.retain(|node| {
        // exclude the ports
        if port_ids.contains(&node.id) {
            return true;
        }
    
        // include spaces between ports for routing
        if barrier_ids.contains(&node.id) {
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
        if port_ids.contains(&edge.source) && port_ids.contains(&edge.target) {
            return false;
        }
    
        nodes.contains(&edge.source) && nodes.contains(&edge.target)
    });

    // update all edges' weight connected to input and output ports
    for edge in graph.edges.iter_mut() {
        if port_ids.contains(&edge.source) || port_ids.contains(&edge.target) {
            edge.weight = 20.0;
        }
    }
    
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
         
        // port information
        let mut output_ports: HashMap<i32, i32> = HashMap::new();
   
        let output_memories: &Vec<Vec<_>> = &db.synthesized_information.memory_synthesis
            .iter()
            .find(|m| m.app_node_id == node.id && m.memory_direction == "out")
            .map(|m| m.memory_structure.clone())
            .into_iter()
            .collect();
   
        for edge_memory in output_memories.iter() {
            let first_position = edge_memory
                .iter()
                .filter_map(|m| m.output_channels.iter().copied().min())
                .min()
                .unwrap_or(0);
            let last_position = edge_memory
                .iter()
                .filter_map(|m| m.output_channels.iter().copied().max())
                .max()
                .unwrap_or(0);
        
            output_ports.insert(
                first_position,
                last_position - first_position + 1, // length
            );
        }   

        let mut input_ports: HashMap<i32, i32> = HashMap::new();
   
        let input_memories: &Vec<Vec<_>> = &db.synthesized_information.memory_synthesis
            .iter()
            .find(|m| m.app_node_id == node.id && m.memory_direction == "in")
            .map(|m| m.memory_structure.clone())
            .into_iter()
            .collect();
    
        for edge_memory in input_memories.iter() {
            let first_position = edge_memory
                .iter()
                .filter_map(|m| m.input_channels.iter().copied().min())
                .min()
                .unwrap_or(0);
            let last_position = edge_memory
                .iter()
                .filter_map(|m| m.input_channels.iter().copied().max())
                .max()
                .unwrap_or(0);
       
            input_ports.insert(
                first_position,
                last_position - first_position + 1, // length
            );
        }  

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

        _add_obstacle(&mut graph, x, y, width, height, output_ports, input_ports)?;
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
        
        // get source/target port positions
        let source_memories: &Vec<_> = db.synthesized_information.memory_synthesis
            .iter()
            .find(|m| 
                m.app_node_id == edge.source_node && 
                m.port_id == edge.source_port && 
                m.memory_direction == "out")
            .map(|m| &m.memory_structure)
            .unwrap();
        
        assert!(source_memories.len() > 0);
        
        let source_first_position = source_memories
            .iter()
            .filter_map(|m| m.output_channels.iter().copied().min())
            .min()
            .unwrap_or(0);
        
        let source_last_position = source_memories
            .iter()
            .filter_map(|m| m.output_channels.iter().copied().max())
            .max()
            .unwrap_or(0);

        
        let target_memories: &Vec<_> = db.synthesized_information.memory_synthesis
            .iter()
            .find(|m| 
                m.app_node_id == edge.target_node && 
                m.port_id == edge.target_port && 
                m.memory_direction == "in")
            .map(|m| &m.memory_structure)
            .unwrap();
 
        assert!(target_memories.len() > 0);

        let target_first_position = target_memories
            .iter()
            .filter_map(|m| m.input_channels.iter().copied().min())
            .min()
            .unwrap_or(0);
        
        let target_last_position = target_memories
            .iter()
            .filter_map(|m| m.input_channels.iter().copied().max())
            .max()
            .unwrap_or(0);
       
        // get the node coordinates
        let mut source = vec![];
        let mut target = vec![];
        
        let mut source_port = format!("n_{}", source_first_position);
        let mut target_port = format!("s_{}", target_first_position);

        source.push(
            _port_id_to_node_id(source_x, source_y, source_width, source_height, source_port)?
        );
        target.push(
            _port_id_to_node_id(target_x, target_y, target_width, target_height, target_port)?
        );

        if source_first_position != source_last_position {
            assert!(source_first_position < source_last_position);
            source_port = format!("n_{}", source_last_position);
            source.push(
                _port_id_to_node_id(source_x, source_y, source_width, source_height, source_port)?
            );
        }

        if target_first_position != target_last_position {
            assert!(target_first_position < target_last_position);
            target_port = format!("s_{}", target_last_position);
            target.push(
                _port_id_to_node_id(target_x, target_y, target_width, target_height, target_port)?
            );
        }

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



fn plot_available_routing_graph(
    db: &DataBase,
    graph: &RoutingGraph,
    module_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let output_file = format!("{}/available_routing_graph.png", module_dir);
    let root = BitMapBackend::new(&output_file, (800, 800)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_x = db.synthesized_information.max_width.clone();
    let max_y = db.synthesized_information.max_height.clone();

    let mut chart = ChartBuilder::on(&root)
        .caption("Available Routing Graph", ("sans-serif", 30))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(0.0..max_x as f64, 0.0..max_y as f64)?;

    chart.configure_mesh().draw()?;

    // Draw grid
    for x in 0..max_x {
        chart.draw_series(LineSeries::new(vec![(x as f64, 0.0), (x as f64, max_y as f64)], &RGBColor(200, 200, 200)))?;
    }
    for y in 0..max_y {
        chart.draw_series(LineSeries::new(vec![(0.0, y as f64), (max_x as f64, y as f64)], &RGBColor(200, 200, 200)))?;
    }

    // helper function to calculate coordinates
    let _node_id_to_xy = |id: &String| -> Result<(f64, f64), Box<dyn std::error::Error>> {
        let (x, y, z) = _node_id_to_coordinates(id.clone())?;
        let (dx, dy) = match z {
            2 => (0.5, 0.5),
            1 => (0.5, 1.0),
            _ => (1.0, 0.5),
        }; 
        Ok(((x as f64) + dx, (y as f64) + dy))
    };

    // Draw nodes
    for node in &graph.nodes {
        let (x, y) = _node_id_to_xy(&node.id)?;
        chart.draw_series(std::iter::once(Circle::new((x as f64, y as f64), 5, RED.filled())))?;
    }

    // Draw edges
    for edge in &graph.edges {
        let (x1, y1) = _node_id_to_xy(&edge.source)?;
        let (x2, y2) = _node_id_to_xy(&edge.target)?;
        let gray = RGBColor(128, 128, 128);
        chart.draw_series(LineSeries::new(vec![(x1 as f64, y1 as f64), (x2 as f64, y2 as f64)], &gray))?;
    }

    root.present()?;
    info!("Saved available routing graph to: {}", output_file);
    Ok(())
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
    let mut graph: RoutingGraph = create_routing_graph(db)?;
    plot_available_routing_graph(&db, &graph, &module_dir)?;
    
    // 3. Add routing paths for all output channels


    info!("Finish: precise routing");
    Ok(())
}
