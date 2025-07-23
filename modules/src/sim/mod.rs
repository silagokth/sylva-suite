use crate::model::{DataBase};
use crate::model::sim::{MemoryList, NodeConfigMap, NodeConfig, TimeTable, TTNode};
use crate::file_handler;
use log::{info, debug, error};


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
                delay: route_delay,
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
    let mut j = TimeTable { list: Vec::new() };

    for (name, fire_time) in &db.synthesized_information.node_fire_times {
        j.list.push( 
            TTNode {
                cycle: *fire_time,
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
    let config_json = create_config_file(&db)?;
    file_handler::write_json_file(config_file, config_json)?;

    // 2. time table file
    let timetable_file = format!("{}/time_table.json", module_dir);
    let timetable_json = create_timetable(&db)?;
    file_handler::write_json_file(timetable_file, timetable_json)?;

    // 3. input/output address patterns
    for binding in &db.synthesized_information.alimp_bindings {
        let in_file = format!("{}/{}_inAP.json", module_dir, binding.app_node_id);
        let out_file = format!("{}/{}_outAP.json", module_dir, binding.app_node_id);
        let inst = binding.alimp_instance;

        if inst.input_addr_time_patterns.len() > 0 {
            let mut j = AddressPatternList { addr_ptrn: Vec::new() }; 
            for (key, val) in inst.input_addr_time_patterns {
                j.addr_ptrn.push(
                    address: key as String,
                    cycle: val as String,
                );
            }
            file_handler::write_json_file(in_file, j)?;
        }

        if inst.output_addr_time_patterns.len() > 0 {
            let mut j = AddressPatternList { addr_ptrn: Vec::new() }; 
            for (key, val) in inst.output_addr_time_patterns {
                j.addr_ptrn.push(
                    address: key as String,
                    cycle: val as String,
                );
            }
            file_handler::write_json_file(out_file, j)?;
        }
    }
    
    // 4. address translation tables



    // 5. transporter instructions

    Ok()
}


fn verify_simulation(
    module_dir: String,
) -> Result<bool, Box<dyn std::error::Error>> {
    
    let memory_image_file = format!("{}/global_mem_image.json", module_dir);
    let reference_image_file = format!("{}/global_mem_reference.json", module_dir);

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
    
    info!("Stage 3: verify simulation");
    let result = verify_simulation(module_dir.clone())?;

    info!("Finish: simulation");
    Ok(result)
}

