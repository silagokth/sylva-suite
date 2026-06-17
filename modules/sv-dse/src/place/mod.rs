use sv_lib::model::{DataBase, FloorPlan, RectangleShape, RectanglePosition, Placement};
use sv_lib::solver::{Solver};
use log::{info, warn, error, debug};
use serde_json;
use std::collections::HashSet;
use plotters::style::{Color, BLACK, FontStyle};
use plotters::prelude::*;
use rand::Rng;


fn verify_dimension(
    db: &mut DataBase,
) -> Result<(), Box<dyn std::error::Error>> {
    if db.technology_constraint.width_grid <= 0.0 {
        return Err(format!("Technology library provides invalid dimensions").into());
    }

    if db.technology_constraint.height_grid <= 0.0 {
        return Err(format!("Technology library provides invalid dimensions").into());
    }

    if db.technology_constraint.width_drra <= 0.0 {
        return Err(format!("Technology library provides invalid dimensions").into());
    }
    
    if db.technology_constraint.height_drra <= 0.0 {
        return Err(format!("Technology library provides invalid dimensions").into());
    }

    if db.technology_constraint.width_drra <= db.technology_constraint.width_grid {
        return Err(format!("Technology library provides invalid dimensions").into());
    }
    
    if db.technology_constraint.height_drra <= db.technology_constraint.height_grid {
        return Err(format!("Technology library provides invalid dimensions").into());
    }


    // helper function to check if a is a multiple of b 
    fn is_multiple(a: f64, b: f64, epsilon: f64) -> bool {
        if b == 0.0 {
           return false; 
        }

        let quotient = a / b;
        let nearest_int = quotient.round();
        (quotient - nearest_int).abs() < epsilon
    }


    if !is_multiple(db.technology_constraint.width_drra, db.technology_constraint.width_grid, 1e-9) {
        return Err(format!("DRRA cell width is not a multiple of the grid width").into());
    }

    if !is_multiple(db.technology_constraint.height_drra, db.technology_constraint.height_grid, 1e-9) {
        return Err(format!("DRRA cell height is not a multiple of the grid height").into());
    }
    
    db.technology_constraint.grid_per_drra_width = (
        db.technology_constraint.width_drra / db.technology_constraint.width_grid
    ).round() as i32;

    db.technology_constraint.grid_per_drra_height = (
        db.technology_constraint.height_drra / db.technology_constraint.height_grid
    ).round() as i32;

    if db.technology_constraint.grid_per_drra_width > 20 {
        return Err(format!("width ratio exceeds limit").into());
    }

    if db.technology_constraint.grid_per_drra_height > 20 {
        return Err(format!("height ratio exceeds limit").into());
    }

    Ok(())
}


// return width, height
fn dimension(
   db: &DataBase,
   node_id: &str,
) -> Result<(i32, i32, i32, i32, i32, i32, i32, i32), Box<dyn std::error::Error>> {
    let instance = db.synthesized_information.alimp_bindings
        .iter()
        .find(|b| b.app_node_id == node_id)
        .map(|b| &b.alimp_instance)
        .ok_or_else(|| {
            Box::from(format!("Cannot find the alimp instance {}", node_id))
            as Box<dyn std::error::Error>
        })?;
   
    let number_inputs = instance.input_addr_time_patterns
        .iter()
        .map(|p| p.channel)
        .collect::<HashSet<_>>()
        .len() as i32;

    let number_outputs = instance.output_addr_time_patterns
        .iter()
        .map(|p| p.channel)
        .collect::<HashSet<_>>()
        .len() as i32;

    // verify input and output ports
    if instance.width < number_inputs || instance.width < number_outputs {
        return Err(format!("Alimp instance {} cannot fit {} inputs and {} outputs", node_id, number_inputs, number_outputs).into());
    }

    // MANDATORY 
    // The height is inflated: 1 for input buffer, 
    // 1 for output buffer, 
    // 1 for the transporters attached to output buffer, 
    let routing_reserved_size = &db.hyper_parameter.place_reserved_routing_size;
    let all_ports = number_inputs + number_outputs;

    // This needs refinement to add some adjustable parameters for feedback optimization
    Ok((
        instance.width * db.technology_constraint.grid_per_drra_width,
        instance.height * db.technology_constraint.grid_per_drra_height,
        // -------- input/output buffers and transporters use DRRA cell sizes ----------
        if number_outputs != 0 { 2 * db.technology_constraint.grid_per_drra_height } else { 0 }, // transporter and output buffer
        if number_inputs != 0 { 1 * db.technology_constraint.grid_per_drra_height } else { 0 },  // input buffer 
        // ----------------------------------------------------------------------------
        ((all_ports / 10) + 1) * routing_reserved_size, // left space   
        ((all_ports / 5) + 1) * routing_reserved_size, // right space
        ((number_outputs / 3) + 2) * routing_reserved_size, // top space
        ((number_inputs / 3) + 1) * routing_reserved_size  // bottom space
    ))
}



