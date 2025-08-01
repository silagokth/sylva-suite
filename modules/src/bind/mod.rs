use sv_lib::model::{DataBase, AppGraph, AlimpBindingOption, AlimpBinding};
use crate::solver::Solver;
use log::{info, error, debug};
use itertools::Itertools;
use std::collections::HashMap;
use serde_json;
use std::sync::{Arc, atomic::AtomicBool};

fn get_predecessor_list(app_graph: &AppGraph) -> Result<HashMap<i32, Vec<i32>>, Box<dyn std::error::Error>> {
    let mut predecessors: HashMap<i32, Vec<i32>> = HashMap::new();

    for edge in &app_graph.edges {
        // Find source and target node indices by their IDs
        let source_index = match app_graph.nodes.iter().enumerate().find(|(_, n)| n.id == edge.source_node) {
            Some((i, _)) => i as i32,
            None => return Err(Box::from(format!("Cannot find the source node '{}'", edge.source_node))),
        };

        let target_index = match app_graph.nodes.iter().enumerate().find(|(_, n)| n.id == edge.target_node) {
            Some((i, _)) => i as i32,
            None => return Err(Box::from(format!("Cannot find the target node '{}'", edge.target_node))),
        };

        // Add the source index as a predecessor of the target index
        predecessors.entry(target_index + 1).or_default().push(source_index + 1);
    }

    Ok(predecessors)
}


fn add_bind_constraints(db: &DataBase, solver: &mut Solver, max_objective_value: i32) -> Result<(), Box<dyn std::error::Error>> {
    
    // Geometry constraint:
    //  The width and height of any alimp cannot be larger than the global width and height
    //  The area of all selected alimps cannot be larger than the global area 
    //
    // Energy constraint:
    //  The energy of all selected alimps cannot be larger than the global energy

    // create variables and constraints for area and energy 
    solver.add(format!("array[1..{}] of var int: area_nodes;", db.app_graph.nodes.len()));
    solver.add(format!("array[1..{}] of var int: energy_nodes;", db.app_graph.nodes.len()));
    solver.new_line();
    
    let mut node_index = 1; 
    for node in &db.app_graph.nodes {
        // get Alimp instances of this node
        let alimp_instances = match db.alimp_lib
                                        .entries
                                        .iter()
                                        .find(|entry| entry.func == node.func) {
            Some(entry) => &entry.instances,        
            None => return Err(Box::from(format!("Cannot find Alimp entry from App Graph"))),
        };
       
        // selected Alimp index
        let number_of_options = alimp_instances.len();
        solver.add(format!("var 1..{}: selected_alimp_{};", number_of_options, node.id));
        
        // Parameters from all instances of this node 
        let widths = alimp_instances
            .iter()
            .map(|inst| format!("{}", inst.width))
            .join(", ");
        solver.add(format!("array [1..{}] of int: width_{} = [{}];", number_of_options, node.id, widths));

        let heights = alimp_instances
            .iter()
            .map(|inst| format!("{}", inst.height))
            .join(", ");
        solver.add(format!("array [1..{}] of int: height_{} = [{}];", number_of_options, node.id, heights));

        let energies = alimp_instances
            .iter()
            .map(|inst| format!("{}", inst.energy))
            .join(", ");
        solver.add(format!("array [1..{}] of int: energy_{} = [{}];", number_of_options, node.id, energies));

        // add local constraints of the node
        solver.add(format!("constraint width_{}[selected_alimp_{}] <= {};", 
            node.id, 
            node.id, 
            db.global_constraint.max_width
        ));
        solver.add(format!("constraint height_{}[selected_alimp_{}] <= {};", 
            node.id, 
            node.id, 
            db.global_constraint.max_height
        ));
        
        // set total area of the node and energy to the global variables 
        solver.add(format!("constraint width_{}[selected_alimp_{}] * height_{}[selected_alimp_{}] == area_nodes[{}];", 
            node.id, 
            node.id, 
            node.id, 
            node.id, 
            node_index
        )); 
        solver.add(format!("constraint energy_{}[selected_alimp_{}] == energy_nodes[{}];", 
            node.id, 
            node.id, 
            node_index
        ));

        solver.new_line();
        node_index += 1;
    }
    
    solver.add(String::from("% add global geometric and energy constraints")); 
    solver.add(format!("constraint sum(area_nodes) <= {};", db.global_constraint.max_width * db.global_constraint.max_height)); 
    solver.add(format!("constraint sum(energy_nodes) <= {};", db.global_constraint.max_energy)); 
    solver.new_line();
    solver.new_line();

    // add latency constraints 
    // 1. we need a predecessor list to find out how much latency of 
    //    previous nodes affect the current one.
    // 2. Assuming that the current one can be fired 
    //    after half a latency of all its previous nodes.
    
    solver.add(String::from("% add latency constraints")); 
    solver.add(format!("array[1..{}] of var 0..{}: latency_nodes;", db.app_graph.nodes.len(), db.global_constraint.max_latency));
    solver.add(format!("array[1..{}] of var 0..{}: start_time_nodes;", db.app_graph.nodes.len(), db.global_constraint.max_latency));
    solver.add(format!("array[1..{}] of var 0..{}: end_time_nodes;", db.app_graph.nodes.len(), db.global_constraint.max_latency));
    solver.new_line();
   
    let predecessors: HashMap<i32, Vec<i32>> = get_predecessor_list(&db.app_graph)?;
    node_index = 1;
    for node in &db.app_graph.nodes {
        // get Alimp instances of this node
        let alimp_instances = match db.alimp_lib
                                        .entries
                                        .iter()
                                        .find(|entry| entry.func == node.func) {
            Some(entry) => &entry.instances,        
            None => return Err(Box::from(format!("Cannot find Alimp entry from App Graph"))),
        };
        
        let number_of_options = alimp_instances.len();
         
        // Parameters from all instances of this node 
        let latencies = alimp_instances
            .iter()
            .map(|inst| format!("{}", inst.latency))
            .join(", ");
        solver.add(format!("array [1..{}] of int: latency_{} = [{}];", number_of_options, node.id, latencies));
    
        // select the latency of each node 
        solver.add(format!("constraint latency_{}[selected_alimp_{}] == latency_nodes[{}];", 
            node.id, 
            node.id, 
            node_index
        ));

        // add start time constraints
        if !predecessors.contains_key(&node_index) {
            solver.add(format!("constraint start_time_nodes[{}] == 0;", node_index));
        } else {
            if let Some(prev_indices) = predecessors.get(&node_index) {
                for prev_index in prev_indices {
                    solver.add(format!("constraint start_time_nodes[{}] >= start_time_nodes[{}] + (latency_nodes[{}] div 2);", 
                            node_index,
                            prev_index,
                            prev_index
                    ));
                }
            } 
        }

        // add end time constraints
        solver.add(format!("constraint end_time_nodes[{}] <= {};", 
            node_index,
            db.global_constraint.max_latency
        ));
        solver.add(format!("constraint end_time_nodes[{}] == start_time_nodes[{}] + latency_nodes[{}];", 
            node_index,
            node_index,
            node_index
        ));

        // add throughput constraint: Any alimp instance should have 
        // a latency that is smaller than the max period
        solver.add(format!("constraint latency_nodes[{}] <= {}; % throughput contraint", 
            node_index,
            db.global_constraint.max_period
        ));

        solver.new_line();
        node_index += 1;
    }

    solver.new_line();
    solver.new_line();
    solver.add(String::from("% add objective and model problem")); 
    solver.add(format!("var 0..{}: objective;", max_objective_value));
    solver.add(format!("constraint objective == ({} * sum(area_nodes)) + ({} * sum(energy_nodes));", 
        db.hyper_parameter.bind_w_area,
        db.hyper_parameter.bind_w_energy
    ));

    Ok(())
}


