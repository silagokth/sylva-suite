use sv_lib::model::{DataBase, PairIntInt};
use crate::solver::Solver;
use log::{debug};
use serde_json;
use std::collections::{HashMap, HashSet};
use itertools::Itertools;


pub fn solve_min_delay(
    src_addr_pattern: &HashMap<i32, i32>,
    dst_addr_pattern: &HashMap<i32, i32>,
    channel_width: i32,
    max_delay: i32,
    module_dir: String,
) -> Result<i32, Box<dyn std::error::Error>> {
    
    let src_keys: HashSet<i32> = src_addr_pattern.keys().cloned().collect();
    let dst_keys: HashSet<i32> = dst_addr_pattern.keys().cloned().collect();
    let common_addr: HashSet<i32> = src_keys
        .intersection(&dst_keys)
        .cloned()
        .collect();

    let mut solver = Solver::new(String::from("solve_min_delay"), module_dir);

    /* formulating the model */
    solver.add(format!("int: N = {};", common_addr.len()));
    solver.add(format!("int: WIDTH = {};", channel_width));
    solver.add(format!("int: MAX_DELAY = {};", max_delay));
    solver.add(format!("array [1..N] of int: src_patterns = [{}];", common_addr.iter().map(|k| format!("{}", src_addr_pattern[k])).join(", ")));
    solver.add(format!("array [1..N] of int: dst_patterns = [{}];", common_addr.iter().map(|k| format!("{}", dst_addr_pattern[k])).join(", ")));
    solver.new_line();

    solver.add(format!("var 2..MAX_DELAY: delay;"));
    solver.add(format!("array [1..N] of var 0..MAX_DELAY: tr_read;"));
    solver.new_line();

    solver.add(format!("% ========= constraints ========="));
    solver.add(format!(
        "constraint forall(i in 1..N)(\
        \n  tr_read[i] > src_patterns[i] /\\\
        \n  dst_patterns[i] + delay > tr_read[i] /\\\
        \n  dst_patterns[i] + delay < MAX_DELAY\
    \n);"));
    solver.add(format!(
        " % At any moment, there can be WIDTH transfers in the communication channel\
        \nconstraint forall(x in min(tr_read)..max(tr_read))(\
        \n  count(tr_read, x) <= WIDTH\
    \n);"));
    solver.new_line();
    solver.new_line();

    solver.add(format!("% ========= objective ========="));
    solver.add(format!("solve minimize delay;")); 
    solver.new_line();
    solver.new_line();
    
    solver.add(format!("% ========= output ========="));
    solver.add(String::from("output [\"{\
        delay:\\(delay)\
    }\"];")); 

    /* solving the model */
    let (status, solutions) = solver.solve("--time-limit 30000 -p 8")?;
    match status.as_str() {
        "OPTIMAL_SOLUTION" | "FEASIBLE" => {}
        "UNSATISFIABLE" => return Ok(-1),
        _ => return Err(format!("MiniZinc status: {}", status).into()),
    };
    let parsed_json_value: serde_json::Value = serde_json::from_str(&solutions[0])?;

    // get the minimum delay from the minizinc result
    let delay: i32 = parsed_json_value
        .get("delay")
        .and_then(|v| v.as_i64())
        .map(|v| v as i32)
        .ok_or_else(|| {
            Box::<dyn std::error::Error>::from("delay not found")
        })?;

    Ok(delay)
} 


fn pair_to_hashmap(list: &Vec<PairIntInt>) -> HashMap<i32, i32> {
    list.iter().map(|e| (e.key, e.value)).collect()
}


fn most_frequent_value_count(map: &HashMap<i32, i32>) -> i32 {
    let mut value_counts = HashMap::new();

    for &value in map.values() {
        *value_counts.entry(value).or_insert(0) += 1;
    }

    value_counts.values().copied().max().unwrap_or(0)
}