fn create_floor_plan(
    db: &mut DataBase,
) -> Result<FloorPlan, Box<dyn std::error::Error>> {

    let mut fp = FloorPlan {
        app_node_ids: vec![],
        app_edge_ids: vec![],
        max_width: db.global_constraint.max_width.clone(),
        max_height: db.global_constraint.max_height.clone(),
        pos: vec![],
        shape: vec![],
        source_node: vec![],
        target_node: vec![],
        source_port: vec![],
        target_port: vec![],
        top_space: vec![],
        bottom_space: vec![],
        left_space: vec![],
        right_space: vec![],
        conn: vec![],
    };

    // add node information from alimp bindings
    for node in &db.app_graph.nodes {
        let (width, height, ob, ib, left, right, top, bottom) = dimension(db, &node.id)?;
        
        let shape = RectangleShape {
            width: width + left + right,
            height: height + ob + ib + top + bottom,
        };
        
        // dummy positions
        let position = RectanglePosition{
            x: -1, 
            y: -1,
        };

        fp.shape.push(shape);
        fp.top_space.push(top as u32);
        fp.bottom_space.push(bottom as u32);
        fp.left_space.push(left as u32);
        fp.right_space.push(right as u32);
        fp.pos.push(position);
        fp.app_node_ids.push(node.id.clone());
    }

    // add edge information 
    for edge in &db.app_graph.edges {
        let (source_node_index, source_node) = db.app_graph.nodes
                                                .iter()
                                                .enumerate()
                                                .find(|&(_, ref entry)| entry.id == edge.source_node)
                                                .map(|(i, entry)| (i as u32, entry)) 
                                                .ok_or_else(|| {
                                                    Box::from(format!("Cannot find the source node {}", edge.source_node))
                                                    as Box<dyn std::error::Error>
                                                })?;

        let (target_node_index, target_node) = db.app_graph.nodes
                                                .iter()
                                                .enumerate()
                                                .find(|&(_, ref entry)| entry.id == edge.target_node)
                                                .map(|(i, entry)| (i as u32, entry)) 
                                                .ok_or_else(|| {
                                                    Box::from(format!("Cannot find the target node {}", edge.target_node))
                                                    as Box<dyn std::error::Error>
                                                })?;

        // finding index position of source port  
        let mut source_start_address = 0;
        let mut source_token_size = -1;
        for output_port in source_node.output_ports.iter() {
            if output_port.id == edge.source_port {
                source_token_size = output_port.token_size;
                break;
            }
            source_start_address += output_port.token_size;
        }

        if source_token_size < 0 {
            return Err(format!("Cannot find the source port {} in {}", edge.source_port, edge.source_node).into());
        }

        let source_port_index: Option<i32> = db.synthesized_information.alimp_bindings
            .iter()
            .find(|a| a.app_node_id == edge.source_node)
            .and_then(|a| {
                let mut channels = a.alimp_instance.output_addr_time_patterns
                    .iter()
                    .filter(|p| p.address >= source_start_address && p.address < source_start_address + source_token_size)
                    .map(|p| p.channel);
                let first = channels.next()?;
                let (min, max) = channels.fold((first, first), |(min, max), c| (min.min(c), max.max(c)));
                Some((min + max) / 2)
            });
        if source_port_index.is_none() {
            return Err(format!("Cannot find the source port {} in {}", edge.source_port, edge.source_node).into());
        }

        let source_port_position = source_port_index.unwrap_or(0) * db.technology_constraint.grid_per_drra_width + 
            ((db.technology_constraint.grid_per_drra_width - 1) / 2);
        
        // finding index position of target port 
        let mut target_start_address = 0;
        let mut target_token_size = -1;
        for input_port in target_node.input_ports.iter() {
            if input_port.id == edge.target_port {
                target_token_size = input_port.token_size;
                break;
            }
            target_start_address += input_port.token_size;
        }

        if target_token_size < 0 {
            return Err(format!("Cannot find the target port {} in {}", edge.target_port, edge.target_node).into());
        }

        let target_port_index: Option<i32> = db.synthesized_information.alimp_bindings
            .iter()
            .find(|a| a.app_node_id == edge.target_node)
            .and_then(|a| {
                let mut channels = a.alimp_instance.input_addr_time_patterns
                    .iter()
                    .filter(|p| p.address >= target_start_address && p.address < target_start_address + target_token_size)
                    .map(|p| p.channel);
                let first = channels.next()?;
                let (min, max) = channels.fold((first, first), |(min, max), c| (min.min(c), max.max(c)));
                Some((min + max) / 2)
            });
        if target_port_index.is_none() {
            return Err(format!("Cannot find the target port {} in {}", edge.target_port, edge.target_node).into());
        }

        let target_port_position = target_port_index.unwrap_or(0) * db.technology_constraint.grid_per_drra_width + 
            ((db.technology_constraint.grid_per_drra_width - 1) / 2);


        fp.source_node.push(source_node_index);
        fp.target_node.push(target_node_index);
        fp.source_port.push(source_port_position as u32);
        fp.target_port.push(target_port_position as u32);
        fp.conn.push(edge.token_size.clone() as u32);
        fp.app_edge_ids.push(edge.id.clone());
    }

    Ok(fp)
}



