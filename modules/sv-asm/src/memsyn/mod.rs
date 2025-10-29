use sv_lib::model::{DataBase, MemorySynthesis, MemoryStructure, MemoryPlacement,
                    TransporterTable, TransportTableEntry, AddressPatterns, AddressTranslation};
use log::{info, error, debug};
use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use itertools::Itertools;
use ndarray::{Axis};

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
    let mut dst_deps: Vec<HashSet<i32>> = Vec::new();
    let mut src_deps: Vec<HashSet<i32>> = Vec::new();

    for (dst, src_list) in dep.iter() {
        // find all groups that overlap with the new src_list
        let indices: Vec<usize> = src_deps
            .iter()
            .enumerate()
            .filter(|(_, group)| group.iter().any(|s| src_list.contains(s)))
            .map(|(idx, _)| idx)
            .collect();

        if indices.is_empty() {
            // create a new group
            let mut new_src = HashSet::new();
            new_src.extend(src_list.iter().copied());
            let mut new_dst = HashSet::new();
            new_dst.insert(*dst);

            src_deps.push(new_src);
            dst_deps.push(new_dst);
        } else {
            // merge all overlapping groups into one
            let mut merged_src = HashSet::new();
            let mut merged_dst = HashSet::new();

            // important to iterate in reverse order
            for &idx in indices.iter().rev() {
                merged_src.extend(src_deps.remove(idx));
                merged_dst.extend(dst_deps.remove(idx));
            }
            
            // add the new sources and destination
            merged_src.extend(src_list.iter().copied());
            merged_dst.insert(*dst);

            src_deps.push(merged_src);
            dst_deps.push(merged_dst);
        }
    }
    
    // Convert HashSet<i32> → Vec<i32> for the result
    let dst_vecs: Vec<Vec<i32>> = dst_deps.into_iter().map(|s| s.into_iter().collect()).collect();
    let src_vecs: Vec<Vec<i32>> = src_deps.into_iter().map(|s| s.into_iter().collect()).collect();
 
    Ok((dst_vecs, src_vecs))
}



fn equitable_address_assignment(
    t0: &Vec<i32>,
    t1: &Vec<i32>,
    capacity: &i32,
) -> Result<Vec<i32>, Box<dyn std::error::Error>> {

    // build a conflict graph for each chunk
    let mut conflict_graph: HashMap<i32, Vec<i32>> = HashMap::new();
    for i in 0..t0.len() {
        for j in 0..t0.len() {
            if i != j {
                if t1[j] - t0[i] > 0 && t1[i] - t0[j] > 0 {
                    conflict_graph.entry(i as i32).or_default().push(j as i32);
                }
            }
        }
    }

    let sorted_index: Vec<usize> = (0..t0.len())
        .collect::<Vec<_>>()
        .into_iter()
        .sorted_by_key(|&i| t0[i])
        .collect();

    let mut assigned_address = vec![-1; t0.len()];

    for &i in &sorted_index {
        // frequently used addresses 
        let mut address_count = vec![0; *capacity as usize];
        for &addr in &assigned_address {
            if addr != -1 {
                address_count[addr as usize] += 1;
            }
        }

        // sort the addresses by the least used ones
        let addresses: Vec<_> = (0..*capacity).sorted_by_key(|&idx| address_count[idx as usize]).collect();

        // assign the address to the first one available
        for candidate in addresses {
            // get all chunks assigned to this candidate address
            let chunks: Vec<_> = assigned_address
                .iter()
                .enumerate()
                .filter_map(|(idx, addr)| if *addr == candidate { Some(idx) } else { None })
                .collect();

            // check for conflicts between i and chunks already assigned to candidate
            if chunks.iter().any(|&idx| {
                conflict_graph
                    .get(&(i as i32))
                    .map_or(false, |set| set.contains(&(idx as i32)))
            }) {
                continue; // try next candidate
            }

            // no conflict, assign and break
            assigned_address[i] = candidate as i32;
            break;
        }
    }

    if assigned_address.iter().any(|&i| i < 0) {
        return Err(format!("Fails to assign equitable addresses").into());
    }

    Ok(assigned_address)
}