fn translate_addr_time(
    db: &DataBase,
    node_name: &str,
    port_name: &str,
    dir: &str,
) -> Result<HashMap<i32, i32>, Box<dyn std::error::Error>> {
    let binding = db.synthesized_information.alimp_bindings
        .iter()
        .find(|n| n.app_node_id == node_name)
        .ok_or("Cannot find the address time patterns in Alimp")?;

    let patterns = match dir {
        "in" => pair_to_hashmap(&binding.alimp_instance.input_addr_time_patterns),
        _ => pair_to_hashmap(&binding.alimp_instance.output_addr_time_patterns),
    };

    // get patterns for the port
    let node = db.app_graph.nodes
        .iter()
        .find(|n| n.id == node_name)
        .ok_or("Cannot find the node in app graph")?;
    
    let ports = if dir == "in" {
        &node.input_ports
    } else {
        &node.output_ports
    };

    let mut addr_time_patterns = HashMap::new();
    let mut start_address = 0;
    
    for port in ports {
        let range_address = port.token_size;
        if port.id == port_name {
            for &addr in patterns.keys() {
                if (addr - start_address >= 0) && (addr - start_address < range_address) {
                    if let Some(&time) = patterns.get(&addr) {
                        addr_time_patterns.insert(addr - start_address, time);
                    }
                }
            }
            break;
        } 
        start_address += range_address;
    }

    Ok(addr_time_patterns)
}

fn mean(data: &[i32]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    data.iter().map(|&x| x as f64).sum::<f64>() / data.len() as f64
}

fn std(data: &[i32], mean: f64) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    let variance = data
        .iter()
        .map(|&x| {
            let diff = x as f64 - mean;
            diff * diff
        })
        .sum::<f64>()
        / data.len() as f64;
    variance.sqrt()
}

fn stats_address_patterns(
    pattern1: &HashMap<i32, i32>,
    pattern2: &HashMap<i32, i32>,
) -> (f64, f64) {
    let mut cycles1 = Vec::new();
    let mut cycles2 = Vec::new();

    // find common addresses
    for &addr in pattern1.keys() {
        if let Some(&v2) = pattern2.get(&addr) {
            let v1 = pattern1[&addr];
            cycles1.push(v1);
            cycles2.push(v2);
        }
    }
    
    // count frequency of each cycle
    let mut cycle_counts1 = HashMap::new();
    let mut cycle_counts2 = HashMap::new();

    for cycle in cycles1 {
        *cycle_counts1.entry(cycle).or_insert(0) += 1;
    }
    for cycle in cycles2 {
        *cycle_counts2.entry(cycle).or_insert(0) += 1;
    }

    let addresses_per_cycle1: Vec<i32> = cycle_counts1.values().cloned().collect();
    let addresses_per_cycle2: Vec<i32> = cycle_counts2.values().cloned().collect();
    
    let mean1 = mean(&addresses_per_cycle1);
    let mean2 = mean(&addresses_per_cycle2);
    let std1 = std(&addresses_per_cycle1, mean1);
    let std2 = std(&addresses_per_cycle2, mean2);

    let mean_avg = (mean1 + mean2) / 2.0;
    let std_avg = (std1 + std2) / 2.0;
    (mean_avg, std_avg)
}