/* 
 * Solve the placement problem optimally. 
 * It's a 2D bin-packing problem. 
 * */
fn place_solve_optimal(
    fp: &mut FloorPlan, 
    module_dir: String,
) -> Result<(), Box<dyn std::error::Error>> {

    let mut solver = Solver::new(String::from("place_optimal"), module_dir);

    /* formulating the model */
    solver.add(format!("% minimising the total area to get estimated max width and height")); 
    solver.add(format!("int: N = {};", fp.shape.len()));
    solver.add(format!("int: MAX_WIDTH = {};", fp.max_width)); 
    solver.add(format!("int: MAX_HEIGHT = {};", fp.max_height)); 
    solver.add(format!("array [1..N] of int: widths = {:?};", fp.shape.iter().map(|s| s.width).collect::<Vec<_>>()));
    solver.add(format!("array [1..N] of int: heights = {:?};", fp.shape.iter().map(|s| s.height).collect::<Vec<_>>()));
    solver.new_line();

    solver.add(format!("% x and y coordinates"));
    solver.add(format!("array [1..N] of var 0..MAX_WIDTH: x;"));
    solver.add(format!("array [1..N] of var 0..MAX_HEIGHT: y;"));
    solver.add(format!("var 0..MAX_WIDTH: max_x_position;"));
    solver.add(format!("var 0..MAX_HEIGHT: max_y_position;"));
    solver.new_line();

    solver.add(format!("% ========= constraints ========="));
    solver.add(format!("% each node must be fully inside the working area"));
    solver.add(format!(
        "constraint forall(i in 1..N)(\
        \n  x[i] + widths[i] <= MAX_WIDTH /\\\
        \n  y[i] + heights[i] <= MAX_HEIGHT\
    \n);"));
    solver.new_line();

    solver.add(format!("% No overlap between any two nodes"));
    solver.add(format!(
        "constraint forall(i, j in 1..N where i < j)(\
        \n  (x[i] + widths[i] <= x[j] \\/ x[j] + widths[j] <= x[i]) \\/\
        \n  (y[i] + heights[i] <= y[j] \\/ y[j] + heights[j] <= y[i])\
    \n);"));
    solver.new_line();

    solver.add(format!("% binding the maximum positions"));
    solver.add(format!(
        "constraint forall(i in 1..N)(\
        \n  x[i] + widths[i] <= max_x_position /\\\
        \n  y[i] + heights[i] <= max_y_position\
    \n);"));
    solver.new_line();

    solver.add(format!("% ========= additional constraints ========="));
    solver.add(format!("% first node should be in quadrant one (bottom-left)"));
    solver.add(format!("constraint x[1] < ((max_x_position + 1) div 2);")); 
    solver.add(format!("constraint y[1] < ((max_y_position + 1) div 2);"));
    solver.new_line();

    solver.add(format!("% add a restriction of the ration between max_x_position and max_y_position"));
    solver.add(format!("% the ratio should be greater than 0.5 and less than 2"));
    solver.add(format!("constraint max_x_position * 2 >= max_y_position;")); 
    solver.add(format!("constraint max_x_position <= max_y_position * 2;")); 
    solver.new_line();
    solver.new_line();

    solver.add(format!("% ========= objective ========="));
    solver.add(format!("solve minimize (max_x_position * max_y_position);")); 
    solver.new_line();
    solver.new_line();
    
    solver.add(format!("% ========= output ========="));
    solver.add(String::from("output [\"{\
        x:\\(x),\
        y:\\(y),\
        max_x_position:\\(max_x_position),\
        max_y_position:\\(max_y_position)\
    }\"];")); 

    /* solving the model */
    let (status, solutions) = solver.solve("cp-sat", 180, "")?;
    match status.as_str() {
        "OPTIMAL_SOLUTION" => {}
        _ => return Err(format!("MiniZinc status: {}", status).into()),
    };
    debug!("Minizinc solutions: \n{:?}", solutions);
    let parsed_json_value: serde_json::Value = serde_json::from_str(&solutions[0])?;

    // get the maximum positions from the minizinc result
    let max_x_position: i32 = parsed_json_value
        .get("max_x_position")
        .and_then(|v| v.as_i64())
        .map(|v| v as i32)
        .ok_or_else(|| {
            Box::<dyn std::error::Error>::from("max x position not found")
        })?;

    let max_y_position: i32 = parsed_json_value
        .get("max_y_position")
        .and_then(|v| v.as_i64()) 
        .map(|v| v as i32)
        .ok_or_else(|| {
            Box::<dyn std::error::Error>::from("max y position not found")
        })?;
       
    // update floor plan 
    fp.max_width = max_x_position;
    fp.max_height = max_y_position;

    Ok(())
}



