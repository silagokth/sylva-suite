use sv_lib::model::{DataBase, PairIntInt};
use sv_lib::sim::{
    MemoryList, NodeConfigMap, NodeConfig, 
    TimeTable, TTNode, AddressPatternList, AddressPattern, 
    TranslationTableList, TranslationTable,
    TransporterInstructionList, TransporterInstruction
};
use crate::file_handler;
use log::{info, error};
use std::collections::HashMap;
use std::io::BufRead;



fn create_config_map(
    db: &DataBase,
) -> Result<NodeConfigMap, Box<dyn std::error::Error>> {    
    let mut j = NodeConfigMap { config_map: HashMap::new() };
    
    // insert all nodes with default config
    for node in &db.app_graph.nodes {
        j.config_map.insert(
            node.id.clone(),
            NodeConfig {
                is_transporter: false,
                in_names: Vec::new(),
                out_names: Vec::new(),
                process_cmd: node.executable.clone(),
                delay: 0,
            },
        );
    }

    for edge in &db.app_graph.edges {
        let route_delay = db.synthesized_information.routing_paths
            .iter()
            .find(|path| path.app_edge_id == edge.id)
            .map(|path| path.delay)
            .ok_or_else(|| format!("Cannot find {} in the rounting paths", edge.id))?;

        let name = format!("transporter_{}", edge.id);

        // add transporter node
        j.config_map.insert(
            name.clone(),
            NodeConfig {
                is_transporter: true,
                in_names: vec![edge.source_node.clone()],
                out_names: vec![edge.target_node.clone()],
                process_cmd: String::new(),
                delay: route_delay as i64,
            },
        );
    
        // update source and target nodes
        j.config_map.get_mut(&edge.source_node)
            .ok_or_else(|| format!("Cannot find source node {}", edge.source_node))?
            .out_names
            .push(name.clone());

        j.config_map.get_mut(&edge.target_node)
            .ok_or_else(|| format!("Cannot find target node {}", edge.target_node))?
            .in_names
            .push(name);
    }

    Ok(j)
}



fn create_timetable(
    db: &DataBase,
) -> Result<TimeTable, Box<dyn std::error::Error>> {    
    let mut j = TimeTable { tt: Vec::new() };

    for (name, fire_time) in &db.synthesized_information.node_fire_times {
        j.tt.push( 
            TTNode {
                cycle: *fire_time as i64,
                node_name: name.clone(),
            }
        );
    }

    Ok(j)
}



fn generate_files(
    db: &DataBase,
    module_dir: String,
) -> Result<(), Box<dyn std::error::Error>> {

    // 1. configuration file
    let config_file = format!("{}/config_map.json", module_dir);
    let config_json = create_config_map(&db)?;
    file_handler::write_json_file(&config_file, &config_json)?;

    // 2. time table file
    let timetable_file = format!("{}/time_table.json", module_dir);
    let timetable_json = create_timetable(&db)?;
    file_handler::write_json_file(&timetable_file, &timetable_json)?;

    // 3. input/output address patterns
    for binding in &db.synthesized_information.alimp_bindings {
        let in_file = format!("{}/{}_inAP.json", module_dir, binding.app_node_id);
        let out_file = format!("{}/{}_outAP.json", module_dir, binding.app_node_id);
        let inst = &binding.alimp_instance;

        if inst.input_addr_time_patterns.len() > 0 {
            let mut j = AddressPatternList { addr_ptrn: Vec::new() }; 
            for PairIntInt { key, value } in &inst.input_addr_time_patterns {
                j.addr_ptrn.push(
                    AddressPattern {
                        address: *key as i64,
                        cycle: *value as i64,
                    }
                );
            }
            file_handler::write_json_file(&in_file, &j)?;
        }

        if inst.output_addr_time_patterns.len() > 0 {
            let mut j = AddressPatternList { addr_ptrn: Vec::new() }; 
            for PairIntInt { key, value } in &inst.output_addr_time_patterns {
                j.addr_ptrn.push(
                    AddressPattern {
                        address: *key as i64,
                        cycle: *value as i64,
                    }
                );
            }
            file_handler::write_json_file(&out_file, &j)?;
        }
    }
    
    // 4. address translation tables
    for node in &db.app_graph.nodes {
        if !node.input_ports.is_empty() {
            let file = format!("{}/{}_inTT.json", module_dir, node.id);
            let mut j = TranslationTableList { list: Vec::new() };

            for chunk in &db.synthesized_information.chunk_address_assignments {
                if chunk.app_node_id == node.id && node.input_ports.iter().any(|p| p.id == chunk.port_id) {
                    for (&addr_in, &addr_out) in &chunk.address_assignment {
                        j.list.push(
                            TranslationTable { 
                                addr_in: addr_in as i64, 
                                addr_out: addr_out as i64, 
                            }
                        );
                    }
                }
            }
            j.list.sort_by_key(|entry| entry.addr_in);

            file_handler::write_json_file(&file, &j)?;
        }

        if !node.output_ports.is_empty() {
            let file = format!("{}/{}_outTT.json", module_dir, node.id);
            let mut j = TranslationTableList { list: Vec::new() };

            for chunk in &db.synthesized_information.chunk_address_assignments {
                if chunk.app_node_id == node.id && node.output_ports.iter().any(|p| p.id == chunk.port_id) {
                    for (&addr_in, &addr_out) in &chunk.address_assignment {
                        j.list.push(
                            TranslationTable { 
                                addr_in: addr_in as i64, 
                                addr_out: addr_out as i64, 
                            }
                        );
                    }
                }
            }
            j.list.sort_by_key(|entry| entry.addr_in);

            file_handler::write_json_file(&file, &j)?;
        }
    } 

    // 5. transporter instructions
    for edge in &db.app_graph.edges {
        let file = format!("{}/transporter_{}_TransInst.json", module_dir, edge.id);
        let mut j = TransporterInstructionList { inst_list: Vec::new() };
        
        if let Some(table) = db.synthesized_information.transport_tables.get(&edge.id) {
            for t in &table.entries {
                j.inst_list.push(
                    TransporterInstruction {
                        cycle: t.time as i64,
                        addr_rd: t.source_address as i64,
                        addr_wr: t.target_address as i64, 
                    }
                );
            }
        } else {
            return Err(format!("No transport table found for edge {}", edge.id).into());
        }
        
        file_handler::write_json_file(&file, &j)?;
    }

    // 6. memory files 
    // create a sub-folder in module_dir
    let mem_dir = format!("{}/mem", module_dir);
    match std::fs::create_dir_all(&mem_dir) {
        Ok(_) => (),
        Err(e) => {
            error!("Failed to create {} with {}", mem_dir, e);
            return Err(Box::new(e));
        },
    };
    
    // clean the mem directory
    for entry in std::fs::read_dir(&mem_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            std::fs::remove_file(path)?;
        } else if path.is_dir() {
            std::fs::remove_dir_all(path)?;
        }
    }

    // copy memory files to the mem directory
    let memory_image_path = format!("{}/global_mem_image.json", mem_dir);
    let reference_image_path = format!("{}/global_mem_reference.json", mem_dir);
    std::fs::copy(&db.app_graph.global_mem_image, &memory_image_path)?;
    std::fs::copy(&db.app_graph.global_mem_reference, &reference_image_path)?;

    Ok(())
}