pub fn solve_channel_width(
    db: &mut DataBase,
    module_dir: String,
) -> Result<HashMap<String, (i32, i32)>, Box<dyn std::error::Error>> {
 
    let mut solver = Solver::new(String::from("solve_channel_width"), module_dir.clone());

    /* formulating the model */
    solver.add(format!("int: N = {};", db.app_graph.nodes.len()));
    solver.add(format!("int: MAX_DELAY = {};", db.global_constraint.max_latency)); 
    solver.add(format!("array [1..N] of int: EXECUTION_TIME = {:?};", 
        db.app_graph.nodes.iter().map(|n| n.execution_time).collect::<Vec<_>>())
    );
 
    // TODO: add loop for handling more than one repetition
    solver.add(format!("array [1..N] of var 0..MAX_DELAY: fire_time;"));
    solver.add(format!("array [1..N] of var 0..MAX_DELAY: end_time;"));
    solver.new_line();

    solver.add(format!("% ========= constraints for scheduling ========="));
    solver.add(format!("% set end time \
        \nconstraint forall(i in 1..N)(\
        \n  end_time[i] = fire_time[i] + EXECUTION_TIME[i]\
    \n);"));
    solver.add(format!("constraint 0 = min([fire_time[i] | i in 1..N]); % initial fire time is 0")); 
    solver.new_line();
    solver.new_line();

    let node_indices: HashMap<String, usize> = db.app_graph.nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.id.clone(), i + 1)) // MiniZinc is 1-indexed
        .collect();

    let edges = db.app_graph.edges.clone();
    for edge in &edges {
        let source_latency = db.synthesized_information.alimp_bindings
            .iter()
            .find(|a| a.app_node_id == edge.source_node)
            .map(|a| a.alimp_instance.latency)
            .ok_or("cannot find latency in alimp bindings")?;
        let target_latency = db.synthesized_information.alimp_bindings
            .iter()
            .find(|a| a.app_node_id == edge.target_node)
            .map(|a| a.alimp_instance.latency)
            .ok_or("cannot find latency in alimp bindings")?;
        let max_latency_of_two_nodes = source_latency + target_latency + 10; 

        let output_addr_time_patterns = translate_addr_time(db, &edge.source_node, &edge.source_port, "out")?; 
        let input_addr_time_patterns = translate_addr_time(db, &edge.target_node, &edge.target_port, "in")?; 
        let (mean_patterns, _) = stats_address_patterns(&output_addr_time_patterns, &input_addr_time_patterns);
        debug!("edge = {}, mean_patterns = {}", edge.id, mean_patterns);
        
        // find minimum delay value of some K
        let mut k = match mean_patterns {
            x if x < 10.0 => x.max(2.0) as i32 - 1,
            x if x < 50.0 => x as i32 - 2,
            x if x < 100.0 => x as i32 - 4,
            x => x as i32 - 5,
        };     

        let mut k_list: Vec<i32> = Vec::new();
        let mut min_delay_list: Vec<i32> = Vec::new();
        loop {
            let min_delay = solve_min_delay(
                &output_addr_time_patterns,
                &input_addr_time_patterns,
                k,
                max_latency_of_two_nodes,
                module_dir.clone(),
            )?;
            
            if min_delay != -1 {
                // terminate if no improvement
                if min_delay_list.last() == Some(&min_delay) {
                    break;
                }
                k_list.push(k);
                min_delay_list.push(min_delay);
            }
            k += 1;
        }
       
        let idx = &edge.id;
        solver.add(format!("% add variables and constraints for edge {}", idx));
        solver.add(format!("array [1..{}] of int: K_OF_{} = [{}];", k_list.len(), idx, k_list.iter().map(|k| format!("{}", k)).join(", ")));
        solver.add(format!("array [1..{}] of int: MIN_DELAY_OF_{} = [{}];", min_delay_list.len(), idx, min_delay_list.iter().map(|k| format!("{}", k)).join(", ")));
        solver.add(format!("var 1..{}: selected_edge_{};", k_list.len(), idx));
        solver.add(format!("var int: k_{};", idx));
        solver.add(format!("var int: min_delay_{};", idx));
        solver.add(format!("constraint k_{} = K_OF_{}[selected_edge_{}];", idx, idx, idx)); 
        solver.add(format!("constraint min_delay_{} = MIN_DELAY_OF_{}[selected_edge_{}];", idx, idx, idx));  
        solver.add(format!("constraint fire_time[{}] + min_delay_{} < fire_time[{}];",
            node_indices[&edge.source_node],
            edge.id, 
            node_indices[&edge.target_node]
        ));  
        solver.new_line();
        solver.new_line();
    }
     
    solver.add(format!("% ========= objective ========="));
    solver.add(format!("var int: sum_of_k;"));
    solver.add(format!("var int: sum_of_min_delay;"));
    solver.add(format!(
        "constraint sum_of_k = sum([{}]);",
        db.app_graph.edges.iter().map(|e| format!("k_{}", e.id)).collect::<Vec<_>>().join(", ")
    ));
    solver.add(format!(
        "constraint sum_of_min_delay = sum([{}]);",
        db.app_graph.edges.iter().map(|e| format!("min_delay_{}", e.id)).collect::<Vec<_>>().join(", ")
    ));
    solver.add(format!("solve minimize (sum_of_min_delay + 2 * sum_of_k);")); 
    solver.new_line();
    solver.new_line();
    
    solver.add(format!("% ========= output ========="));
    let outputs = db.app_graph.edges.iter()
        .map(|e| format!("k_{}:\\(k_{}), min_delay_{}:\\(min_delay_{})", e.id, e.id, e.id, e.id))
        .collect::<Vec<_>>()
        .join(", ");

    solver.add(format!(
        "output [\"{{end_time:\\(end_time), {}}}\"];", outputs
    ));


    /* solving the model */
    let (status, solutions) = solver.solve("--time-limit 120000 -p 8")?;
    match status.as_str() {
        "OPTIMAL_SOLUTION" => {}
        _ => return Err(format!("MiniZinc status: {}", status).into()),
    };
    let parsed_json_value: serde_json::Value = serde_json::from_str(&solutions[0])?;

    let latency: Vec<i32> = parsed_json_value
        .get("end_time")
        .and_then(|v| v.as_array())
        .ok_or("end_time not found or not an array")?
        .iter()
        .map(|v| v.as_i64().unwrap_or(0) as i32)
        .collect();
    // update max latency with relaxation
    db.synthesized_information.max_latency = 2 * latency.iter().max().copied().unwrap_or(0);

    let mut result: HashMap<String, (i32, i32)> = HashMap::new();
    
    // Extract k and min_delay for each edge
    for edge in &db.app_graph.edges {
        let k_val = parsed_json_value
            .get(&format!("k_{}", edge.id))
            .and_then(|v| v.as_i64())
            .ok_or_else(|| format!("k_{} not found", edge.id))? as i32;
    
        let delay_val = parsed_json_value
            .get(&format!("min_delay_{}", edge.id))
            .and_then(|v| v.as_i64())
            .ok_or_else(|| format!("min_delay_{} not found", edge.id))? as i32;
    
        result.insert(edge.id.clone(), (k_val, delay_val));
    }

    Ok(result)
}