/*
 * solve a similar model to that of the optimal placement,
 * but here, we use the max_width and max_height obtained from the optimal solution.
 * Also, we add additional constraints for Manhattan distance of source nodes to target nodes 
 * */
fn place_solve_approx_optimal(
    fp: &mut FloorPlan,
    module_dir: String,
) -> Result<(), Box<dyn std::error::Error>> {
 
    let mut solver = Solver::new(String::from("approx_place_optimal"), module_dir);

    /* formulating the model */
    solver.add(format!("% minimising the total area to get estimated max width and height")); 
    solver.add(format!("int: N = {};", fp.shape.len()));
    solver.add(format!("int: MAX_WIDTH = {};", fp.max_width)); 
    solver.add(format!("int: MAX_HEIGHT = {};", fp.max_height)); 
    solver.add(format!("array [1..N] of int: widths = {:?};", fp.shape.iter().map(|s| s.width).collect::<Vec<_>>()));
    solver.add(format!("array [1..N] of int: heights = {:?};", fp.shape.iter().map(|s| s.height).collect::<Vec<_>>()));
    solver.new_line();

    solver.add(format!("% x and y coordinates"));
    solver.add(format!("array [1..N] of var 0..MAX_WIDTH: x;"));
    solver.add(format!("array [1..N] of var 0..MAX_HEIGHT: y;"));
    solver.add(format!("var 0..MAX_WIDTH: max_x_position;"));
    solver.add(format!("var 0..MAX_HEIGHT: max_y_position;"));
    solver.new_line();

    solver.add(format!("% ========= constraints ========="));
    solver.add(format!("% each node must be fully inside the working area"));
    solver.add(format!(
        "constraint forall(i in 1..N)(\
        \n  x[i] + widths[i] <= MAX_WIDTH /\\\
        \n  y[i] + heights[i] <= MAX_HEIGHT\
    \n);"));
    solver.new_line();

    solver.add(format!("% No overlap between any two nodes"));
    solver.add(format!(
        "constraint forall(i, j in 1..N where i < j)(\
        \n  (x[i] + widths[i] <= x[j] \\/ x[j] + widths[j] <= x[i]) \\/\
        \n  (y[i] + heights[i] <= y[j] \\/ y[j] + heights[j] <= y[i])\
    \n);"));
    solver.new_line();

    solver.add(format!("% binding the maximum positions"));
    solver.add(format!(
        "constraint forall(i in 1..N)(\
        \n  x[i] + widths[i] <= max_x_position /\\\
        \n  y[i] + heights[i] <= max_y_position\
    \n);"));
    solver.new_line();

    solver.add(format!("% ========= additional constraints ========="));
    solver.add(format!("% first node should be in quadrant one (bottom-left)"));
    solver.add(format!("constraint x[1] < ((max_x_position + 1) div 2);")); 
    solver.add(format!("constraint y[1] < ((max_y_position + 1) div 2);"));
    solver.new_line();

    solver.add(format!("% add a restriction of the ration between max_x_position and max_y_position"));
    solver.add(format!("% the ratio should be greater than 0.5 and less than 2"));
    solver.add(format!("constraint max_x_position * 2 >= max_y_position;")); 
    solver.add(format!("constraint max_x_position <= max_y_position * 2;")); 
    solver.new_line();
 
    solver.add(format!("% add Manhattan constraints for each pair of nodes"));
    solver.add(format!("int: E = {};", fp.source_node.len()));
    solver.add(format!("array [1..N] of int: TOP_SIZE = {:?};", fp.top_space.iter().map(|s| s).collect::<Vec<_>>()));
    solver.add(format!("array [1..N] of int: BOTTOM_SIZE = {:?};", fp.bottom_space.iter().map(|s| s).collect::<Vec<_>>()));
    solver.add(format!("array [1..N] of int: LEFT_SIZE = {:?};", fp.left_space.iter().map(|s| s).collect::<Vec<_>>()));
    solver.add(format!("array [1..E] of int: edge_sources = {:?};", fp.source_node.iter().map(|s| s + 1).collect::<Vec<_>>()));
    solver.add(format!("array [1..E] of int: edge_targets = {:?};", fp.target_node.iter().map(|s| s + 1).collect::<Vec<_>>()));
    solver.add(format!("array [1..E] of int: edge_source_ports = {:?};", fp.source_port.iter().map(|s| s).collect::<Vec<_>>()));
    solver.add(format!("array [1..E] of int: edge_target_ports = {:?};", fp.target_port.iter().map(|s| s).collect::<Vec<_>>()));
    solver.add(format!("array [1..E] of int: edge_conns = {:?};", fp.conn.iter().map(|s| s).collect::<Vec<_>>()));
    solver.new_line();
    solver.add(format!("array [1..E] of var 0..MAX_WIDTH: source_x;"));
    solver.add(format!("array [1..E] of var 0..MAX_HEIGHT: source_y;"));
    solver.add(format!("array [1..E] of var 0..MAX_WIDTH: target_x;"));
    solver.add(format!("array [1..E] of var 0..MAX_HEIGHT: target_y;"));
    solver.add(format!("constraint forall(i in 1..E)(\
        \n  source_x[i] = x[edge_sources[i]] + edge_source_ports[i] + LEFT_SIZE[edge_sources[i]]/\\\
        \n  source_y[i] = y[edge_sources[i]] + heights[edge_sources[i]] - TOP_SIZE[edge_sources[i]]/\\\
        \n  target_x[i] = x[edge_targets[i]] + edge_target_ports[i] + LEFT_SIZE[edge_targets[i]]/\\\
        \n  target_y[i] = y[edge_targets[i]] + BOTTOM_SIZE[edge_targets[i]]\
    \n);")); 
    solver.add(format!("array [1..E] of var 0..MAX_WIDTH: dx;"));
    solver.add(format!("array [1..E] of var 0..MAX_HEIGHT: dy;"));
    solver.add(format!("array [1..E] of var 0..(MAX_WIDTH + MAX_HEIGHT): distance_matrix;"));
    solver.add(format!("array [1..E] of var 0..((MAX_WIDTH + MAX_HEIGHT)*{}): weighted_distance;", fp.conn.iter().max().unwrap()));
    solver.add(format!("constraint forall(i in 1..E)(\
        \n  dx[i] = abs(source_x[i] - target_x[i]) /\\\
        \n  dy[i] = abs(source_y[i] - target_y[i]) /\\\
        \n  distance_matrix[i] = dx[i] + dy[i] /\\\
        \n  weighted_distance[i] = edge_conns[i] * distance_matrix[i]\
    \n);")); 
    solver.new_line();
    solver.new_line();

    solver.add(format!("% ========= objective ========="));
    let (weight_area, weight_distance) = (2, 1); 
    solver.add(format!("solve minimize ({} * sum(weighted_distance) + {} * (max_x_position * max_y_position));", weight_distance, weight_area)); 
    solver.new_line();
    solver.new_line();
    
    solver.add(format!("% ========= output ========="));
    solver.add(String::from("output [\"{\
        x:\\(x),\
        y:\\(y),\
        max_x_position:\\(max_x_position),\
        max_y_position:\\(max_y_position),\
        distance_matrix:\\(distance_matrix)\
    }\"];")); 

    /* solving the model */
    let (status, solutions) = solver.solve("cp-sat", 180, "")?;
    match status.as_str() {
        "OPTIMAL_SOLUTION" | "FEASIBLE" => {}
        _ => return Err(format!("MiniZinc status: {}", status).into()),
    };
    debug!("Minizinc solutions: \n{:?}", solutions);
    let parsed_json_value: serde_json::Value = serde_json::from_str(&solutions[0])?;

    // get the minizinc results 
    let x: Vec<i32> = parsed_json_value
        .get("x")
        .and_then(|v| v.as_array())
        .ok_or_else(|| {
            Box::<dyn std::error::Error>::from("x not found")
        })?
        .iter()
        .map(|val| val.as_i64().unwrap_or(0) as i32) 
        .collect();

    let y: Vec<i32> = parsed_json_value
        .get("y")
        .and_then(|v| v.as_array())
        .ok_or_else(|| {
            Box::<dyn std::error::Error>::from("y not found")
        })?
        .iter()
        .map(|val| val.as_i64().unwrap_or(0) as i32) 
        .collect();

    let max_x_position: i32 = parsed_json_value
        .get("max_x_position")
        .and_then(|v| v.as_i64())
        .map(|v| v as i32)
        .ok_or_else(|| {
            Box::<dyn std::error::Error>::from("max x position not found")
        })?;

    let max_y_position: i32 = parsed_json_value
        .get("max_y_position")
        .and_then(|v| v.as_i64()) 
        .map(|v| v as i32)
        .ok_or_else(|| {
            Box::<dyn std::error::Error>::from("max y position not found")
        })?;

    // update the floor plan
    fp.max_width = max_x_position;
    fp.max_height = max_y_position;

    let pos: Vec<RectanglePosition> = x.into_iter().zip(y.into_iter())
        .map(|(x_val, y_val)| RectanglePosition { x: x_val, y: y_val })
        .collect();
    fp.pos = pos;

    Ok(())
}



