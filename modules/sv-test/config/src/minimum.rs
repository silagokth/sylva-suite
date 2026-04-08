use sv_lib::model::*;


pub fn minimum(db: &mut DataBase) -> Result<(), Box<dyn std::error::Error>> {
    
    db.global_constraint.max_energy = 100;
    db.global_constraint.max_width = 100;
    db.global_constraint.max_height = 100;
    db.global_constraint.max_latency = 100;
    db.global_constraint.max_period = 70;

    db.hyper_parameter.bind_w_area = 1;
    db.hyper_parameter.bind_w_energy = 1;
    db.hyper_parameter.bind_w_latency = 1;
    db.hyper_parameter.bind_relaxation_factor = 1.1;
    db.hyper_parameter.place_relaxation_factor = 1.0;
    db.hyper_parameter.place_reserved_routing_size = 1;

    db.alimp_lib.entries.push(AlimpEntry {
        func: "FA".to_string(),
        instances: vec![
            AlimpInstance { width: 2, height: 2, energy: 1, latency: 26,
                input_addr_time_patterns: vec![],
                output_addr_time_patterns: vec![AddressPatterns { address: 0, channel: 0, time: 10 }, AddressPatterns { address: 1, channel: 0, time: 11 }, AddressPatterns { address: 2, channel: 0, time: 12 }, AddressPatterns { address: 3, channel: 0, time: 13 }, AddressPatterns { address: 4, channel: 0, time: 14 }, AddressPatterns { address: 5, channel: 0, time: 15 }, AddressPatterns { address: 6, channel: 0, time: 16 }, AddressPatterns { address: 7, channel: 0, time: 17 }, AddressPatterns { address: 8, channel: 0, time: 18 }, AddressPatterns { address: 9, channel: 0, time: 19 }, AddressPatterns { address: 10, channel: 0, time: 20 }, AddressPatterns { address: 11, channel: 0, time: 21 }, AddressPatterns { address: 12, channel: 0, time: 22 }, AddressPatterns { address: 13, channel: 0, time: 23 }, AddressPatterns { address: 14, channel: 0, time: 24 }, AddressPatterns { address: 15, channel: 0, time: 25 }],
                instruction_code: vec![1000, 1002, 1003, 1004],
                instruction_offsets: vec![vec![0, 1], vec![2, 3]],
                number_of_instructions: vec![vec![1, 1], vec![1, 1]],
                start_address_cells: vec![vec![100, 200], vec![300, 400]],
                ..Default::default()
            },
        ],
    });
    
    db.alimp_lib.entries.push(AlimpEntry {
        func: "FB".to_string(),
        instances: vec![
            AlimpInstance { width: 1, height: 1, energy: 1, latency: 32,
                input_addr_time_patterns: vec![AddressPatterns { address: 0, channel: 0, time: 0 }, AddressPatterns { address: 1, channel: 0, time: 1 }, AddressPatterns { address: 2, channel: 0, time: 2 }, AddressPatterns { address: 3, channel: 0, time: 3 }, AddressPatterns { address: 4, channel: 0, time: 4 }, AddressPatterns { address: 5, channel: 0, time: 5 }, AddressPatterns { address: 6, channel: 0, time: 6 }, AddressPatterns { address: 7, channel: 0, time: 7 }, AddressPatterns { address: 8, channel: 0, time: 8 }, AddressPatterns { address: 9, channel: 0, time: 9 }, AddressPatterns { address: 10, channel: 0, time: 10 }, AddressPatterns { address: 11, channel: 0, time: 11 }, AddressPatterns { address: 12, channel: 0, time: 12 }, AddressPatterns { address: 13, channel: 0, time: 13 }, AddressPatterns { address: 14, channel: 0, time: 14 }, AddressPatterns { address: 15, channel: 0, time: 15 }],
                output_addr_time_patterns: vec![AddressPatterns { address: 0, channel: 0, time: 16 }, AddressPatterns { address: 1, channel: 0, time: 17 }, AddressPatterns { address: 2, channel: 0, time: 18 }, AddressPatterns { address: 3, channel: 0, time: 19 }, AddressPatterns { address: 4, channel: 0, time: 20 }, AddressPatterns { address: 5, channel: 0, time: 21 }, AddressPatterns { address: 6, channel: 0, time: 22 }, AddressPatterns { address: 7, channel: 0, time: 23 }, AddressPatterns { address: 8, channel: 0, time: 24 }, AddressPatterns { address: 9, channel: 0, time: 25 }, AddressPatterns { address: 10, channel: 0, time: 26 }, AddressPatterns { address: 11, channel: 0, time: 27 }, AddressPatterns { address: 12, channel: 0, time: 28 }, AddressPatterns { address: 13, channel: 0, time: 29 }, AddressPatterns { address: 14, channel: 0, time: 30 }, AddressPatterns { address: 15, channel: 0, time: 31 }],
                instruction_code: vec![999],
                instruction_offsets: vec![vec![0]],
                number_of_instructions: vec![vec![1]],
                start_address_cells: vec![vec![50]],
                ..Default::default()
            },
        ],
    });
    
    db.alimp_lib.entries.push(AlimpEntry {
        func: "FC".to_string(),
        instances: vec![
            AlimpInstance { width: 4, height: 4, energy: 10, latency: 16,
                input_addr_time_patterns: vec![AddressPatterns { address: 0, channel: 0, time: 0 }, AddressPatterns { address: 1, channel: 0, time: 1 }, AddressPatterns { address: 2, channel: 0, time: 2 }, AddressPatterns { address: 3, channel: 0, time: 3 }, AddressPatterns { address: 4, channel: 0, time: 4 }, AddressPatterns { address: 5, channel: 0, time: 5 }, AddressPatterns { address: 6, channel: 0, time: 6 }, AddressPatterns { address: 7, channel: 0, time: 7 }, AddressPatterns { address: 8, channel: 0, time: 8 }, AddressPatterns { address: 9, channel: 0, time: 9 }, AddressPatterns { address: 10, channel: 0, time: 10 }, AddressPatterns { address: 11, channel: 0, time: 11 }, AddressPatterns { address: 12, channel: 0, time: 12 }, AddressPatterns { address: 13, channel: 0, time: 13 }, AddressPatterns { address: 14, channel: 0, time: 14 }, AddressPatterns { address: 15, channel: 0, time: 15 }],
                output_addr_time_patterns: vec![],
                instruction_code: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25],
                instruction_offsets: vec![vec![0, 2, 3, 6], vec![7, 8, 10, 11], vec![12, 14, 16, 17], vec![19, 20, 21, 22]],
                number_of_instructions: vec![vec![2, 1, 3, 1], vec![1, 2, 1, 1], vec![2, 2, 1, 2], vec![1, 1, 1, 3]],
                start_address_cells: vec![vec![100, 200, 300, 400], vec![500, 600, 700, 800], vec![900, 1000, 1100, 1200], vec![1300, 1400, 1500, 1600]],
                ..Default::default()
            },
        ],
    });

    db.app_graph.nodes.push(AppNode {
        id: "A".to_string(),
        func: "FA".to_string(),
        executable: "examples/minimum/A".to_string(),
        input_ports: vec![],
        output_ports: vec![AppNodePort {id: "A:ab".to_string(), rate: 1, token_size: 16, ..Default::default()},],
        ..Default::default()
    });
    
    db.app_graph.nodes.push(AppNode {
        id: "B".to_string(),
        func: "FB".to_string(),
        executable: "examples/minimum/B".to_string(),
        input_ports: vec![AppNodePort {id: "B:ab".to_string(), rate: 1, token_size: 16, ..Default::default()},],
        output_ports: vec![AppNodePort {id: "B:bc".to_string(), rate: 1, token_size: 16, ..Default::default()},],
        ..Default::default()
    });
    
    db.app_graph.nodes.push(AppNode {
        id: "C".to_string(),
        func: "FC".to_string(),
        executable: "examples/minimum/C".to_string(),
        input_ports: vec![AppNodePort {id: "C:bc".to_string(), rate: 1, token_size: 16, ..Default::default()},],
        output_ports: vec![],
        ..Default::default()
    });
    
    db.app_graph.edges.push(AppEdge {
        id: "A_B".to_string(),
        source_node: "A".to_string(),
        target_node: "B".to_string(),
        source_port: "A:ab".to_string(),
        target_port: "B:ab".to_string(),
        token_size: 16,
        ..Default::default()
    });
    
    db.app_graph.edges.push(AppEdge {
        id: "B_C".to_string(),
        source_node: "B".to_string(),
        target_node: "C".to_string(),
        source_port: "B:bc".to_string(),
        target_port: "C:bc".to_string(),
        token_size: 16,
        ..Default::default()
    });

    db.app_graph.global_mem_image = "examples/minimum/mem/global_mem_image.json".to_string();
    db.app_graph.global_mem_reference = "examples/minimum/mem/global_mem_reference.json".to_string();

    Ok(())
}
