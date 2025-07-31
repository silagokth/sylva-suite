use sv_lib::model::*;


pub fn copy(db: &mut DataBase) -> Result<(), Box<dyn std::error::Error>> {
    
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
            AlimpInstance {
                width: 3,
                height: 3,
                energy: 1,
                latency: 11,
                input_addr_time_patterns: vec![], 
                output_addr_time_patterns: vec![
                    PairIntInt { key: 0, value: 2 }, PairIntInt { key: 1, value: 2 }, PairIntInt { key: 2, value: 2 },
                    PairIntInt { key: 3, value: 2 }, PairIntInt { key: 4, value: 2 }, PairIntInt { key: 5, value: 2 },
                    PairIntInt { key: 6, value: 4 }, PairIntInt { key: 7, value: 7 }, PairIntInt { key: 8, value: 3 },
                    PairIntInt { key: 9, value: 10 }, PairIntInt { key: 10, value: 8 }, PairIntInt { key: 11, value: 6 },
                    PairIntInt { key: 12, value: 5 }, PairIntInt { key: 13, value: 9 }, PairIntInt { key: 14, value: 7 },
                    PairIntInt { key: 15, value: 10 }
                ],
                ..Default::default()
            },
        ],
    });

    db.alimp_lib.entries.push(AlimpEntry {
        func: "FB".to_string(),
        instances: vec![
            AlimpInstance {
                width: 2,
                height: 2,
                energy: 1,
                latency: 17,
                input_addr_time_patterns: vec![
                    PairIntInt { key: 0, value: 0 }, PairIntInt { key: 1, value: 1 }, PairIntInt { key: 2, value: 4 },
                    PairIntInt { key: 3, value: 2 }, PairIntInt { key: 4, value: 5 }, PairIntInt { key: 5, value: 0 },
                    PairIntInt { key: 6, value: 3 }, PairIntInt { key: 7, value: 1 }, PairIntInt { key: 8, value: 5 },
                    PairIntInt { key: 9, value: 2 }, PairIntInt { key: 10, value: 1 }, PairIntInt { key: 11, value: 2 },
                    PairIntInt { key: 12, value: 3 }, PairIntInt { key: 13, value: 3 }, PairIntInt { key: 14, value: 1 },
                    PairIntInt { key: 15, value: 5 }
                ],
                output_addr_time_patterns: vec![
                    PairIntInt { key: 0, value: 1 }, PairIntInt { key: 1, value: 2 }, PairIntInt { key: 2, value: 3 },
                    PairIntInt { key: 3, value: 4 }, PairIntInt { key: 4, value: 5 }, PairIntInt { key: 5, value: 6 },
                    PairIntInt { key: 6, value: 7 }, PairIntInt { key: 7, value: 8 }, PairIntInt { key: 8, value: 9 },
                    PairIntInt { key: 9, value: 10 }, PairIntInt { key: 10, value: 11 }, PairIntInt { key: 11, value: 12 },
                    PairIntInt { key: 12, value: 13 }, PairIntInt { key: 13, value: 14 }, PairIntInt { key: 14, value: 15 },
                    PairIntInt { key: 15, value: 16 }
                ],
                ..Default::default()
            },
        ],
    });

    db.alimp_lib.entries.push(AlimpEntry {
        func: "FC".to_string(),
        instances: vec![
            AlimpInstance {
                width: 4,
                height: 4,
                energy: 10,
                latency: 16,
                input_addr_time_patterns: vec![
                    PairIntInt { key: 0, value: 0 }, PairIntInt { key: 1, value: 1 }, PairIntInt { key: 2, value: 2 },
                    PairIntInt { key: 3, value: 3 }, PairIntInt { key: 4, value: 4 }, PairIntInt { key: 5, value: 5 },
                    PairIntInt { key: 6, value: 6 }, PairIntInt { key: 7, value: 7 }, PairIntInt { key: 8, value: 8 },
                    PairIntInt { key: 9, value: 9 }, PairIntInt { key: 10, value: 10 }, PairIntInt { key: 11, value: 11 },
                    PairIntInt { key: 12, value: 12 }, PairIntInt { key: 13, value: 13 }, PairIntInt { key: 14, value: 14 },
                    PairIntInt { key: 15, value: 15 }
                ],
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
        output_ports: vec![AppNodePort { id: "A:ab".to_string(), rate: 1, token_size: 16, ..Default::default() }],
        ..Default::default()
    });

    db.app_graph.nodes.push(AppNode {
        id: "B".to_string(),
        func: "FB".to_string(),
        executable: "examples/copy/B".to_string(),
        input_ports: vec![AppNodePort { id: "B:ab".to_string(), rate: 1, token_size: 16, ..Default::default() }],
        output_ports: vec![AppNodePort { id: "B:bc".to_string(), rate: 1, token_size: 16, ..Default::default() }],
        ..Default::default()
    });

    db.app_graph.nodes.push(AppNode {
        id: "C".to_string(),
        func: "FC".to_string(),
        executable: "examples/copy/C".to_string(),
        input_ports: vec![AppNodePort { id: "C:bc".to_string(), rate: 1, token_size: 16, ..Default::default() }],
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

    db.app_graph.global_mem_image = "examples/copy/mem/global_mem_image.json".to_string();
    db.app_graph.global_mem_reference = "examples/copy/mem/global_mem_reference.json".to_string();

    Ok(())
}