#[derive(Debug)]
pub struct ScheduleStruct {
    pub fire_time: HashMap<String, i32>,
    pub end_time: HashMap<String, i32>,
    pub k: HashMap<String, i32>,
    pub ob: HashMap<String, i32>,
    pub ib: HashMap<String, i32>,
    pub t0: HashMap<String, Vec<i32>>,
    pub t1: HashMap<String, Vec<i32>>,
    pub t2: HashMap<String, Vec<i32>>,
    pub t3: HashMap<String, Vec<i32>>,
}

/*
 * This function is not applicable for the start node
 * return fire_time, end_time, K, OB, IB, T0, T1, T2, T3 
 * */
fn solve_node_schedule(
    db: &DataBase,
    node_id: &str,
    fire_times: HashMap<String, i32>,
    channels: HashMap<String, (i32, i32)>,
    module_dir: String,
) -> Result<ScheduleStruct, Box<dyn std::error::Error>> {

    // helper function to replace ":" with "_" for ports 
    fn _rename(label: &str) -> String {
        label.replace(':', "_")
    }


    let mut solver = Solver::new(format!("solve_schedule_{}", node_id), module_dir.clone());
    
    // get all edges attached to the input of this node
    let edges: Vec<_> = db.app_graph.edges
        .iter()
        .find(|e| e.target_node == node_id)
        .into_iter()
        .cloned()
        .collect();

    /* formulating the model */
    solver.add(format!("include \"cumulative.mzn\";")); 
    solver.new_line();
    solver.add(format!("int: MAX_DELAY = {};", db.global_constraint.max_latency)); 
    solver.add(format!("int: EXECUTION_TIME = {};", db.app_graph.nodes.iter().find(|n| n.id == node_id).map(|n| n.execution_time).ok_or("Node is not found")?));
    solver.add(format!("var 0..MAX_DELAY: fire_time;"));
    solver.add(format!("var 0..MAX_DELAY: end_time;"));
    solver.add(format!("constraint end_time = fire_time + EXECUTION_TIME;")); 
    solver.new_line();
    
    solver.add(format!("% input and output buffers"));
    for edge in &edges {
        let input_addr_time_patterns = translate_addr_time(db, &edge.target_node, &edge.target_port, "in")?;
        let input_max_buffer_size = most_frequent_value_count(&input_addr_time_patterns); 

        let output_addr_time_patterns = translate_addr_time(db, &edge.source_node, &edge.source_port, "out")?;
        let output_max_buffer_size = most_frequent_value_count(&output_addr_time_patterns); 
        
        solver.add(format!("var 0..{}: IB_{};", 2 * input_max_buffer_size, _rename(&edge.target_port)));
        solver.add(format!("var 0..{}: OB_{};", 2 * output_max_buffer_size, _rename(&edge.source_port)));
    }
    solver.new_line();
    solver.new_line();
    solver.new_line();

    solver.add(format!("% ========= main scheduling ========="));
    let mut max_buffer_size = 0;
    for edge in &edges {
        let output_addr_time_patterns = translate_addr_time(db, &edge.source_node, &edge.source_port, "out")?; 
        let input_addr_time_patterns = translate_addr_time(db, &edge.target_node, &edge.target_port, "in")?; 
        let mut output_key_patterns: Vec<_> = output_addr_time_patterns.keys().cloned().collect(); 
        let mut input_key_patterns: Vec<_> = input_addr_time_patterns.keys().cloned().collect();
        output_key_patterns.sort();
        input_key_patterns.sort();
        
        let route_delay = db.synthesized_information.routing_paths
            .iter()
            .find(|r| r.app_edge_id == edge.id)
            .map(|r| r.delay)
            .ok_or("Cannot find an edge in the routing paths")?;
        
        if (output_key_patterns.len() as i32 != input_key_patterns.len() as i32) || 
            (output_key_patterns.len() as i32 != edge.token_size) {
            return Err(format!("Token size of input and output is not aligned (in={}, out={}, size={})", 
                    input_key_patterns.len(),
                    output_key_patterns.len(),
                    edge.token_size
            ).into());
        }

        max_buffer_size += 2 * edge.token_size;

        let (channel_width, min_delay) = channels.get(&edge.id).ok_or("Node is not found in channels")?;
        let size = format!("TOKENSIZE_{}", edge.id);
        solver.add(format!("int: {} = {};", size, edge.token_size));  
        solver.add(format!("int: K_{} = {};", edge.id, channel_width));  
        solver.add(format!("int: SOURCE_FIRE_TIME_{} = {};", edge.id, fire_times[&edge.source_node]));  
        solver.add(format!("int: WIRE_DELAY_{} = {};", edge.id, route_delay));  
        solver.add(format!("int: TOTAL_DELAY_{} = {};", edge.id, min_delay + route_delay));  
        
        solver.add(format!("array [1..{}] of int: source_addr_time_{} = {:?};", size, edge.id, output_key_patterns.iter().map(|k| output_addr_time_patterns[k]).collect::<Vec<_>>()));  
        solver.add(format!("array [1..{}] of int: target_addr_time_{} = {:?};", size, edge.id, input_key_patterns.iter().map(|k| input_addr_time_patterns[k]).collect::<Vec<_>>()));  
        solver.add(format!("array [1..{}] of var 0..MAX_DELAY: T0_{};", size, edge.id));  
        solver.add(format!("array [1..{}] of var 0..MAX_DELAY: T1_{};", size, edge.id));  
        solver.add(format!("array [1..{}] of var 0..MAX_DELAY: T2_{};", size, edge.id));  
        solver.add(format!("array [1..{}] of var 0..MAX_DELAY: T3_{};", size, edge.id));  
        solver.add(format!("array [1..{}] of var 1..MAX_DELAY: D01_{};", size, edge.id));  
        solver.add(format!("array [1..{}] of var 1..MAX_DELAY: D23_{};", size, edge.id));  
        solver.new_line();

        solver.add(format!("% scheduling constraints"));
        solver.add(format!("constraint fire_time  >= SOURCE_FIRE_TIME_{} + TOTAL_DELAY_{};", edge.id, edge.id)); 
        solver.add(format!("constraint forall(i in 1..{})(", size));
        solver.add(format!("    T0_{}[i] = SOURCE_FIRE_TIME_{} + source_addr_time_{}[i] /\\", edge.id, edge.id, edge.id));
        solver.add(format!("    T2_{}[i] = T1_{}[i] + WIRE_DELAY_{} /\\", edge.id, edge.id, edge.id));
        solver.add(format!("    T3_{}[i] = fire_time + target_addr_time_{}[i] /\\", edge.id, edge.id));
        solver.add(format!("    D01_{}[i] = T1_{}[i] - T0_{}[i] /\\", edge.id, edge.id, edge.id));
        solver.add(format!("    D23_{}[i] = T3_{}[i] - T2_{}[i]", edge.id, edge.id, edge.id));
        solver.add(format!(");"));
        solver.add(format!("% at any memory, buffers should not be overused"));
        solver.add(format!("constraint cumulative("));
        solver.add(format!("    [T0_{}[i] | i in 1..{}],", edge.id, size));
        solver.add(format!("    [D01_{}[i] | i in 1..{}],", edge.id, size));
        solver.add(format!("    [1 | i in 1..{}],", size));
        solver.add(format!("    OB_{},", _rename(&edge.source_port)));
        solver.add(format!(");"));
        solver.add(format!("constraint cumulative("));
        solver.add(format!("    [T2_{}[i] | i in 1..{}],", edge.id, size));
        solver.add(format!("    [D23_{}[i] | i in 1..{}],", edge.id, size));
        solver.add(format!("    [1 | i in 1..{}],", size));
        solver.add(format!("    IB_{},", _rename(&edge.target_port)));
        solver.add(format!(");"));
        solver.add(format!("% channel bandwidth K"));
        solver.add(format!("constraint cumulative("));
        solver.add(format!("    [T1_{}[i] | i in 1..{}],", edge.id, size));
        solver.add(format!("    [1 | i in 1..{}],", size));
        solver.add(format!("    [1 | i in 1..{}],", size));
        solver.add(format!("    K_{},", edge.id));
        solver.add(format!(");"));
        solver.new_line();
        solver.new_line();
    }
    solver.new_line();

    solver.add(format!("% ========= objective ========="));
    solver.add(format!("var {}..{}: BUFFER_SIZE;", 2 * edges.len(), max_buffer_size));
    solver.add(format!("constraint BUFFER_SIZE = sum([{}]) + sum([{}]);", 
        edges.iter().map(|e| format!("OB_{}", _rename(&e.source_port))).into_iter().collect::<Vec<_>>().join(", "),
        edges.iter().map(|e| format!("IB_{}", _rename(&e.target_port))).into_iter().collect::<Vec<_>>().join(", ")
    )); 
    solver.add(format!("solve minimize BUFFER_SIZE;")); 
    solver.new_line();
    solver.new_line();
     
    solver.add(format!("% ========= output ========="));
    let outputs = edges
        .iter()
        .map(|e| format!("OB_{}:\\(OB_{}), IB_{}:\\(IB_{}), T0_{}:\\(T0_{}), T1_{}:\\(T1_{}), T2_{}:\\(T2_{}), T3_{}:\\(T3_{})", 
                _rename(&e.source_port), _rename(&e.source_port), _rename(&e.target_port), _rename(&e.target_port), 
                e.id, e.id, e.id, e.id, e.id, e.id, e.id, e.id))
        .into_iter().collect::<Vec<_>>()
        .join(", ");

    solver.add(format!(
        "output [\"{{fire_time:\\(fire_time), end_time:\\(end_time), {}}}\"];", outputs
    ));

    /* solving the model */
    let (status, solutions) = solver.solve("--time-limit 180000 -p 8")?;
    match status.as_str() {
        "OPTIMAL_SOLUTION" | "FEASIBLE" => {}
        _ => return Err(format!("MiniZinc status: {}", status).into()),
    };
    let parsed_json_value: serde_json::Value = serde_json::from_str(&solutions[0])?;

    // formatting output 
    let mut result: ScheduleStruct = ScheduleStruct {
        fire_time: HashMap::new(),
        end_time: HashMap::new(),
        k: HashMap::new(),
        ob: HashMap::new(),
        ib: HashMap::new(),
        t0: HashMap::new(),
        t1: HashMap::new(),
        t2: HashMap::new(),
        t3: HashMap::new(),
    };

    result.fire_time.insert(String::from(node_id), parsed_json_value
        .get("fire_time")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| format!("fire_time not found"))? as i32
    ); 
    result.end_time.insert(String::from(node_id), parsed_json_value
        .get("end_time")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| format!("end_time not found"))? as i32
    );

    for edge in &edges {
        result.k.insert(edge.id.clone(), 
            channels[&edge.id].0
        );
        result.ob.insert(edge.source_port.clone(), parsed_json_value
            .get(format!("OB_{}", _rename(&edge.source_port)))
            .and_then(|v| v.as_i64())
            .ok_or_else(|| format!("OB_{} not found", _rename(&edge.source_port)))? as i32
        );
        result.ib.insert(edge.target_port.clone(), parsed_json_value
            .get(format!("IB_{}", _rename(&edge.target_port)))
            .and_then(|v| v.as_i64())
            .ok_or_else(|| format!("IB_{} not found", _rename(&edge.target_port)))? as i32
        );
        result.t0.insert(edge.id.clone(), parsed_json_value
            .get(format!("T0_{}", edge.id))
            .and_then(|v| v.as_array())
            .ok_or_else(|| format!("t0_{} not found", edge.id))?
            .iter()
            .map(|val| val.as_i64().unwrap_or(0) as i32)
            .collect::<Vec<_>>()
        );
        result.t1.insert(edge.id.clone(), parsed_json_value
            .get(format!("T1_{}", edge.id))
            .and_then(|v| v.as_array())
            .ok_or_else(|| format!("t1_{} not found", edge.id))?
            .iter()
            .map(|val| val.as_i64().unwrap_or(0) as i32)
            .collect::<Vec<_>>()
        );
        result.t2.insert(edge.id.clone(), parsed_json_value
            .get(format!("T2_{}", edge.id))
            .and_then(|v| v.as_array())
            .ok_or_else(|| format!("t2_{} not found", edge.id))?
            .iter()
            .map(|val| val.as_i64().unwrap_or(0) as i32)
            .collect::<Vec<_>>()
        );
        result.t3.insert(edge.id.clone(), parsed_json_value
            .get(format!("T3_{}", edge.id))
            .and_then(|v| v.as_array())
            .ok_or_else(|| format!("t3_{} not found", edge.id))?
            .iter()
            .map(|val| val.as_i64().unwrap_or(0) as i32)
            .collect::<Vec<_>>()
        );
    }

    Ok(result)
}