fn optimal_binding(db: &mut DataBase, interrupt: &Arc<AtomicBool>, module_dir: String) -> std::result::Result<i64, Box<dyn std::error::Error>> {
    // formulating model 
    let mut solver = Solver::new(String::from("bind_optimal"), module_dir);
    add_bind_constraints(db, &mut solver, 1000000)?;
    solver.add(String::from("solve minimize objective;")); 
    solver.add(String::from("output [\"{objective:\\(objective)}\"];")); 

    // solving
    let (status, solutions) = solver.solve("", 20, interrupt)?;
    match status.as_str() {
        "OPTIMAL_SOLUTION" => {}
        _ => return Err(format!("MiniZinc status: {}", status).into()),
    };
    debug!("Minizinc solutions: \n{:?}", solutions);
    let parsed_json_value: serde_json::Value = serde_json::from_str(&solutions[0])?;
    
    // return the minimized objective value 
    let objective_value: i64 = parsed_json_value
        .get("objective")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| {
            Box::from("objective not found")
            as Box<dyn std::error::Error>
        })?;
    Ok(objective_value)
}


fn approximate_optimal_binding(db: &mut DataBase, interrupt: &Arc<AtomicBool>, module_dir: String, minimized_objective: i64) -> std::result::Result<Vec<HashMap<String, i32>>, Box<dyn std::error::Error>> {
    // add new objective and its constraint, and search for all feasible solutions
    // These numbers should be adjusted
    let mut solver = Solver::new(String::from("approx_bind_optimal"), module_dir);
    add_bind_constraints(db, &mut solver, 2*1000000)?;
    solver.add(format!("constraint objective <= 2 * {};", minimized_objective)); 
    solver.add(String::from("solve satisfy;")); 
    
    // formatting solver output 
    let mut output_str = String::from("output [\"{objective:\\(objective)");
    for node in &db.app_graph.nodes {
        let name = format!("selected_alimp_{}", node.id);
        output_str.push_str(&format!(", {}:\\({})", name, name));
    }
    output_str.push_str("}\"]");
    solver.add(output_str);

    // solving 
    let (status, solutions) = solver.solve("-a", 20, interrupt)?; // -a: search for all
    match status.as_str() {
        "ALL_SOLUTIONS" => {} 
        // allow time-out (unknown sending ctrl-c)
        "UNKNOWN" => {
            if solutions.is_empty() {
                return Err(format!("MiniZinc status: {}", status).into());
            }
        }
        _ => return Err(format!("MiniZinc status: {}", status).into()),
    };
    debug!("Minizinc solutions: \n{:?}", solutions);

    // put solutions into a suitable HashMap object
    let mut ret: Vec<HashMap<String, i32>> = vec![];
    for sol in solutions {
        let sol_json: serde_json::Value = serde_json::from_str(&sol)?;

        let mut j = HashMap::new();
        for node in &db.app_graph.nodes {
            let name = format!("selected_alimp_{}", node.id);
            let selected_position: i32 = sol_json
                .get(name)
                .and_then(|v| v.as_i64())
                .ok_or_else(|| {
                    Box::from("JSON field not found")
                    as Box<dyn std::error::Error>
                })? as i32;
            // -1 because it starts from 1 in minizinc
            j.entry(node.id.clone()).or_insert(selected_position - 1);
        }
        ret.push(j);
    }

    Ok(ret)
}