fn memory_synthesis(
    db: &mut DataBase, 
    module_dir: String,
) -> Result<(), Box<dyn std::error::Error>> {

    let previous_db = db.clone();

    // delete all information about channel_width, 
    // address translation, memory synthesis, and transporter tables
    db.synthesized_information.channel_width = HashMap::new(); // never use again
    db.synthesized_information.address_translations = vec![];
    db.synthesized_information.memory_synthesis = vec![];
    db.synthesized_information.transporter_tables = vec![];
    db.synthesized_information.node_fire_times.retain(|k, _| !k.starts_with("transporter_"));
    
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
            output_buffer_size: total_output_buffer as i32,
            input_buffer_size: total_input_buffer as i32,
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
                error!("displaying out_addrs: {:?}", out_addrs);
                error!("displaying in_addrs: {:?}", in_addrs);
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
        debug!("found all solutions for edge {}, updating synthesized information", edge.id);
      
        // ---------------------------------------------------------------
        // memory synthesis
        let mut source_memory = MemorySynthesis {
            app_node_id: edge.source_node.clone(),
            port_id: edge.source_port.clone(),
            memory_direction: "out".to_string(),
            memory_structure: vec![],
        };
        let mut target_memory = MemorySynthesis {
            app_node_id: edge.target_node.clone(),
            port_id: edge.target_port.clone(),
            memory_direction: "in".to_string(),
            memory_structure: vec![],
        };

        // getting geometry information 
        // need to get the height of the alimp to calculate the positions of OB and transporter
        let source_height = previous_db.synthesized_information.alimp_bindings
            .iter()
            .find(|b| b.app_node_id == edge.source_node)
            .map(|b| b.alimp_instance.height)
            .ok_or(format!("cannot find alimp binding of {}", &edge.target_node))?;

        let (source_x, source_y) = previous_db.synthesized_information.placements
            .iter()
            .find(|p| p.app_node_id == edge.source_node)
            .map(|p| (p.x, p.y + source_height * db.technology_constraint.grid_per_drra_height)) // point to vertical OB level 
            .ok_or(format!("cannot find placement of {}", &edge.source_node))?;

        let (target_x, target_y) = previous_db.synthesized_information.placements
            .iter()
            .find(|p| p.app_node_id == edge.target_node)
            .map(|p| (p.x, p.y - db.technology_constraint.grid_per_drra_height)) // point to vertical IB level
            .ok_or(format!("cannot find placement of {}", &edge.target_node))?;

        // iterate to collect all banking information into the memory synthesis data
        for ((i, dst_list), src_list) in dst_groups.iter().enumerate().zip(src_groups.iter()) {
            // since sharing channels is not supported, we can just do this
            let memory_info = &memory_collections[i];
            
            for bank_index in 0..memory_info.ob_memory_types.len() {
                // output buffer 
                let source_most_left_x_position = *src_list.iter().min().unwrap_or(&0) as usize;
                let source_most_right_x_position = *src_list.iter().max().unwrap_or(&0) as usize;
                
                let output_communication_channels: Vec<_> = memory_info.output_ob_channels
                    .index_axis(Axis(0), bank_index)
                    .iter()
                    .enumerate()
                    .filter(|&(_, &val)| val == 1)
                    .map(|(idx, _)| (idx + source_most_left_x_position) as i32)
                    .collect();
                
                // input buffer            
                let target_most_left_x_position = *dst_list.iter().min().unwrap_or(&0) as usize;
                let target_most_right_x_position = *dst_list.iter().max().unwrap_or(&0) as usize;
                               
                let input_communication_channels: Vec<_> = memory_info.input_ib_channels
                    .index_axis(Axis(0), bank_index)
                    .iter()
                    .enumerate()
                    .filter(|&(_, &val)| val == 1)
                    .map(|(idx, _)| (idx + target_most_left_x_position) as i32)
                    .collect();    
                
                assert_eq!(output_communication_channels.len(), input_communication_channels.len());

                source_memory.memory_structure.push(MemoryStructure {
                    memory_type: memory_info.ob_memory_types[bank_index].clone(),
                    memory_size: memory_info.ob_size[bank_index] as u32,
                    input_channels: src_list.iter().map(|&x| x as u32).collect(), 
                    output_channels: output_communication_channels.iter().map(|&x| x as u32).collect(), 
                    corresponding_channels: input_communication_channels.iter().map(|&x| x as u32).collect(), 
                    placement: MemoryPlacement {
                        x: source_x + (source_most_left_x_position as i32) * db.technology_constraint.grid_per_drra_width, 
                        y: source_y,
                        width: (source_most_right_x_position - source_most_left_x_position) as i32 + 1,
                        height: 1, // fixed to 1
                    },
                });
                
                target_memory.memory_structure.push(MemoryStructure {
                    memory_type: memory_info.ib_memory_types[bank_index].clone(),
                    memory_size: memory_info.ib_size[bank_index] as u32,
                    input_channels: dst_list.iter().map(|&x| x as u32).collect(), 
                    output_channels: input_communication_channels.iter().map(|&x| x as u32).collect(), 
                    corresponding_channels: vec![],
                    placement: MemoryPlacement {
                        x: target_x + (target_most_left_x_position as i32) * db.technology_constraint.grid_per_drra_width, 
                        y: target_y,
                        width: (target_most_right_x_position - target_most_left_x_position) as i32 + 1,
                        height: 1, // fixed to 1
                    },
                });
            }
        }


        // update the memory synthesis
        db.synthesized_information.memory_synthesis.push(
            source_memory.clone()
        );

        db.synthesized_information.memory_synthesis.push(
            target_memory.clone()
        );

        // ---------------------------------------------------------------
        // Address translation
        let mut bank_index = 0;
                
        let mut source_assignment = AddressTranslation {
            app_node_id: edge.source_node.clone(),
            port_id: edge.source_port.clone(),
            address_assignment: HashMap::new(),
        };
       
        let mut target_assignment = AddressTranslation {
            app_node_id: edge.target_node.clone(),
            port_id: edge.target_port.clone(),
            address_assignment: HashMap::new(),
        };

        // To keep the information about transporter tables 
        // This is a vector of transporter entries grouped per transporter: Vec<Vec<(t1, src_addr, tgt_addr)>>
        let mut source_target_pairs: Vec<Vec<(i32, i32, i32)>> = vec![];

        for ((i, dst_list), src_list) in dst_groups.iter().enumerate().zip(src_groups.iter()) {
            let mut working_output_patterns: Vec<_> = output_patterns
                .iter()
                .filter(|(_, (c, _))| src_list.contains(c))
                .map(|(a, (c, t))| (*a, *c, *t)) 
                .collect();
            
            let mut working_input_patterns: Vec<_> = input_patterns
                .iter()
                .filter(|(_, (c, _))| dst_list.contains(c))
                .map(|(a, (c, t))| (*a, *c, *t)) 
                .collect();
            
            working_output_patterns.sort_by_key(|(addr, _, _)| *addr);
            working_input_patterns.sort_by_key(|(addr, _, _)| *addr);
           
            let memory_info = &memory_collections[i];

            let flattened_t0: Vec<_> = memory_info.t0
                .axis_iter(Axis(1))
                .map(|col| col.iter().copied().find(|&x| x > 0).unwrap_or(0))
                .collect();
            
            let flattened_t1: Vec<_> = memory_info.t1
                .axis_iter(Axis(1))
                .map(|col| col.iter().copied().find(|&x| x > 0).unwrap_or(0))
                .collect();
 
            let flattened_t2: Vec<_> = memory_info.t2
                .axis_iter(Axis(1))
                .map(|col| col.iter().copied().find(|&x| x > 0).unwrap_or(0))
                .collect();
            
            let flattened_t3: Vec<_> = memory_info.t3
                .axis_iter(Axis(1))
                .map(|col| col.iter().copied().find(|&x| x > 0).unwrap_or(0))
                .collect(); 

            let number_of_bank = memory_info.ob_memory_types.len(); // should be 1
            for j in 0..number_of_bank {
                // Address assignment for source node
                let source_channels: Vec<_> = memory_info.input_ob_channels
                    .index_axis(Axis(0), j)
                    .iter()
                    .enumerate()
                    .filter(|&(_, &v)| v == 1)
                    .map(|(idx, _)| src_list[idx])
                    .collect();
 
                let selected_indices: Vec<_> = working_output_patterns
                    .iter()
                    .enumerate()
                    .filter(|(_, (_, c, _))| source_channels.contains(c))
                    .map(|(idx, _)| idx)
                    .collect();
               
                let scheduled_t0 = flattened_t0
                    .iter()
                    .enumerate()
                    .filter(|(addr, _)| selected_indices.contains(addr))
                    .map(|(_, &t)| t)
                    .collect();
               
                let scheduled_t1 = flattened_t1
                    .iter()
                    .enumerate()
                    .filter(|(addr, _)| selected_indices.contains(addr))
                    .map(|(_, &t)| t)
                    .collect();
               
                let source_assigned_address: Vec<i32> = equitable_address_assignment(
                    &scheduled_t0,
                    &scheduled_t1,
                    &memory_info.ob_size[j],
                )?; 
 
                assert_eq!(source_assigned_address.len(), selected_indices.len());

                for k in 0..source_assigned_address.len() {
                    let (virtual_address, channel, _) = working_output_patterns[selected_indices[k]];
                    
                    match source_assignment.address_assignment.entry(virtual_address) {
                        Entry::Vacant(entry) => {
                            entry.insert((channel, bank_index, source_assigned_address[k]));
                        }
                        Entry::Occupied(_) => {
                            return Err(format!("virtual address is already occupied at source node {} ({}).", &edge.source_node, &edge.source_port).into());
                        }
                    }
                }

                // Address assignment for target node
                let scheduled_t2 = flattened_t2
                    .iter()
                    .enumerate()
                    .filter(|(addr, _)| selected_indices.contains(addr))
                    .map(|(_, &t)| t)
                    .collect();
               
                let scheduled_t3 = flattened_t3
                    .iter()
                    .enumerate()
                    .filter(|(addr, _)| selected_indices.contains(addr))
                    .map(|(_, &t)| t)
                    .collect();
               
                let target_assigned_address: Vec<i32> = equitable_address_assignment(
                    &scheduled_t2,
                    &scheduled_t3,
                    &memory_info.ib_size[j],
                )?; 
 
                assert_eq!(target_assigned_address.len(), selected_indices.len());

                for k in 0..target_assigned_address.len() {
                    let (virtual_address, channel, _) = working_input_patterns[selected_indices[k]];
                    
                    match target_assignment.address_assignment.entry(virtual_address) {
                        Entry::Vacant(entry) => {
                            entry.insert((channel, bank_index, target_assigned_address[k]));
                        }
                        Entry::Occupied(_) => {
                            return Err(format!("virtual address is already occupied at target node {} ({}).", &edge.target_node, &edge.target_port).into());
                        }
                    }
                }

                // keep some information for assigning transporter tables
                assert_eq!(source_assigned_address.len(), target_assigned_address.len());
                assert_eq!(source_assigned_address.len(), scheduled_t1.len());
                
                let grouped_pairs: Vec<(i32, i32, i32)> = scheduled_t1
                    .iter()
                    .zip(source_assigned_address.iter())
                    .zip(target_assigned_address.iter())
                    .map(|((t1, src), tgt)| (*t1, *src, *tgt))
                    .collect();

                // split the addresses according to the number of communication channels inuse
                let set_indices: Vec<Vec<_>> = memory_info.t1
                    .axis_iter(Axis(0))
                    .map(|row| {
                        row.iter()
                            .enumerate()
                            .filter(|&(_, &x)| x > 0)
                            .map(|(idx, _)| idx)
                            .collect::<Vec<_>>()
                    })
                    .collect();

                assert_eq!(grouped_pairs.len(), set_indices.iter().map(|x| x.len()).sum::<usize>());
            
                for indices in set_indices.iter() {
                    let collected: Vec<_> = grouped_pairs
                        .iter()
                        .enumerate()
                        .filter(|(idx, _)| indices.contains(idx))
                        .map(|(_, content)| content.clone())
                        .collect();
                
                    source_target_pairs.push(collected);
                }

                bank_index += 1;
            } 
        }
            
        // verify that all addresses have been assigned
        assert_eq!(
            source_assignment.address_assignment.len(),
            output_patterns.len()
        );

        assert_eq!(
            target_assignment.address_assignment.len(),
            input_patterns.len()
        );

        // update address translation
        db.synthesized_information.address_translations.push(
            source_assignment
        );

        db.synthesized_information.address_translations.push(
            target_assignment
        );


        // ---------------------------------------------------------------
        // Transporter tables 
        
        // verify that the table exists for each transporter
        let number_of_transporters: usize = source_memory.memory_structure
            .iter()
            .map(|mem| mem.output_channels.len())
            .sum();

        if source_target_pairs.len() != number_of_transporters {
            error!("number_of_transporters = {} from {:?}", number_of_transporters, source_memory.memory_structure);
            error!("source_target_pairs = {:?}", source_target_pairs);
            return Err(format!("fail to verify that a table exists for every transporter").into());
        }

        let mut transporter_index = 0;

        for memory in &source_memory.memory_structure {
            for i in 0..memory.output_channels.len() {
                if source_target_pairs.is_empty() {
                    return Err(format!("Fail to assign the transporter tables at source node {}", &edge.source_node).into());
                }
                let transporter_id = format!("transporter_{}_{}", &edge.id, transporter_index);
                
                let mut assign = source_target_pairs.remove(0);
                
                // compute fire_time and end_time
                let fire_time: i32 = assign.iter().map(|(t, _, _)| *t).min().unwrap_or(0);
                let end_time: i32 = assign.iter().map(|(t, _, _)| *t).max().unwrap_or(0) + 1;
                
                // update time table (make times relative) 
                for (time, _, _) in assign.iter_mut() {
                    *time -= fire_time; 
                }

                let entries: Vec<TransportTableEntry> = assign
                    .iter()
                    .map(|(t, src_addr, tgt_addr)| TransportTableEntry {
                        relative_time: *t,
                        source_address: *src_addr,
                        target_address: *tgt_addr,
                    })
                    .collect();

                // update synthesized information 
                db.synthesized_information.transporter_tables.push(
                    TransporterTable {
                        transporter_id: transporter_id.clone(),
                        fire_time: fire_time,
                        end_time: end_time,
                        entries: entries,
                        from: memory.output_channels[i],
                        to: memory.corresponding_channels[i],
                        placement: MemoryPlacement {
                            x: memory.placement.x + (i as i32) * db.technology_constraint.grid_per_drra_width,
                            y: memory.placement.y + db.technology_constraint.grid_per_drra_height,
                            width: 1,
                            height: 1,
                        }
                    }
                );

                db.synthesized_information.node_fire_times.insert(
                    transporter_id.clone(),
                    fire_time
                );
                
                transporter_index += 1;
            }
        }


        // ---------------------------------------------------------------
        // end of updating synthesized information at this edge

        // sanity checks for constraints
        if number_of_transporters as i32 > *total_communication_channel {
            return Err(format!("find an overestimated solution for channel width at edge {}", &edge.id).into());
        }       

        if source_memory.memory_structure.iter().map(|m| m.memory_size).sum::<u32>() >
            2 * total_output_buffer {
            return Err(format!("find an overestimated solution for OB at edge {}", &edge.id).into());
        } 

        if target_memory.memory_structure.iter().map(|m| m.memory_size).sum::<u32>() >
            2 * total_input_buffer {
            return Err(format!("find an overestimated solution for IB at edge {}", &edge.id).into());
        } 
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


















