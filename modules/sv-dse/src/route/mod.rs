use sv_lib::model::{DataBase, RoutingGraph, Node, Edge, Channel, RoutingPath, Coordinate};
use log::{info, warn, error};
use std::collections::{HashMap, HashSet};
use plotters::prelude::*;

mod graph;


fn _port_id_to_node_id(
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    port_id: String,
) -> Result<String, Box<dyn std::error::Error>> {
    let parts: Vec<&str> = port_id.split('_').collect();    
    if parts.len() != 2 {
        return Err("Port ID format is incorrect".into())
    }
    
    let dir = parts[0];
    let i: i32 = parts[1].parse()?;

    match dir {
        "N" | "n" => return Ok(format!("{}_{}_{}", x + i, y + height - 1, 1)),
        "S" | "s" => return Ok(format!("{}_{}_{}", x + i, y - 1, 1)),
        "W" | "w" => return Ok(format!("{}_{}_{}", x - 2, y + i, 0)),
        "E" | "e" => return Ok(format!("{}_{}_{}", x + width - 1, y + i, 0)),
        _ => return Err("Invalid direction in port ID".into()),
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
    excludes: Vec<String>, 
) -> Result<(), Box<dyn std::error::Error>> {
    
    let mut exclude_ids = HashSet::new();
    for port_id in excludes {
        let node_id = _port_id_to_node_id(x, y, width, height, port_id)?;
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


fn find_port_position(
    db: &DataBase,
    node_id: &str,
    port_id: &str,
    dir: &str,
) -> Result<i32, Box<dyn std::error::Error>> {
    // finding start address and token size
    let ports: &Vec<_> = db.app_graph.nodes
        .iter()
        .find(|n| n.id == node_id)
        .map(|n| 
            if dir == "in" {
                &n.input_ports
            } else {
                &n.output_ports
            }
        )
        .ok_or_else(|| {
            Box::from(format!("Cannot find node {}", node_id))
            as Box<dyn std::error::Error>
        })?;

    let mut start_address = 0;
    let mut token_size = -1;
    for port in ports.iter() {
        if port.id == port_id {
            token_size = port.token_size;
            break;
        }
        start_address += port.token_size;
    }

    if token_size < 0 {
        return Err(format!("Cannot find port {} in {}", port_id, node_id).into());
    }

    // finding index
    let patterns: &Vec<_> = db.synthesized_information.alimp_bindings
        .iter()
        .find(|a| a.app_node_id == node_id)
        .map(|a|
            if dir == "in" {
                &a.alimp_instance.input_addr_time_patterns
            } else {
                &a.alimp_instance.output_addr_time_patterns
            }
        ) 
        .ok_or_else(|| {
            Box::from(format!("Cannot find alimp {}", node_id))
            as Box<dyn std::error::Error>
        })?;

    let channels: Vec<_> = patterns
        .iter()
        .filter(|p| p.address >= start_address && p.address < start_address + token_size)
        .map(|p| p.channel)
        .collect();
    
    if channels.is_empty() {
        return Err(format!("Cannot map patterns in alimp {}", node_id).into());
    }
    
    // assume that the input/output goes to/from the most left side
    // of each DRRA cell w.r.t. grid cell
    Ok(channels.iter().min().unwrap_or(&0) * db.technology_constraint.width_ratio)
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
        let input_space = if node.input_ports.len() > 0 { 1 * db.technology_constraint.height_ratio } else { 0 };
        let output_space = if node.output_ports.len() > 0 { 2 * db.technology_constraint.height_ratio } else { 0 };
        
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
            .map(|b| (
                b.alimp_instance.width * db.technology_constraint.width_ratio, 
                b.alimp_instance.height * db.technology_constraint.height_ratio + input_space + output_space
            ))
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
        for input_port in node.input_ports.iter() {
            let position = find_port_position(
                &db,
                &node.id,
                &input_port.id,
                "in",
            )?;
            exclude_lists.push(format!("s_{}", position));
        }
        for output_port in node.output_ports.iter() {
            let position = find_port_position(
                &db,
                &node.id,
                &output_port.id,
                "out",
            )?;
            exclude_lists.push(format!("n_{}", position));
        }
        _add_obstacle(&mut graph, x, y, width, height, exclude_lists)?;
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
        let source_position = find_port_position(
            &db,
            &edge.source_node,
            &edge.source_port,
            "out",
        )?;

        let target_position = find_port_position(
            &db,
            &edge.target_node,
            &edge.target_port,
            "in",
        )?;
        
        // get the node coordinates
        let source_port = format!("n_{}", source_position);
        let target_port = format!("s_{}", target_position);
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




fn plot_available_routing_graph(
    db: &DataBase,
    graph: &RoutingGraph,
    module_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    
    let max_x = db.synthesized_information.max_width.clone();
    let max_y = db.synthesized_information.max_height.clone();
    let step_x_label = (max_x / 20) + 1;
    let step_y_label = (max_y / 20) + 1;
    let resolution = match max_x * max_y {
        a if a > 1_000_000 => 10001,
        a if a > 500_000 => 8000,
        a if a > 200_000 => 6000,
        a if a > 100_000 => 4000,
        a if a > 50_000 => 2500,
        a if a > 10_000 => 2000,
        a if a > 5_000 => 1000,
        _ => 800,
    } as u32;

    if resolution > 10000 {
        warn!("the floorplan is too large to be presented in the graph!");
        return Ok(())
    }
    
    let output_file = format!("{}/available_routing_graph.png", module_dir);
    let root = BitMapBackend::new(&output_file, (resolution, resolution)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Available Routing Graph", ("sans-serif", 30))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(0.0..max_x as f64, 0.0..max_y as f64)?;

    chart.configure_mesh()
        .x_labels(((max_x / step_x_label) + 1) as usize)
        .y_labels(((max_y / step_y_label) + 1) as usize)
        .draw()?;

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


fn route(
    graph: &mut RoutingGraph,
) -> Result<(), Box<dyn std::error::Error>> {

    // helper functions
    fn _remove_nodes(graph: &mut RoutingGraph, nodes: &[String]) -> Result<(), Box<dyn std::error::Error>> {
        for id in nodes {
            // check if node exists
            if !graph.nodes.iter().any(|n| n.id == *id) {
                return Err(format!("Node {} does not exist in the graph", id).into());
            }
            // remove the node in graph
            graph.nodes.retain(|n| n.id != *id);
            // remove all edges connected to the node
            graph.edges.retain(|e| e.source != *id && e.target != *id);
        }
        Ok(())
    }

    fn _share_paths(c1: &Channel, c2: &Channel) -> bool {
        c1.source == c2.source || c1.target == c2.target
    }


    // sort channels by traffic (high to low)
    graph.channels.sort_by(|a, b| b.traffic.partial_cmp(&a.traffic).unwrap_or(std::cmp::Ordering::Equal));
    
    // allocate paths
    for index in 0..graph.channels.len() {
        let mut routing_graph = graph.clone();

        for i in 0..index {
            if !_share_paths(&graph.channels[index], &graph.channels[i]) {
                _remove_nodes(&mut routing_graph, &graph.channels[i].path)?;    
            }
        }

        // find a path 
        let path = graph::a_star(
            &routing_graph, 
            graph.channels[index].source.clone(), 
            graph.channels[index].target.clone()
        )?;

        // update the actual channel's path
        graph.channels[index].path = path; 
    }

    Ok(())
}


fn update_synthesized_info(
    db: &mut DataBase,
    graph: &RoutingGraph,
) -> Result<(), Box<dyn std::error::Error>> {
   
    // helper function
    fn _path_segment_to_coordinate(
        n0: String, 
        n1: String,
    ) -> Result<Coordinate, Box<dyn std::error::Error>> {
        let (x0, y0, d0) = _node_id_to_coordinates(n0.clone())?;
        let (x1, y1, d1) = _node_id_to_coordinates(n1.clone())?;
    
        match (x0, y0, d0, x1, y1, d1) {
            // west to east
            (x0, y0, 0, x1, y1, 0) if x0 == x1 - 1 && y0 == y1 => Ok(Coordinate { x: x1, y: y1 }),
            // east to west
            (x0, y0, 0, x1, y1, 0) if x0 == x1 + 1 && y0 == y1 => Ok(Coordinate { x: x0, y: y0 }),
            // south to north
            (x0, y0, 1, x1, y1, 1) if x0 == x1 && y0 == y1 - 1 => Ok(Coordinate { x: x1, y: y1 }),
            // north to south
            (x0, y0, 1, x1, y1, 1) if x0 == x1 && y0 == y1 + 1 => Ok(Coordinate { x: x0, y: y0 }),
            // east to north
            (x0, y0, 0, x1, y1, 1) if x0 == x1 && y0 == y1 => Ok(Coordinate { x: x1, y: y1 }),
            // north to east
            (x0, y0, 1, x1, y1, 0) if x0 == x1 && y0 == y1 => Ok(Coordinate { x: x0, y: y0 }),
            // west to north
            (x0, y0, 0, x1, y1, 1) if x0 == x1 - 1 && y0 == y1 => Ok(Coordinate { x: x1, y: y1 }),
            // north to west
            (x0, y0, 1, x1, y1, 0) if x0 == x1 + 1 && y0 == y1 => Ok(Coordinate { x: x0, y: y0 }),
            // east to south
            (x0, y0, 0, x1, y1, 1) if x0 == x1 && y0 == y1 + 1 => Ok(Coordinate { x: x0, y: y0 }),
            // south to east
            (x0, y0, 1, x1, y1, 0) if x0 == x1 && y0 == y1 - 1 => Ok(Coordinate { x: x1, y: y1 }),
            // west to south
            (x0, y0, 0, x1, y1, 1) if x0 == x1 - 1 && y0 == y1 + 1 => Ok(Coordinate { x: x0 + 1, y: y0 }),
            // south to west
            (x0, y0, 1, x1, y1, 0) if x0 == x1 + 1 && y0 == y1 - 1 => Ok(Coordinate { x: x0, y: y0 + 1 }),
            _ => Err(format!("Invalid path segment: {} -> {}", n0, n1).into()),
        }
    }

    // update synthesized info
    for channel in graph.channels.iter() {
        // check if edge exists
        if !db.app_graph.edges.iter().any(|edge| edge.id == channel.app_edge_id) {
            return Err(format!("Edge {} does not exist in the graph", channel.app_edge_id).into());
        }

        let mut path: Vec<Coordinate> = Vec::new();
        for i in 0..(channel.path.len() - 1) {
            let coord = _path_segment_to_coordinate(
                channel.path[i].clone(),
                channel.path[i + 1].clone(),
            )?;
            path.push(coord);
        }

        db.synthesized_information.routing_paths.push(
            RoutingPath {
                app_edge_id: channel.app_edge_id.clone(),
                path: path,
                delay: 0,
            }
        );
    }

    Ok(())
}



fn plot_routing_graph(
    db: &DataBase,
    module_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    
    let max_x = db.synthesized_information.max_width.clone();
    let max_y = db.synthesized_information.max_height.clone();
    let step_x_label = (max_x / 20) + 1;
    let step_y_label = (max_y / 20) + 1;
    let resolution = match max_x * max_y {
        a if a > 1_000_000 => 10001,
        a if a > 500_000 => 8000,
        a if a > 200_000 => 6000,
        a if a > 100_000 => 4000,
        a if a > 50_000 => 2500,
        a if a > 10_000 => 2000,
        a if a > 5_000 => 1000,
        _ => 800,
    } as u32;

    if resolution > 10000 {
        warn!("the floorplan is too large to be presented in the graph!");
        return Ok(())
    }
    
    let output_file = format!("{}/routing_graph.png", module_dir);
    let root = BitMapBackend::new(&output_file, (resolution, resolution)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Routing Graph", ("sans-serif", 30))
        .margin(10)
        .set_label_area_size(LabelAreaPosition::Left, 40)
        .set_label_area_size(LabelAreaPosition::Bottom, 40)
        .build_cartesian_2d(-0.5f64..(max_x as f64 - 0.5), -0.5f64..(max_y as f64 - 0.5))?;

    chart.configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .x_labels(((max_x / step_x_label) + 1) as usize)
        .y_labels(((max_y / step_y_label) + 1) as usize)
        .draw()?;
    
        
    // Plot routing paths (blue blocks)
    for routing_path in &db.synthesized_information.routing_paths {
        for coord in &routing_path.path {
            let x = coord.x as f64;
            let y = coord.y as f64;

            let rect = Rectangle::new(
                [(x - 0.5, y - 0.5), (x + 0.5, y + 0.5)],
                BLUE.filled(),
            );

            chart.draw_series(std::iter::once(rect))?;
        }
    }

    // draw grid
    for i in 0..max_x {
        chart.draw_series(LineSeries::new(
            vec![(i as f64 - 0.5, -0.5), (i as f64 - 0.5, max_y as f64 - 0.5)],
            ShapeStyle::from(&RGBColor(200, 200, 200)).stroke_width(2),
        ))?;
    }
    for i in 0..max_y {
        chart.draw_series(LineSeries::new(
            vec![(-0.5, i as f64 - 0.5), (max_x as f64 - 0.5, i as f64 - 0.5)],
            ShapeStyle::from(&RGBColor(200, 200, 200)).stroke_width(2),
        ))?;
    }

    // plot nodes
    for node in &db.app_graph.nodes {
        let (mut x, mut y, mut width, mut height) = (-1, -1, -1, -1);

        for placement in &db.synthesized_information.placements {
            if placement.app_node_id == node.id {
                x = placement.x;
                y = placement.y;
                break;
            }
        }

        for binding in &db.synthesized_information.alimp_bindings {
            if binding.app_node_id == node.id {
                width = binding.alimp_instance.width * db.technology_constraint.width_ratio;
                height = binding.alimp_instance.height * db.technology_constraint.height_ratio; 
                break;
            }
        }

        if x == -1 || y == -1 || width == -1 || height == -1 {
            return Err(format!("Placement or binding missing for node {}", node.id).into());
        }

        let (x_float, y_float) = (x as f64, y as f64);
        let step_x = db.technology_constraint.width_ratio;
        let step_y = db.technology_constraint.height_ratio;
        let (step_x_float, step_y_float) = (step_x as f64, step_y as f64);
        let (width_float, height_float) = (width as f64, height as f64);

        // Orange: Main node
        for i in (x..(x + width)).step_by(step_x as usize) {
            for j in (y..(y + height)).step_by(step_y as usize) {
                let (i_float, j_float) = (i as f64, j as f64);
                chart.draw_series(std::iter::once(Rectangle::new(
                    [(i_float - 0.30, j_float - 0.30), (i_float - 0.70 + step_x_float, j_float - 0.70 + step_y_float)],
                    RGBColor(255, 165, 0).filled(), // Orange
                )))?;
            }
        }

        if node.input_ports.len() > 0 {
            // Purple: Input buffer (south of node)
            for i in (x..(x + width)).step_by(step_x as usize) {
                for j in ((y - step_y)..y).step_by(step_y as usize) {
                    let (i_float, j_float) = (i as f64, j as f64);
                    chart.draw_series(std::iter::once(Rectangle::new(
                        [(i_float - 0.30, j_float - 0.30), (i_float - 0.70 + step_x_float, j_float - 0.70 + step_y_float)],
                        RGBColor(160, 32, 240).filled(), // Purple
                    )))?;
                }
            }
        }
       
        if node.output_ports.len() > 0 {
            // Red: Output buffer (north of node)
            for i in (x..(x + width)).step_by(step_x as usize) {
                for j in ((y + height)..(y + height + step_y)).step_by(step_y as usize) {
                    let (i_float, j_float) = (i as f64, j as f64);
                    chart.draw_series(std::iter::once(Rectangle::new(
                        [(i_float - 0.30, j_float - 0.30), (i_float - 0.70 + step_x_float, j_float - 0.70 + step_y_float)],
                        RED.filled(),
                    )))?;
                }
            }

            // Green: Data transporter (north+1)
            for i in (x..(x + width)).step_by(step_x as usize) {
                for j in ((y + height + step_y)..(y + height + (2 * step_y))).step_by(step_y as usize) {
                    let (i_float, j_float) = (i as f64, j as f64);
                    chart.draw_series(std::iter::once(Rectangle::new(
                        [(i_float - 0.30, j_float - 0.30), (i_float - 0.70 + step_x_float, j_float - 0.70 + step_y_float)],
                        GREEN.filled(),
                    )))?;
                }
            }
        }
    }

    root.present()?;
    info!("Saved routing graph to: {}", output_file);
    Ok(())
}


#[allow(unused_variables)]
pub fn run(
    db: &mut DataBase, 
    dir: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Start: routing");
    let module_dir = format!("{}route", dir);
    match std::fs::create_dir_all(&module_dir) {
        Ok(_) => (),
        Err(e) => {
            error!("Failed to create {} with {}", module_dir, e);
            return Err(Box::new(e));
        },
    };

    info!("Stage 1: creat floor plan");
    let mut graph: RoutingGraph = create_routing_graph(db)?;

    info!("Stage 2: plot the graph");
    plot_available_routing_graph(&db, &graph, &module_dir)?;

    info!("Stage 3: start routing algorithm");
    route(&mut graph)?;

    info!("Stage 4: update and generate the routing result");
    update_synthesized_info(db, &graph)?;
    plot_routing_graph(db, &module_dir)?;

    info!("Finish: routing");
    Ok(())
}