/*
 * main scheduling: first optimize each node, 
 * second check global timing, 
 * third post-optimisation
 * */
pub fn solve_scheduling(
    db: &DataBase,
    channels: HashMap<String, (i32, i32)>,
    module_dir: String,
) -> Result<ScheduleStruct, Box<dyn std::error::Error>> {

   let mut schedule: ScheduleStruct = ScheduleStruct {
        fire_time: HashMap::new(),
        end_time: HashMap::new(),
        k: HashMap::new(),
        ob: HashMap::new(),
        ib: HashMap::new(),
        t0: HashMap::new(),
        t1: HashMap::new(),
        t2: HashMap::new(),
        t3: HashMap::new(),
    };

    // initialise with the start nodes
    for node in &db.app_graph.nodes {
        if node.input_ports.len() == 0 {
            schedule.fire_time.insert(node.id.clone(), 0);
            schedule.end_time.insert(node.id.clone(), node.execution_time);
        }
    }

    // go through each node that the nodes fire to it have the fire times determined
    loop {
        let mut no_execute = true;
        for node in &db.app_graph.nodes {
            if !schedule.fire_time.keys().contains(&node.id) {
                let all_input_sources: Vec<_> = db.app_graph.edges
                    .iter() 
                    .find(|e| e.target_node == node.id)
                    .map(|e| e.source_node.clone())
                    .into_iter().collect();
                let all_input_sources_done: Vec<_> = all_input_sources
                    .iter()
                    .map(|i| schedule.fire_time.keys().contains(i))
                    .collect();

                if all_input_sources_done.iter().all(|&x| x) {
                    debug!("scheduling {}", node.id);
                    no_execute = false;

                    let node_schedule = solve_node_schedule(
                        db,
                        &node.id,
                        schedule.fire_time.clone(),
                        channels.clone(),
                        module_dir.clone()
                    )?;

                    // update schedule
                    schedule.fire_time.extend(node_schedule.fire_time.clone());
                    schedule.end_time.extend(node_schedule.end_time.clone());
                    schedule.k.extend(node_schedule.k.clone());
                    schedule.ob.extend(node_schedule.ob.clone());
                    schedule.ib.extend(node_schedule.ib.clone());
                    schedule.t0.extend(node_schedule.t0.clone());
                    schedule.t1.extend(node_schedule.t1.clone());
                    schedule.t2.extend(node_schedule.t2.clone());
                    schedule.t3.extend(node_schedule.t3.clone());
                }
            }
        }
        
        let all_nodes_done: Vec<_> = db.app_graph.nodes
            .iter() 
            .map(|n| schedule.fire_time.keys().contains(&n.id))
            .collect();
        if all_nodes_done.iter().all(|&x| x) {
            break;
        }

        // report error if there's no scheduling in one iteration
        if no_execute {
             return Err(format!("Cannot schedule a node due to a graph conflict").into());
        }
    }

    Ok(schedule)
}
