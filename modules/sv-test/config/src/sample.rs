use sv_lib::model::*;


fn common(db: &mut DataBase) -> Result<(), Box<dyn std::error::Error>> {
    
    db.global_constraint.max_energy = 200;
    db.global_constraint.max_width = 200;
    db.global_constraint.max_height = 200;
    db.global_constraint.max_latency = 1000;
    db.global_constraint.max_period = 400;

    db.hyper_parameter.bind_w_area = 1;
    db.hyper_parameter.bind_w_energy = 1;
    db.hyper_parameter.bind_w_latency = 1;
    db.hyper_parameter.bind_relaxation_factor = 1.1;
    db.hyper_parameter.place_relaxation_factor = 1.0;
    db.hyper_parameter.place_reserved_routing_size = 1;

    Ok(())
}


pub fn sample1(db: &mut DataBase) -> Result<(), Box<dyn std::error::Error>> {
    common(db)?;

    db.alimp_lib.entries.push(AlimpEntry {
        func: "FA".to_string(),
        instances: vec![
            AlimpInstance { width: 2, height: 2, energy: 1, latency: 8,
                input_addr_time_patterns: vec![],
                output_addr_time_patterns: vec![AddressPatterns { address: 0, channel: 0, time: 0 }, AddressPatterns { address: 1, channel: 1, time: 0 }, AddressPatterns { address: 2, channel: 0, time: 1 }, AddressPatterns { address: 3, channel: 1, time: 1 }, AddressPatterns { address: 4, channel: 0, time: 2 }, AddressPatterns { address: 5, channel: 1, time: 2 }, AddressPatterns { address: 6, channel: 0, time: 3 }, AddressPatterns { address: 7, channel: 1, time: 3 }, AddressPatterns { address: 8, channel: 0, time: 4 }, AddressPatterns { address: 9, channel: 1, time: 4 }, AddressPatterns { address: 10, channel: 0, time: 5 }, AddressPatterns { address: 11, channel: 1, time: 5 }, AddressPatterns { address: 12, channel: 0, time: 6 }, AddressPatterns { address: 13, channel: 1, time: 6 }, AddressPatterns { address: 14, channel: 0, time: 7 }, AddressPatterns { address: 15, channel: 1, time: 7 }],
                ..Default::default()
            },
        ],
    });

    db.alimp_lib.entries.push(AlimpEntry {
        func: "FC".to_string(),
        instances: vec![
            AlimpInstance { width: 2, height: 2, energy: 1, latency: 8,
                input_addr_time_patterns: vec![AddressPatterns { address: 0, channel: 0, time: 0 }, AddressPatterns { address: 1, channel: 1, time: 0 }, AddressPatterns { address: 2, channel: 0, time: 1 }, AddressPatterns { address: 3, channel: 1, time: 1 }, AddressPatterns { address: 4, channel: 0, time: 2 }, AddressPatterns { address: 5, channel: 1, time: 2 }, AddressPatterns { address: 6, channel: 0, time: 3 }, AddressPatterns { address: 7, channel: 1, time: 3 }, AddressPatterns { address: 8, channel: 0, time: 4 }, AddressPatterns { address: 9, channel: 1, time: 4 }, AddressPatterns { address: 10, channel: 0, time: 5 }, AddressPatterns { address: 11, channel: 1, time: 5 }, AddressPatterns { address: 12, channel: 0, time: 6 }, AddressPatterns { address: 13, channel: 1, time: 6 }, AddressPatterns { address: 14, channel: 0, time: 7 }, AddressPatterns { address: 15, channel: 1, time: 7 }],
                output_addr_time_patterns: vec![],
                ..Default::default()
            },
        ],
    });

    db.app_graph.nodes.push(AppNode {
        id: "A".to_string(),
        func: "FA".to_string(),
        executable: "examples/copy/A".to_string(),
        input_ports: vec![],
        output_ports: vec![AppNodePort {id: "A:ac".to_string(), rate: 1, token_size: 16, ..Default::default()},],
        ..Default::default()
    });
    
    db.app_graph.nodes.push(AppNode {
        id: "C".to_string(),
        func: "FC".to_string(),
        executable: "examples/copy/C".to_string(),
        input_ports: vec![AppNodePort {id: "C:ac".to_string(), rate: 1, token_size: 16, ..Default::default()},],
        output_ports: vec![],
        ..Default::default()
    });

    db.app_graph.edges.push(AppEdge {
        id: "A_C".to_string(),
        source_node: "A".to_string(),
        target_node: "C".to_string(),
        source_port: "A:ac".to_string(),
        target_port: "C:ac".to_string(),
        token_size: 16,
        ..Default::default()
    });
    
    db.app_graph.global_mem_image = "examples/copy/mem/global_mem_image.json".to_string();
    db.app_graph.global_mem_reference = "examples/copy/mem/global_mem_reference.json".to_string();

    Ok(())
}


