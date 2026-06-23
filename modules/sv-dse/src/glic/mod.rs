use sv_lib::model::{DataBase, AppNodePort, AddressTranslation, 
                    TransporterTable, TransportTableEntry, AddressPatterns,
                    MemorySynthesis, MemoryStructure, MemoryPlacement};
use log::{info, debug, error};
use itertools::Itertools;
use std::collections::{HashMap, HashSet, BTreeMap};

mod optimiser;


fn select_alimp(
    db: &mut DataBase,
) -> Result<(), Box<dyn std::error::Error>> {
    for node in &mut db.app_graph.nodes {
        node.repetition = 1;

        let alimp_binding = db.synthesized_information.alimp_bindings
            .iter()
            .find(|binding| binding.app_node_id == node.id)
            .ok_or_else(|| format!("missing ALIMP binding for node {}", node.id))?;

        node.execution_time = alimp_binding.alimp_instance.latency;
    }
    Ok(())
}


fn verify_alimp_patterns(
    db: &DataBase,
) -> Result<(bool, String), Box<dyn std::error::Error>> {
    let mut is_verified = true;
    let mut node_id = String::new();

    fn all_address_unique(patterns: &Vec<AddressPatterns>) -> bool {
        let mut seen = HashSet::new();
        patterns.iter().all(|p| seen.insert(p.address))
    }

    fn channels_in_bound(patterns: &Vec<AddressPatterns>, limit: i32) -> bool {
        // set: (time, channel) 
        let mut seen: HashSet<(i32, i32)> = HashSet::new();
        
        for p in patterns {
            if p.channel >= limit {
                return false
            } 
        }

        patterns.iter().all(|p| seen.insert((p.time, p.channel)))
    }

    fn channels_consistency(patterns: &Vec<AddressPatterns>, ports: &Vec<AppNodePort>) -> bool {
        let mut ret = true;
        let mut graph: Vec<(i32, i32)> = vec![];
        let mut address = 0;
        
        for port in ports {
            let max_address = address + port.token_size;
            
            // get all patterns within [address, max_address)
            let filtered_patterns: Vec<&AddressPatterns> = patterns
                .iter()
                .filter(|p| address <= p.address && p.address < max_address)
                .collect();
            
            let channels: Vec<i32> = filtered_patterns.iter().map(|p| p.channel).collect();
            let min_channel = *channels.iter().min().unwrap();
            let max_channel = *channels.iter().max().unwrap();

            // each port connection must have an individual set of continuous channels
            for (checked_min, checked_max) in &graph {
                if !(min_channel > *checked_max || max_channel < *checked_min) {
                    ret = false;
                    break;
                } 
            }

            if !ret {
                break;
            }

            graph.push((min_channel, max_channel));
            address += port.token_size;
        }
        
        ret 
    }


    for binding in &db.synthesized_information.alimp_bindings {
        node_id = binding.app_node_id.clone();
        let max_channels = binding.alimp_instance.width;
        let in_patterns = &binding.alimp_instance.input_addr_time_patterns;
        let out_patterns = &binding.alimp_instance.output_addr_time_patterns;
        
        let node = db.app_graph.nodes
            .iter()
            .find(|n| n.id == node_id)
            .ok_or("Cannot find the node in app graph")?;

        if is_verified {
            is_verified = all_address_unique(&in_patterns);
            if !is_verified {
                error!("fail to verify uniqueness of in_patterns of {}", node_id);
            }
        }

        if is_verified {
            is_verified = all_address_unique(&out_patterns);
            if !is_verified {
                error!("fail to verify uniqueness of out_patterns of {}", node_id);
            }
        }

        if is_verified {
            is_verified = channels_in_bound(&in_patterns, max_channels);
            if !is_verified {
                error!("fail to verify in-bound channels of in_patterns {}", node_id);
            }
        }

        if is_verified {
            is_verified = channels_in_bound(&out_patterns, max_channels);
            if !is_verified {
                error!("fail to verify out-bound channels of out_patterns of {}", node_id);
            }
        }

        if is_verified {
            is_verified = channels_consistency(&in_patterns, &node.input_ports);
            if !is_verified {
                error!("fail to verify channel consistency of in_patterns of {}", node_id);
            }
        }   
        
        if is_verified {
            is_verified = channels_consistency(&out_patterns, &node.output_ports);
            if !is_verified {
                error!("fail to verify channel consistency of out_patterns of {}", node_id);
            }
        }

        if !is_verified {
            break;
        }
    }

    Ok((is_verified, node_id))
}