fn run_simulator(
    module_dir: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let command_str = format!("./bin/sv-sim --dir {} ", module_dir); 

    let mut child = std::process::Command::new("sh")
        .arg("-c")
        .arg(&command_str)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to execute simulation binary: {}", e))?;
    
    let stdout = child
        .stdout
        .take()
        .ok_or("Failed to capture stdout of the process")?;

    let stderr = child
        .stderr
        .take()
        .ok_or("Failed to capture stderr of the process")?;
    
    // Stream stdout
    let stdout_reader = std::io::BufReader::new(stdout);
    let stderr_reader = std::io::BufReader::new(stderr);

    println!("");
    
    let stdout_thread = std::thread::spawn(move || {
        for line in stdout_reader.lines().flatten() {
            println!("[stdout] {}", line);
        }
    });

    let stderr_thread = std::thread::spawn(move || {
        for line in stderr_reader.lines().flatten() {
            eprintln!("[stderr] {}", line);
        }
    });

    
    let status = child.wait()?;

    stdout_thread.join().unwrap();
    stderr_thread.join().unwrap();

    println!("");
    
    if !status.success() {
        return Err(format!("Simulation binary exited with error {:?}", status).into());
    }

    Ok(())
}




fn verify_simulation(
    module_dir: String,
) -> Result<bool, Box<dyn std::error::Error>> {
    
    let memory_image_file = format!("{}/mem/global_mem_image.json", module_dir);
    let reference_image_file = format!("{}/mem/global_mem_reference.json", module_dir);

    let memory_image: MemoryList = file_handler::load_json_file(&memory_image_file)?;
    let reference_image: MemoryList = file_handler::load_json_file(&reference_image_file)?;

    Ok(memory_image == reference_image)
}



pub fn run(
    db: &mut DataBase, 
    dir: &String,
) -> Result<bool, Box<dyn std::error::Error>> {
    info!("Start: simulation");
    let module_dir = format!("{}sim", dir);
    match std::fs::create_dir_all(&module_dir) {
        Ok(_) => (),
        Err(e) => {
            error!("Failed to create {} with {}", module_dir, e);
            return Err(Box::new(e));
        },
    };

    info!("Stage 1: generate simulation files");
    generate_files(db, module_dir.clone())?;

    info!("Stage 2: run simulation");
    run_simulator(module_dir.clone())?;

    info!("Stage 3: verify simulation");
    let result = verify_simulation(module_dir.clone())?;

    info!("Finish: simulation");
    Ok(result)
}