fn apply_binding(db: &mut DataBase, bindings: Vec<HashMap<String, i32>>) -> std::result::Result<(), Box<dyn std::error::Error>> {
    
    // clear binding option field in db
    db.alimp_binding_options = vec![];

    for sol in bindings {

        // create new binding option
        let mut binding_option: AlimpBindingOption = AlimpBindingOption{
            alimp_bindings: vec![],
        };

        for (node_id, alimp_index) in sol.into_iter() {
            // get node func from 
            let node_func = match db.app_graph
                                    .nodes
                                    .iter()
                                    .find(|entry| entry.id == node_id) {
                Some(entry) => &entry.func,
                _ => return Err(Box::from("Cannot find node func in App Graph")),
            };

            // get alimp instances of this node
            let alimp_instances = match db.alimp_lib
                                        .entries
                                        .iter()
                                        .find(|entry| entry.func == *node_func) {
                Some(entry) => &entry.instances,        
                _ => return Err(Box::from("Cannot find Alimp entry from App Graph")),
            };

            // select alimp instance
            let target_alimp_instance = &alimp_instances[alimp_index as usize];

            // map to alimp binding options
            binding_option.alimp_bindings.push(
                AlimpBinding {
                    app_node_id: node_id,
                    alimp_instance: target_alimp_instance.clone(),
                }
            );
        }
    
        db.alimp_binding_options.push(binding_option);
    }

    Ok(())   
}

#[allow(unused_variables)]
pub fn run(
    db: &mut DataBase, 
    interrupt: &Arc<AtomicBool>, 
    dir: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Start: binding");
    let module_dir = format!("{}bind", dir);
    match std::fs::create_dir_all(&module_dir) {
        Ok(_) => (),
        Err(e) => {
            error!("Failed to create {} with {}", module_dir, e);
            return Err(Box::new(e));
        },
    };

    info!("Stage 1: optimal binding"); 
    let minimized_objective = optimal_binding(db, interrupt, module_dir.clone())?;
    
    info!("Stage 2: approximate optimal binding with the objective value {}", minimized_objective); 
    let valid_bindings = approximate_optimal_binding(db, interrupt, module_dir.clone(), minimized_objective)?;
    
    info!("Stage 3: found {} solutions starting the binding process", valid_bindings.len()); 
    debug!("valid bindings are \n {:?}", valid_bindings); 
    apply_binding(db, valid_bindings)?; 

    info!("Stage 4 (TODO): add only the first binding option to synthesized information"); 
    for binding in &db.alimp_binding_options[0].alimp_bindings {
        db.synthesized_information.alimp_bindings.push(binding.clone());
    }

    info!("Finish: binding");
    Ok(())
}


