fn verify_global_timing(
    db: &DataBase,
    schedules: &optimiser::ScheduleStruct,
) -> bool {
    let mut is_verified = true;

    // latency
    for node in &db.app_graph.nodes {
        if is_verified {
            is_verified = schedules.end_time[&node.id] <= db.global_constraint.max_latency;
        }
    }

    // throughput
    for node in &db.app_graph.nodes {
        if is_verified {
            let period = schedules.end_time[&node.id] - schedules.fire_time[&node.id];
            is_verified = period <= db.global_constraint.max_period;
        }
    }

    is_verified
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



fn update_synthesized_information(
    db: &mut DataBase,
    schedules: &optimiser::ScheduleStruct,
) -> Result<(), Box<dyn std::error::Error>> {
   
    let node_ids: Vec<_> = db.app_graph.nodes.iter().map(|n| n.id.clone()).collect();
    let edge_ids: Vec<_> = db.app_graph.edges.iter().map(|e| e.id.clone()).collect();

    // fire_time, input/output buffer size
    for nid in &node_ids {
        db.synthesized_information.node_fire_times.insert(
            nid.clone(),
            schedules.fire_time[nid]
        );

        let node = db.app_graph.nodes.iter().find(|n| n.id == *nid).unwrap();

        let input_port_names: Vec<_> = node.input_ports.iter().map(|p| p.id.clone()).collect();
        for port in &input_port_names {
            let new_memory = MemorySynthesis{
                app_node_id: nid.clone(),
                port_id: port.clone(),
                memory_direction: "in".to_string(),
                memory_structure: vec![
                    MemoryStructure {
                        memory_type: "None".to_string(),
                        memory_size: schedules.ib[port] as u32,
                        input_channels: vec![],
                        output_channels: vec![],
                        corresponding_channels: vec![],
                        placement: MemoryPlacement {
                           x: -1,
                           y: -1,
                           width: -1,
                           height: -1,
                        },
                    }
                ]
            };
            db.synthesized_information.memory_synthesis.push(new_memory);
        }

        let output_port_names: Vec<_> = node.output_ports.iter().map(|p| p.id.clone()).collect();
        for port in &output_port_names {
            let new_memory = MemorySynthesis{
                app_node_id: nid.clone(),
                port_id: port.clone(),
                memory_direction: "out".to_string(),
                memory_structure: vec![
                    MemoryStructure {
                        memory_type: "None".to_string(),
                        memory_size: schedules.ob[port] as u32,
                        input_channels: vec![],
                        output_channels: vec![],
                        corresponding_channels: vec![],
                        placement: MemoryPlacement {
                           x: -1,
                           y: -1,
                           width: -1,
                           height: -1,
                        },
                    }
                ]
            };
            db.synthesized_information.memory_synthesis.push(new_memory);
        }
    }

    // channel width
    for eid in &edge_ids {
        // TODO: In case of fire time > 1, 
        // to figure out when each transporter fires, 
        // we should use the min value of all in T1

        let transporter_id = format!("transporter_{}", eid);
        
        db.synthesized_information.channel_width.insert(
            transporter_id.clone(),
            schedules.k[eid]
        );
    }


    // helper function to translate addresses
    fn translate_addresses(db: &DataBase, node_id: &str, port_id: &str, in_addr: Vec<i32>, dir: &str) -> Vec<i32> {
        let node = db.app_graph.nodes.iter().find(|n| n.id == node_id).unwrap();
        let ports = if dir == "in" {
            &node.input_ports
        } else {
            &node.output_ports
        };

        let mut out_addr = vec![0; in_addr.len()];

        for port in ports {
            if port.id == port_id {
                for (i, val) in out_addr.iter_mut().enumerate() {
                    *val += in_addr[i];
                }
                break;
            }
            for val in out_addr.iter_mut() {
                *val += port.token_size;
            }
        }

        out_addr
    }


    debug!("assigning translation tables");
    for nid in &node_ids {
        let node = db.app_graph.nodes.iter().find(|n| n.id == *nid).unwrap();
        let input_port_names: Vec<_> = node.input_ports.iter().map(|p| p.id.clone()).collect();
        let output_port_names: Vec<_> = node.output_ports.iter().map(|p| p.id.clone()).collect();

        let mut offset = 0;
        for out_port in output_port_names {
            let edge = db.app_graph.edges
                .iter()
                .find(|e| e.source_node == *nid && e.source_port == out_port)
                .unwrap();
            
            let mut assigned_address: Vec<i32> = equitable_address_assignment(
                &schedules.t0[&edge.id],
                &schedules.t1[&edge.id],
                &schedules.ob[&out_port],
            )?; 
           
            // apply offset
            for addr in assigned_address.iter_mut() {
                *addr += offset;
            }

            let memory_address = translate_addresses(
                db, 
                nid, 
                &out_port, 
                (0..assigned_address.len()).map(|x| x as i32).collect::<Vec<i32>>(), 
                "out"
            );
            
            let mut assignment = AddressTranslation {
                app_node_id: nid.clone(),
                port_id: out_port.clone(),
                address_assignment: HashMap::new(),
                translation_table: HashMap::new(),
            };

            for i in 0..assigned_address.len() {
                assignment.address_assignment.insert(
                    memory_address[i],
                    (0, 0, assigned_address[i]) // from channel, to bank, physical address
                );
            }

            db.synthesized_information.address_translations.push(assignment);
            offset += schedules.ob[&out_port];
        } 

        offset = 0;
        for in_port in input_port_names {
            let edge = db.app_graph.edges
                .iter()
                .find(|e| e.target_node == *nid && e.target_port == in_port)
                .unwrap();
            
            let mut assigned_address: Vec<i32> = equitable_address_assignment(
                &schedules.t2[&edge.id],
                &schedules.t3[&edge.id],
                &schedules.ib[&in_port],
            )?; 
           
            // apply offset
            for addr in assigned_address.iter_mut() {
                *addr += offset;
            }

            let memory_address = translate_addresses(
                db, 
                nid, 
                &in_port, 
                (0..assigned_address.len()).map(|x| x as i32).collect::<Vec<i32>>(), 
                "in"
            );
            
            let mut assignment = AddressTranslation {
                app_node_id: nid.clone(),
                port_id: in_port.clone(),
                address_assignment: HashMap::new(),
                translation_table: HashMap::new(),
            };

            for i in 0..assigned_address.len() {
                assignment.address_assignment.insert(
                    memory_address[i],
                    (0, 0, assigned_address[i])
                );
            }

            db.synthesized_information.address_translations.push(assignment);
            offset += schedules.ib[&in_port];
        } 

    }

    debug!("assigning transporter instructions");
    for edge in &db.app_graph.edges {
        let transporter_name = format!("transporter_{}", edge.id);
 
        let mut time_array = schedules.t1
            .get(&edge.id)
            .expect("Missing t1 schedule for edge")
            .clone();
        let fire_time = *time_array.iter().min().unwrap();
 
        db.synthesized_information.node_fire_times.insert(
            transporter_name.clone(),
            fire_time
        );       

        // update time table 
        for val in time_array.iter_mut() {
            *val -= fire_time;
        }

        let virtual_source_address = translate_addresses(
            db, 
            &edge.source_node, 
            &edge.source_port, 
            (0..time_array.len()).map(|x| x as i32).collect(), 
            "out"
        );

        let source_assignment = db.synthesized_information.address_translations
            .iter()
            .find(|c| c.app_node_id == edge.source_node && c.port_id == edge.source_port)
            .map(|c| &c.address_assignment)
            .ok_or(format!(
                "No source address assignment for node {}",
                edge.source_node
            ))?;

        let source_address: Result<Vec<i32>, _> = virtual_source_address
            .iter()
            .map(|&vaddr| {
                source_assignment
                    .get(&vaddr)
                    .map(|&(_, _, phy_addr)| phy_addr)
                    .ok_or_else(|| format!(
                        "Cannot find source address for virtual address {} in node {}",
                        vaddr, edge.source_node
                    ))
            })
            .collect();

        let virtual_target_address = translate_addresses(
            db, 
            &edge.target_node, 
            &edge.target_port, 
            (0..time_array.len()).map(|x| x as i32).collect(), 
            "in"
        );

        let target_assignment = db.synthesized_information.address_translations
            .iter()
            .find(|c| c.app_node_id == edge.target_node && c.port_id == edge.target_port)
            .map(|c| &c.address_assignment)
            .ok_or(format!(
                "No target address assignment for node {}",
                edge.target_node
            ))?;
        
        let target_address: Result<Vec<i32>, _> = virtual_target_address
            .iter()
            .map(|&vaddr| {
                target_assignment
                    .get(&vaddr)
                    .map(|&(_, _, phy_addr)| phy_addr)
                    .ok_or_else(|| format!(
                        "Cannot find target address for virtual address {} in node {}",
                        vaddr, edge.target_node
                    ))
            })
            .collect();
        
        // assign instructions to transporter
        let (source_address, target_address) = (source_address?, target_address?);
        
        let entries: Vec<TransportTableEntry> = source_address
            .iter()
            .zip(target_address.iter())
            .zip(time_array.iter())
            .map(|((&src, &tgt), &t)| TransportTableEntry {
                relative_time: t,
                source_address: src,
                target_address: tgt,
            })
            .collect();
       
        db.synthesized_information.transporter_tables.push(
            TransporterTable {
                transporter_id: transporter_name.clone(),
                fire_time: fire_time,
                end_time: fire_time + time_array.iter().max().unwrap(),
                latency: *time_array.iter().max().unwrap() as u32,
                entries: entries,
                ir: BTreeMap::new(),
                binary: Vec::new(),
                size: 0,
                from: 0,
                to: 0,
                placement: MemoryPlacement {
                    x: -1,
                    y: -1,
                    width: -1,
                    height: -1,
                }
            }
        );
    }


    Ok(())
}


#[allow(unused_variables)]
pub fn run(
    db: &mut DataBase,
    dir: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Start: glic");
    let module_dir = format!("{}glic", dir);
    match std::fs::create_dir_all(&module_dir) {
        Ok(_) => (),
        Err(e) => {
            error!("Failed to create {} with {}", module_dir, e);
            return Err(Box::new(e));
        },
    };

    info!("Stage 1: optimise channel width and delay");
    select_alimp(db)?;
    let (status, node_id) = verify_alimp_patterns(db)?;
    if !status {
        return Err(format!("{} alimp fails to verify the patterns", node_id).into());
    }
    let channels = optimiser::solve_channel_width(db, module_dir.clone())?;
    debug!("Solved channel width: \n{:?}", channels);

    info!("Stage 2: optimise scheduling");
    let schedules: optimiser::ScheduleStruct = optimiser::solve_scheduling(db, channels, module_dir.clone())?;

    info!("Stage 3: post optimisation");
    loop {
        // TODO: post_optimisation();
        if verify_global_timing(db, &schedules) {
            break
        }
            
        error!("GLIC fails to schedule, timing uncleaned (TODO: post_optimisation)");
        return Err(format!("GLIC global timing verification fails").into());
    }

    info!("Stage 4: update information");
    update_synthesized_information(db, &schedules)?;

    info!("Finish: glic");
    Ok(())
}
