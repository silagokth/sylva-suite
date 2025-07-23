use crate::model::{DataBase, FloorPlan, RectangleShape, RectanglePosition, Placement};
use crate::solver::Solver;
use log::{info, warn, error, debug};
use serde_json;
use plotters::style::{Color, BLACK, FontStyle};
use plotters::prelude::*;
use rand::Rng;



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
        conn: vec![],
    };

    // add node information from alimp bindings
    for node in &db.app_graph.nodes {
        let instance = db.synthesized_information.alimp_bindings
                        .iter()
                        .find(|b| b.app_node_id == node.id)
                        .map(|b| &b.alimp_instance)
                        .ok_or_else(|| {
                            Box::from(format!("Cannot find the alimp instance {}", node.id))
                            as Box<dyn std::error::Error>
                        })?;
        
        // The height is inflated: 1 for input buffer, 
        // 1 for output buffer, 
        // 1 for the transporters attached to output buffer, 
        // and 1 on each side for routing space.
        //
        // The width is inflated: 1 on each side for routing space.
        let routing_reserved_size = &db.hyper_parameter.place_reserved_routing_size;
        let shape = RectangleShape {
            width: instance.width + 2 * routing_reserved_size,
            height: instance.height + 3 + 2 * routing_reserved_size,
        };
        
        // dummy positions
        let position = RectanglePosition{
            x: -1, 
            y: -1,
        };

        fp.shape.push(shape);
        fp.pos.push(position);
        fp.app_node_ids.push(node.id.clone());
    }

    // add edge information 
    for edge in &db.app_graph.edges {
        let (source_node_index, source_node) = db.app_graph.nodes
                                                .iter()
                                                .enumerate()
                                                .find(|&(_, ref entry)| entry.id == edge.source_node)
                                                .map(|(i, entry)| (i as i32, entry)) 
                                                .ok_or_else(|| {
                                                    Box::from(format!("Cannot find the source node {}", edge.source_node))
                                                    as Box<dyn std::error::Error>
                                                })?;

        let (target_node_index, target_node) = db.app_graph.nodes
                                                .iter()
                                                .enumerate()
                                                .find(|&(_, ref entry)| entry.id == edge.target_node)
                                                .map(|(i, entry)| (i as i32, entry)) 
                                                .ok_or_else(|| {
                                                    Box::from(format!("Cannot find the target node {}", edge.target_node))
                                                    as Box<dyn std::error::Error>
                                                })?;

        let source_port_index = source_node.output_ports
                                .iter()
                                .enumerate()
                                .find(|&(_, ref entry)| entry.id == edge.source_port)
                                .map(|(i, _)| i as i32) 
                                .ok_or_else(|| {
                                    Box::from(format!("Cannot find the source port {} in {}", edge.source_port, edge.source_node))
                                    as Box<dyn std::error::Error>
                                })?;
 
        let target_port_index = target_node.input_ports
                                .iter()
                                .enumerate()
                                .find(|&(_, ref entry)| entry.id == edge.target_port)
                                .map(|(i, _)| i as i32) 
                                .ok_or_else(|| {
                                    Box::from(format!("Cannot find the target port {} in {}", edge.source_port, edge.source_node))
                                    as Box<dyn std::error::Error>
                                })?;           

        fp.source_node.push(source_node_index);
        fp.target_node.push(target_node_index);
        fp.source_port.push(source_port_index);
        fp.target_port.push(target_port_index);
        fp.conn.push(edge.token_size.clone());
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
    solver.add(format!("constraint x[1] < ((MAX_WIDTH + 1) div 2);")); 
    solver.add(format!("constraint y[1] < ((MAX_HEIGHT + 1) div 2);"));
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
    let (status, solutions) = solver.solve("")?;
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
    db: &mut DataBase,
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
    solver.add(format!("constraint x[1] < ((MAX_WIDTH + 1) div 2);")); 
    solver.add(format!("constraint y[1] < ((MAX_HEIGHT + 1) div 2);"));
    solver.new_line();

    solver.add(format!("% add a restriction of the ration between max_x_position and max_y_position"));
    solver.add(format!("% the ratio should be greater than 0.5 and less than 2"));
    solver.add(format!("constraint max_x_position * 2 >= max_y_position;")); 
    solver.add(format!("constraint max_x_position <= max_y_position * 2;")); 
    solver.new_line();
 
    solver.add(format!("% add Manhattan constraints for each pair of nodes"));
    solver.add(format!("int: E = {};", fp.source_node.len()));
    solver.add(format!("int: ROUTING_SIZE = {};", db.hyper_parameter.place_reserved_routing_size));
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
        \n  source_x[i] = x[edge_sources[i]] + edge_source_ports[i] + ROUTING_SIZE/\\\
        \n  source_y[i] = y[edge_sources[i]] + heights[edge_sources[i]] - 2*ROUTING_SIZE/\\\
        \n  target_x[i] = x[edge_targets[i]] + edge_target_ports[i] + ROUTING_SIZE/\\\
        \n  target_y[i] = y[edge_targets[i]] + ROUTING_SIZE\
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
    solver.add(format!("solve minimize sum(weighted_distance);")); 
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
    let (status, solutions) = solver.solve("")?;
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
    let output_path = format!("{}/placement.png", module_dir);
  
    let root = BitMapBackend::new(&output_path, (1000, 1000)).into_drawing_area();
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
        .x_labels((max_x - 0) as usize) // number of x labels (1 per unit)
        .y_labels((max_y - 0) as usize) // number of y labels (1 per unit)
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

        let offset = db.hyper_parameter.place_reserved_routing_size;
        let x0 = start.x + offset;
        let y0 = start.y + 1 + offset;
        let w = size.width - 2 * offset;
        let h = size.height - 3 - 2 * offset;

        chart.draw_series(std::iter::once(Rectangle::new(
            [(x0, y0), (x0 + w, y0 + h)],
            r.filled(),
        )))?;

        chart.draw_series(std::iter::once(Text::new(
            fp.app_node_ids[i].clone(),
            (x0 + w / 2, y0 + h / 2),
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
) -> () {
    // nodes and their coordinates
    for (index, node_id) in fp.app_node_ids.iter().enumerate() {
        let placement = Placement {
            app_node_id: node_id.clone(),
            x: fp.pos[index].x + 1,
            y: fp.pos[index].y + 2,
        };
        db.synthesized_information.placements.push(placement);
    }
    
    // area information
    db.synthesized_information.max_width = fp.max_width;
    db.synthesized_information.max_height = fp.max_height;
}



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

    info!("Stege 1: creating floor plan");
    let mut fp = create_floor_plan(db)?;
    debug!("working floorplan: \n {:?}", fp);

    info!("Stege 2: start optimal placement");
    match place_solve_optimal(&mut fp, module_dir.clone()) {
        Ok(()) => (),
        err => {
            warn!("Cannot find an optimal solution for the placement problem");
            return err;
        },
    };
    
    info!("Stege 3: start approximate optimal placement with width={}, height={}",
        fp.max_width,
        fp.max_height
    );
    // first relax the width and height constraints from the previous solution
    fp.max_width = (fp.max_width as f64 * db.hyper_parameter.place_relaxation_factor).round() as i32;
    fp.max_height = (fp.max_height as f64 * db.hyper_parameter.place_relaxation_factor).round() as i32;
    match place_solve_approx_optimal(db, &mut fp, module_dir.clone()) {
        Ok(()) => (),
        err => {
            warn!("Cannot find an approximate optimal solution for the placement problem");
            return err;
        },
    };

    info!("Stege 3: generating placement results and updating synthesized information");
    debug!("Floorplan: \n {:?}", fp);
    generate_placement(db, &fp, &module_dir)?;
    update_placement(db, &mut fp);

    info!("Finish: placement");
    Ok(())
}