pub fn generate_placement(
    db: &DataBase,
    fp: &FloorPlan,
    module_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {

    let max_x = fp.max_width;
    let max_y = fp.max_height;
    let step_x_label = (max_x / 20) + 1;
    let step_y_label = (max_y / 20) + 1;
    let resolution = match max_x * max_y {
        a if a > 100_000 => 10000,
        a if a > 50_000 => 4000,
        a if a > 10_000 => 2000,
        a if a > 5_000 => 1000,
        _ => 800,
    } as u32;

    if resolution >= 10000 {
        warn!("the floorplan is too large to be presented in the graph!");
        return Ok(())
    }

    let output_path = format!("{}/placement.png", module_dir);
  
    let root = BitMapBackend::new(&output_path, (resolution, resolution)).into_drawing_area();
    root.fill(&WHITE)?;

    let margin = 10;
    let mut chart = ChartBuilder::on(&root)
        .margin(margin)
        .caption("Floorplan", ("sans-serif", 50))
        .x_label_area_size(20)
        .y_label_area_size(20)
        .build_cartesian_2d(0..max_x, 0..max_y)?;

    chart.configure_mesh()
        .x_desc("X")
        .y_desc("Y")
        .x_labels(((max_x / step_x_label) + 1) as usize)
        .y_labels(((max_y / step_y_label) + 1) as usize)
        .x_label_formatter(&|x| format!("{}", *x as i32)) // force integer labels
        .y_label_formatter(&|y| format!("{}", *y as i32))
        .x_label_style(("sans-serif", 18))
        .y_label_style(("sans-serif", 18))
        .axis_desc_style(("sans-serif", 20).into_font().style(FontStyle::Bold))
        .light_line_style(&BLACK.mix(0.2))
        .draw()?;
       
    let mut rng = rand::thread_rng();

    for (i, (start, size)) in fp.pos.iter().zip(fp.shape.iter()).enumerate() {
        let min_brightness = 0.3; // 0 = black, 1 = full brightness

        let x = rng.gen_range(min_brightness..=1.0);
        let y = rng.gen_range(min_brightness..=1.0);
        let z = rng.gen_range(min_brightness..=1.0);

        let r = RGBColor((x * 255.0) as u8, (y * 255.0) as u8, (z * 255.0) as u8);
        let black = RGBColor(0, 0, 0);

        let (width, height, ob, ib, left, right, top, bottom) = dimension(db, &fp.app_node_ids[i])?;
        let x0 = start.x + left;
        let y0 = start.y + ib + bottom;
        let w = size.width - left - right;
        let h = size.height - ob - ib - top - bottom;
        assert_eq!(w, width, "placement dimension error");
        assert_eq!(h, height, "placement dimension error");

        chart.draw_series(std::iter::once(Rectangle::new(
            [(x0, y0), (x0 + w, y0 + h)],
            r.filled(),
        )))?;

        chart.draw_series(std::iter::once(Text::new(
            fp.app_node_ids[i].clone(),
            (x0 + w / 5, y0 + h / 2),
            ("sans-serif", 50).into_font().style(FontStyle::Bold).color(&black),
        )))?;
    }

    root.present()?;
    info!("Saved floorplan to: {}", output_path);
    Ok(())
}


fn update_placement(
    db: &mut DataBase,
    fp: &FloorPlan,
) -> Result<(), Box<dyn std::error::Error>> {
    // nodes and their coordinates
    for (index, node_id) in fp.app_node_ids.iter().enumerate() {
        let (_, _, _, ib, left, _, _, bottom) = dimension(&db, &node_id)?;
        let placement = Placement {
            app_node_id: node_id.clone(),
            x: fp.pos[index].x + left,
            y: fp.pos[index].y + ib + bottom,
        };
        db.synthesized_information.placements.push(placement);
    }
    
    // area information
    db.synthesized_information.max_width = fp.max_width;
    db.synthesized_information.max_height = fp.max_height;
    Ok(())
}



#[allow(unused_variables)]
pub fn run(
    db: &mut DataBase, 
    dir: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Start: placement");
    let module_dir = format!("{}place", dir);
    match std::fs::create_dir_all(&module_dir) {
        Ok(_) => (),
        Err(e) => {
            error!("Failed to create {} with {}", module_dir, e);
            return Err(Box::new(e));
        },
    };

    info!("Stege 1: verifying dimension size");
    verify_dimension(db)?;

    info!("Stege 2: creating floor plan");
    let mut fp = create_floor_plan(db)?;
    debug!("working floorplan: \n {:?}", fp);

    info!("Stege 3: start optimal placement");
    match place_solve_optimal(&mut fp, module_dir.clone()) {
        Ok(()) => (),
        err => {
            warn!("Cannot find an optimal solution for the placement problem");
            return err;
        },
    };
    
    info!("Stege 4: start approximate optimal placement with width={}, height={}",
        fp.max_width,
        fp.max_height
    );
    // first relax the width and height constraints from the previous solution
    fp.max_width = (fp.max_width as f64 * db.hyper_parameter.place_relaxation_factor).round() as i32;
    fp.max_height = (fp.max_height as f64 * db.hyper_parameter.place_relaxation_factor).round() as i32;
    match place_solve_approx_optimal(&mut fp, module_dir.clone()) {
        Ok(()) => (),
        err => {
            warn!("Cannot find an approximate optimal solution for the placement problem");
            return err;
        },
    };

    info!("Stege 5: generating placement results and updating synthesized information");
    debug!("Floorplan: \n {:?}", fp);
    generate_placement(db, &fp, &module_dir)?;
    update_placement(db, &mut fp)?;

    info!("Finish: placement");
    Ok(())
}
