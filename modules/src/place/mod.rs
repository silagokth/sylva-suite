use crate::model::{DataBase, FloorPlan, RectangleShape, RectanglePosition};
use crate::solver::Solver;
use log::{info, warn, error, debug};
use itertools::Itertools;
use std::collections::HashMap;
use serde_json;

fn create_floor_plan(db: &mut DataBase) -> Result<FloorPlan, Box<dyn std::error::Error>> {
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
    for binding in &db.synthesized_information.alimp_bindings {
        let instance = &binding.alimp_instance;
        
        // The height is inflated: 1 for input buffer, 
        // 1 for output buffer, 
        // 1 for the transporters attached to output buffer, 
        // and 1 on each side for routing space.
        //
        // The width is inflated: 1 on each side for routing space.
        let routing_space_scaling = &db.hyper_parameter.place_reserved_routing_size;
        let shape = RectangleShape {
            width: instance.width + 2 * routing_space_scaling,
            height: instance.height + 3 + 2 * routing_space_scaling,
        };
        
        // dummy positions
        let position = RectanglePosition{
            x: -1, 
            y: -1,
        };

        fp.shape.push(shape);
        fp.pos.push(position);
        fp.app_node_ids.push(binding.app_node_id.clone());
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
fn place_solve_optimal(fp: &mut FloorPlan, module_dir: String) -> Result<(), Box<dyn std::error::Error>> {

    let mut solver = Solver::new(String::from("place_optimal"), module_dir);

    /* formulating the model */
    solver.add(format!("% minimising the total area to get estimated max width and height")); 
    solver.add(format!("int: N = {};", fp.shape.len()));
    solver.add(format!("int: MAX_WIDTH = {};", fp.max_width - 1)); // coordinates start from 0
    solver.add(format!("int: MAX_HEIGHT = {};", fp.max_height - 1)); // coordinates start from 0
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

    solver.add(format!("% No overlap between any two nodes in both x and y directions"));
    solver.add(format!(
        "constraint forall(i, j in 1..N where i < j)(\
        \n  (x[i] + widths[i] <= x[j] \\/ x[j] + widths[j] <= x[i]) /\\\
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
    solver.add(String::from("output [\"{max_x_position:\\(max_x_position), max_y_position:\\(max_y_position)}\"];")); 

    /* solving the model */
    let acceptable_status = vec![String::from("OPTIMAL_SOLUTION")];
    let solutions = solver.solve("", acceptable_status)?;
    debug!("Minizinc solutions: \n{:?}", solutions);
    let parsed_json_value: serde_json::Value = serde_json::from_str(&solutions[0])?;

    // get the maximum positions from the minizinc result
    let max_x_position: i32 = parsed_json_value
        .get("max_x_position")
        .and_then(|v| v.as_i64()) 
        .ok_or_else(|| {
            Box::from("max x position not found")
            as Box<dyn std::error::Error>
        })? as i32;

    let max_y_position: i32 = parsed_json_value
        .get("max_y_position")
        .and_then(|v| v.as_i64()) 
        .ok_or_else(|| {
            Box::from("max x position not found")
            as Box<dyn std::error::Error>
        })? as i32;

    // update floor plan 
    // need to add +1 because the coordinates start from 0
    fp.max_width = max_x_position + 1;
    fp.max_height = max_y_position + 1;

    Ok(())
}




/*
 *
 *
 * */
fn place_solve_approx_optimal(fp: &mut FloorPlan, module_dir: String) -> Result<(), Box<dyn std::error::Error>> {
    

    Ok(())
}




pub fn run(db: &mut DataBase, dir: &String) -> Result<(), Box<dyn std::error::Error>> {
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
    fp.max_width = fp.max_width * db.hyper_parameter.place_relaxation_factor;
    fp.max_height = fp.max_height * db.hyper_parameter.place_relaxation_factor;
    match place_solve_approx_optimal(&mut fp, module_dir.clone()) {
        Ok(()) => (),
        err => {
            warn!("Cannot find an approximate optimal solution for the placement problem");
            return err;
        },
    };

    info!("Stege 3: generating placement results and updating synthesized information");
    

    info!("Finish: placement");
    Ok(())
}
