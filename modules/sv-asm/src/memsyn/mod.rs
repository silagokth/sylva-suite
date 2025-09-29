use sv_lib::model::{DataBase, AddressPatterns};
use log::{info, error, debug};
use std::collections::{HashMap, HashSet};
use ndarray::Array2;

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

    let previous_db = db.clone()


    // delete all information about channel_width, 
    // address translation, memory synthesis, and transporter tables
    db.synthesized_information.channel_width = HashMap::new(); // never use again
    db.synthesized_information.address_translation = vec![];
    db.synthesized_information.memory_synthesis = vec![];
    db.synthesized_information.transporter_tables = vec![];


    for edge in &previous_db.app_graph.edges {
        debug!("memory optimising on edge {}", edge.id);
        // preparing some constraints     
        let src_fire_time = previous_db.synthesized_information.node_fire_times
            .get(&edge.source_node)
            .ok_or(format!("cannot find fire time of {} node", &edge.source_node))?;
        
        let dst_fire_time = previous_db.synthesized_information.node_fire_times
            .get(&edge.target_node)
            .ok_or(format!("cannot find fire time of {} node", &edge.target_node))?;
        
        let routing_delay = previous_db.synthesized_information.routing_paths
            .iter()
            .find(|r| r.app_edge_id == edge.id)
            .map(|r| r.delay)
            .ok_or("Cannot find an edge in the routing paths")?;

        let total_output_buffer = previous_db.synthesized_information.memory_synthesis
            .iter()
            .find(|m| m.app_node_id == edge.source_node && m.port_id == edge.source_port)
            .map(|m| m.memory_structure[0].memory_size)
            .ok_or(format!("cannot find memory synthesis of {} node at {} port", &edge.source_node, &edge.source_port))?;

        let total_input_buffer = previous_db.synthesized_information.memory_synthesis
            .iter()
            .find(|m| m.app_node_id == edge.target_node && m.port_id == edge.target_port)
            .map(|m| m.memory_structure[0].memory_size)
            .ok_or(format!("cannot find input buffer size of {} node", &edge.target_node))?;
        
        let total_communication_channel = previous_db.synthesized_information.channel_width
            .get(&format!("transporter_{}", edge.id))
            .ok_or(format!("cannot find channel width of {} edge", &edge.id))?;
        
        let edge_memory_constraints = banking::MemoryConstraint {
            edge_id: edge.id.clone(),
            src_fire_time: *src_fire_time,
            dst_fire_time: *dst_fire_time,
            routing_delay: routing_delay,
            output_buffer_size: total_output_buffer,
            input_buffer_size: total_input_buffer,
            channel_width_size: *total_communication_channel,
        };

        let dst_dependencies = pattern_dependencies(
            &previous_db, 
            &edge.source_node, 
            &edge.source_port, 
            &edge.target_node, 
            &edge.target_port
        )?;
        let (dst_groups, src_groups) = group_dependencies(&dst_dependencies)?;
       
        let output_patterns = translate_addr_pattern(&previous_db, &edge.source_node, &edge.source_port, "out")?;
        let input_patterns = translate_addr_pattern(&previous_db, &edge.target_node, &edge.target_port, "in")?;
        
        let mut numbering = 0;
        let mut memory_collections: Vec<banking::MemoryBankInfo> = vec![];

        for (dst_list, src_list) in dst_groups.iter().zip(src_groups.iter()) {
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

            // solve the constraint programming problem 
            memory_collections.push(
                banking::optimise_memory(
                    &edge_memory_constraints,
                    &mut numbering,
                    &working_output_patterns,
                    &working_input_patterns,
                    module_dir.clone(),
                )?
            );
        }
    
        // update each edge's solution
        debug!("found all solutions for edge {}, updating the synthesized information", edge.id);
      
        // ---------------------------------------------------------------
        // memory synthesis
        let mut source_memory = MemorySynthesis {
            app_node_id: edge.source_node,
            port_id: edge.source_port,
            memory_direction: "out",
            memory_structure: vec![],
        };
        let mut target_memory = MemorySynthesis {
            app_node_id: edge.target_node,
            port_id: edge.target_port,
            memory_direction: "in",
            memory_structure: vec![],
        };

        // getting geometry information 
        let mut source_x, source_y = previous_db.synthesized_information.placements
            .iter()
            .find(|p| p.app_node_id == edge.source_node)
            .map(|p| (p.x, p.y - 1)) // point to vertical OB level 
            .collect();

        // need to get the height of the alimp to calculate the positions of input buffers
        let target_height = previous_db.synthesized_information.alimp_bindings
            .iter()
            .find(|b| b.app_node_id == edge.target_node)
            .map(|b| b.alimp.instance.height)
            .collect();

        let mut target_x, target_y = previous_db.synthesized_information.placements
            .iter()
            .find(|p| p.app_node_id == edge.target_node)
            .map(|p| (p.x, p.y + target_height)) // point to vertical IB level
            .collect();

        let mut most_left_x_position = 0;
        let mut most_right_x_position = 0;
    
        // iterate to collect all banking information into the memory synthesis data
        for ((i, dst_list), src_list)) in dst_groups.iter().enumerate().zip(src_groups.iter()) {
            
            // since sharing channels is not supported, we can just do this
            let memory_info = memory_collections[i];
            
            for bank_index in 0..memory_info.ob_memory_types.len() {
                // output buffer 
                most_left_x_position = src_list.iter().min();
                most_right_x_position = src_list.iter().min();
                           
                let output_communication_channels = memory_info.output_ob_channels
                    .index_axis(Axis(0), bank_id)
                    .iter()
                    .enumerate()
                    .filter(|_, i| i == 1)
                    .map(|idx, _| idx + most_left_x_position)
                    .collect();
                    
                source_memory.memory_structure.push(MemoryStructure {
                    memory_type: memory_info.ob_memory_types[bank_index],
                    memory_size: memory_info.ob_size[bank_index]
                    input_channels: src_list.clone(), 
                    output_chennels: output_communication_channels.clone(), 
                    placement: MemoryPlacement {
                        x: source_x + most_left_x_position, 
                        y: source_y,
                        width: most_right_x_position - most_left_x_position + 1,
                        height: 1, // fixed to 1
                    }
                });
                
                // input buffer 
                most_left_x_position = dst_list.iter().min();
                most_right_x_position = dst_list.iter().min();
            
                let input_communication_channels = memory_info.input_ib_channels
                    .index_axis(Axis(0), bank_id)
                    .iter()
                    .enumerate()
                    .filter(|(_, i)| i == 1)
                    .map(|idx, _| idx + most_left_x_position)
                    .collect();
                    
                target_memory.memory_structure.push(MemoryStructure {
                    memory_type: memory_info.ib_memory_types[bank_index],
                    memory_size: memory_info.ib_size[bank_index]
                    input_channels: dst_list.clone(), 
                    output_chennels: input_communication_channels.clone(), 
                    placement: MemoryPlacement {
                        x: target_x + most_left_x_position, 
                        y: target_y,
                        width: most_right_x_position - most_left_x_position + 1,
                        height: 1, // fixed to 1
                    }
                });
            }
        }


        // update the memory synthesis
        db.synthesis_information.memory_synthesis.push(
            source_memory
        );

        db.synthesis_information.memory_synthesis.push(
            target_memory
        );

        // ---------------------------------------------------------------
        // Address translation
        let mut bank_index = 0;
        for ((i, dst_list), src_list)) in dst_groups.iter().enumerate().zip(src_groups.iter()) {
            let mut output_patterns: Vec<_> = output_patterns
                .iter()
                .filter(|(_, (c, _))| src_list.contains(c))
                .map(|(a, (c, t))| (*a, *c, *t)) 
                .collect();

            output_patterns.sort_by_key(|(addr, _, _)| *addr);
           
            let memory_info = memory_collections[i];

            let flattened_t0: Vec<_> = memory_info.t0
                .axis_iter(Axis(1))
                .map(|col| {
                    col.iter()
                        .copied()
                        .find(|&x| x > 0)
                        .unwrap_or(0)
                })
                .collect();
            
            let flattened_t1: Vec<_> = memory_info.t1
                .axis_iter(Axis(1))
                .map(|col| {
                    col.iter()
                        .copied()
                        .find(|&x| x > 0)
                        .unwrap_or(0)
                })
                .collect();
            
            let number_of_bank = memory_info.ob_memory_types.len(); // should be 1
            for j in 0..number_of_bank {
                
                let channels = memory_info.input_ob_channels
                    .index_axis(Axis(0), j)
                    .iter()
                    .enumerate()
                    .filter(|(_, i)| i == 1)
                    .map(|idx, _| src_list[idx]);
                    .collect();
 
                let valid_indices: Vec<_> = output_patterns
                    .iter()
                    .enumerate()
                    .filter(|(_, (_, c, _))| channels.contains(c))
                    .map(|(idx, _)| idx)
                    .collect();
               
                let scheduled_t0 = flattened_t0
                    .iter()
                    .enumerate()
                    .filter(|addr, _| valid_indices.contains(addr))
                    .map(|_, t| t)
                    .collect();
               
                let scheduled_t1 = flattened_t1
                    .iter()
                    .enumerate()
                    .filter(|addr, _| valid_indices.contains(addr))
                    .map(|_, t| t)
                    .collect();
               
                let mut assigned_address: Vec<i32> = equitable_address_assignment(
                    scheduled_t0,
                    scheduled_t1,
                    memory_info.ob_size[j],
                )?; 
 







                bank_index += 1;
            } 


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

    memory_synthesis(db, module_dir.clone())?;

    info!("Finish: memory synthesis");
    Ok(())
}


















