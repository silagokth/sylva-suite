use sv_lib::model::{DataBase, AddressPatterns};
use sv_lib::solver::{Solver};
use log::{info, error, debug};
use std::collections::{HashMap, HashSet};

mod banking;

fn address_pattern_to_hashmap(list: &Vec<AddressPatterns>) -> HashMap<i32, (i32, i32)> {
    list.iter().map(|e| (e.address, (e.channel, e.time))).collect()
}


fn translate_addr_pattern(
    db: &DataBase,
    node_name: &str,
    port_name: &str,
    dir: &str,
) -> Result<HashMap<i32, (i32, i32)>, Box<dyn std::error::Error>> {
    let binding = db.synthesized_information.alimp_bindings
        .iter()
        .find(|n| n.app_node_id == node_name)
        .ok_or("Cannot find the address time patterns in Alimp")?;

    let patterns = match dir {
        "in" => address_pattern_to_hashmap(&binding.alimp_instance.input_addr_time_patterns),
        _ => address_pattern_to_hashmap(&binding.alimp_instance.output_addr_time_patterns),
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

    let mut addr_patterns = HashMap::new();
    let mut start_address = 0;
    
    for port in ports {
        let range_address = port.token_size;
        if port.id == port_name {
            for &addr in patterns.keys() {
                if (addr - start_address >= 0) && (addr - start_address < range_address) {
                    if let Some((c ,t)) = patterns.get(&addr) {
                        addr_patterns.insert(addr - start_address, (*c, *t));
                    }
                }
            }
            break;
        } 
        start_address += range_address;
    }

    Ok(addr_patterns)
}


fn pattern_dependencies(
    db: &DataBase,
    src_node_name: &str,
    src_port_name: &str,
    dst_node_name: &str, 
    dst_port_name: &str, 
) -> Result<HashMap<i32, Vec<i32>>, Box<dyn std::error::Error>> {
    let mut dep: HashMap<i32, HashSet<i32>> = HashMap::new();

    let output_patterns = translate_addr_pattern(db, src_node_name, src_port_name, "out")?;
    let input_patterns = translate_addr_pattern(db, dst_node_name, dst_port_name, "in")?;

    for (addr, (dst_channel, _)) in input_patterns.iter() {
        if let Some(src_channel) = output_patterns
            .iter()
            .find(|(key, _)| *key == addr)
            .map(|(_, (channel,_))| channel)
        {
            dep.entry(*dst_channel)
                .or_insert_with(HashSet::new)
                .insert(*src_channel);
        };
    }

    // Convert HashSet<i32> → Vec<i32>
    let dep_vec: HashMap<i32, Vec<i32>> = dep
        .into_iter()
        .map(|(k, v)| (k, v.into_iter().collect()))
        .collect();

    Ok(dep_vec)
}

fn group_dependencies(
    dep: &HashMap<i32, Vec<i32>>,
) -> Result<(Vec<Vec<i32>>, Vec<Vec<i32>>), Box<dyn std::error::Error>> {
    let mut dst_deps: Vec<Vec<i32>> = Vec::new();
    let mut src_deps: Vec<Vec<i32>> = Vec::new();

    for (dst, src_list) in dep.iter() {
        // Check if src_list overlaps with an existing group
        if let Some((idx, _)) = src_deps
            .iter()
            .enumerate()
            .find(|(_, group)| group.iter().any(|s| src_list.contains(s)))
        {
            // merge into existing group
            src_deps[idx].extend(src_list.clone());
            dst_deps[idx].push(*dst);
        } else {
            // create a new group
            src_deps.push(src_list.clone());
            dst_deps.push(vec![*dst]);
        }
        
        for group in src_deps.iter_mut() {
            group.sort();
            group.dedup();
        }
        for group in dst_deps.iter_mut() {
            group.sort();
            group.dedup();
        }
    }
    
    Ok((dst_deps, src_deps))
}



fn memory_synthesis(
    db: &mut DataBase, 
    module_dir: String,
) -> Result<(), Box<dyn std::error::Error>> {

    for edge in &db.app_graph.edges {
        let dst_dependencies = pattern_dependencies(
            db, 
            &edge.source_node, 
            &edge.source_port, 
            &edge.target_node, 
            &edge.target_port
        )?;
        let (dst_groups, src_groups) = group_dependencies(&dst_dependencies)?;
       
        let output_patterns = translate_addr_pattern(db, &edge.source_node, &edge.source_port, "out")?;
        let input_patterns = translate_addr_pattern(db, &edge.target_node, &edge.target_port, "in")?;
        
        for ((i, dst_list), src_list) in dst_groups.iter().enumerate().zip(src_groups.iter()) {
            let mut working_output_patterns: Vec<_> = output_patterns
                .iter()
                .filter(|(_, (c, _))| src_list.contains(c))
                .map(|(a, (c, t))| (*a, *c, *t)) // deref so we own i32, not refs
                .collect();

            let mut working_input_patterns: Vec<_> = input_patterns
                .iter()
                .filter(|(_, (c, _))| dst_list.contains(c))
                .map(|(a, (c, t))| (*a, *c, *t))
                .collect();

            // Sort by address
            working_output_patterns.sort_by_key(|(addr, _, _)| *addr);
            working_input_patterns.sort_by_key(|(addr, _, _)| *addr);

            // verifying working patterns 
            let out_addrs: Vec<_> = working_output_patterns.iter().map(|(addr, _, _)| *addr).collect();
            let in_addrs: Vec<_> = working_input_patterns.iter().map(|(addr, _, _)| *addr).collect();
            if out_addrs != in_addrs {
                return Err(format!("cannot extract working address channel patterns").into());
            }

            banking::optimise_memory(
                i as i32,
                &db,
                &edge.id,
                &working_output_patterns,
                &working_input_patterns,
                module_dir.clone(),
            );
            //...  
        }
/*
        // constraints verification 
        if sum(output_buffer_size) > total_output_buffer {
            return Err(format!("find an overestimated solution for OB at edge {}", &edge.id).into());
        } 
        
        if sum(input_buffer_size) > total_input_buffer {
            return Err(format!("find an overestimated solution for IB at edge {}", &edge.id).into());
        }
        
        if sum(communication_channel_size) > total_communication_channel {
            return Err(format!("find an overestimated solution for K at edge {}", &edge.id).into());
        }       
*/
        // update the edge solution 
        //....
    }

    Ok(())
}

#[allow(unused_variables)]
pub fn run(
    db: &mut DataBase, 
    dir: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Start: memory synthesis");
    let module_dir = format!("{}/memsyn", dir);
    match std::fs::create_dir_all(&module_dir) {
        Ok(_) => (),
        Err(e) => {
            error!("Failed to create {} with {}", module_dir, e);
            return Err(Box::new(e));
        },
    };

    info!("Stage 1: "); 
    memory_synthesis(db, module_dir.clone())?;

    info!("Finish: memory synthesis");
    Ok(())
}


















