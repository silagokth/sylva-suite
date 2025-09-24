use sv_lib::model::*;


pub fn sobel(db: &mut DataBase) -> Result<(), Box <dyn std::error::Error>> {
 
    db.global_constraint.max_energy = 100;
    db.global_constraint.max_width = 100;
    db.global_constraint.max_height = 100;
    db.global_constraint.max_latency = 5000;
    db.global_constraint.max_period = 2000;

    db.hyper_parameter.bind_w_area = 1;
    db.hyper_parameter.bind_w_energy = 1;
    db.hyper_parameter.bind_w_latency = 1;
    db.hyper_parameter.bind_relaxation_factor = 1.1;
    db.hyper_parameter.place_relaxation_factor = 1.0;
    db.hyper_parameter.place_reserved_routing_size = 1;
 
    db.alimp_lib.entries.push(AlimpEntry {
        func: "func_load".to_string(),
        instances: vec![
            AlimpInstance { 
                width: 2, 
                height: 4, 
                energy: 4, 
                latency: 1610,
                input_addr_time_patterns: vec![],
                output_addr_time_patterns: (0..3200)
                    .map(|i| AddressPatterns {
                        address: i,
                        channel: i % 2,
                        time: 10 + i / 2,
                    })
                    .collect(), 
                ..Default::default()
            },
        ],
    });

    db.alimp_lib.entries.push(AlimpEntry {
        func: "func_copy".to_string(),
        instances: vec![
            AlimpInstance { 
                width: 4, 
                height: 1, 
                energy: 5, 
                latency: 1620,
                input_addr_time_patterns:  (0..3200)
                    .map(|i| AddressPatterns {
                        address: i,
                        channel: i % 2,
                        time: i / 2,
                    })
                    .collect(), 
                output_addr_time_patterns: (0..6400)
                    .map(|i| AddressPatterns {
                        address: i,
                        channel: if i < 3200 { i % 2 } else { (i % 2) + 2 },
                        time: 20 + (i % 3200) / 2,
                    })
                    .collect(), 
                ..Default::default()
            },
        ],
    });

    db.alimp_lib.entries.push(AlimpEntry {
        func: "func_gx".to_string(),
        instances: vec![
            AlimpInstance { 
                width: 8, 
                height: 6, 
                energy: 15, 
                latency: 1700,
                input_addr_time_patterns:  (0..3200)
                    .map(|i| AddressPatterns {
                        address: i,
                        channel: (i + 2) % 4,
                        time: i / 2,
                    })
                    .collect(), 
                output_addr_time_patterns: (0..12800)
                    .map(|i| AddressPatterns {
                        address: i,
                        channel: i % 8,
                        time: 100 + i / 8,
                    })
                    .collect(), 
                ..Default::default()
            },
        ],
    });

    db.alimp_lib.entries.push(AlimpEntry {
        func: "func_gy".to_string(),
        instances: vec![
            AlimpInstance { 
                width: 8, 
                height: 6, 
                energy: 15, 
                latency: 1700,
                input_addr_time_patterns:  (0..3200)
                    .map(|i| AddressPatterns {
                        address: i,
                        channel: (i + 3) % 4,
                        time: i / 2,
                    })
                    .collect(), 
                output_addr_time_patterns: (0..12800)
                    .map(|i| AddressPatterns {
                        address: i,
                        channel: i % 8,
                        time: 100 + i / 8,
                    })
                    .collect(), 
                ..Default::default()
            },
        ],
    });

    db.alimp_lib.entries.push(AlimpEntry {
        func: "func_combine".to_string(),
        instances: vec![
            AlimpInstance { 
                width: 16, 
                height: 12, 
                energy: 18, 
                latency: 1700,
                input_addr_time_patterns:  (0..25600)
                    .map(|i| AddressPatterns {
                        address: i,
                        channel: if i < 12800 { i % 8 } else { (i % 8) + 8 },
                        time: (i % 12800) / 8,
                    })
                    .collect(), 
                output_addr_time_patterns: (0..3200)
                    .map(|i| AddressPatterns {
                        address: i,
                        channel: i % 8,
                        time: 100 + i / 2,
                    })
                    .collect(), 
                ..Default::default()
            },
        ],
    });

    db.alimp_lib.entries.push(AlimpEntry {
        func: "func_store".to_string(),
        instances: vec![
            AlimpInstance { 
                width: 4, 
                height: 3, 
                energy: 6, 
                latency: 1600,
                input_addr_time_patterns:  (0..3200)
                    .map(|i| AddressPatterns {
                        address: i,
                        channel: i % 4,
                        time: i / 2,
                    })
                    .collect(), 
                output_addr_time_patterns: vec![],
                ..Default::default()
            },
        ],
    });
 
    db.app_graph.nodes.push(AppNode {
        id: "load".to_string(),
        func: "func_load".to_string(),
        executable: "examples/sobel/load".to_string(),
        output_ports: vec![
            AppNodePort {
                id: "load_output".to_string(),
                rate: 1,
                token_size: 3200,
                ..Default::default()
            },
        ],
        ..Default::default()
    });

    db.app_graph.nodes.push(AppNode {
        id: "copy".to_string(),
        func: "func_copy".to_string(),
        executable: "examples/sobel/copy".to_string(),
        input_ports: vec![
            AppNodePort {
                id: "copy_input".to_string(),
                rate: 1,
                token_size: 3200,
                ..Default::default()
            },
        ],
        output_ports: vec![
            AppNodePort {
                id: "copy_output_0".to_string(),
                rate: 1,
                token_size: 3200,
                ..Default::default()
            },
            AppNodePort {
                id: "copy_output_1".to_string(),
                rate: 1,
                token_size: 3200,
                ..Default::default()
            },
        ],
        ..Default::default()
    });

    db.app_graph.nodes.push(AppNode {
        id: "gx".to_string(),
        func: "func_gx".to_string(),
        executable: "examples/sobel/gx".to_string(),
        input_ports: vec![
            AppNodePort {
                id: "gx_input".to_string(),
                rate: 1,
                token_size: 3200,
                ..Default::default()
            },
        ],
        output_ports: vec![
            AppNodePort {
                id: "gx_output".to_string(),
                rate: 1,
                token_size: 12800,
                ..Default::default()
            },
        ],
        ..Default::default()
    });

    db.app_graph.nodes.push(AppNode {
        id: "gy".to_string(),
        func: "func_gy".to_string(),
        executable: "examples/sobel/gy".to_string(),
        input_ports: vec![
            AppNodePort {
                id: "gy_input".to_string(),
                rate: 1,
                token_size: 3200,
                ..Default::default()
            },
        ],
        output_ports: vec![
            AppNodePort {
                id: "gy_output".to_string(),
                rate: 1,
                token_size: 12800,
                ..Default::default()
            },
        ],
        ..Default::default()
    });


    db.app_graph.nodes.push(AppNode {
        id: "combine".to_string(),
        func: "func_combine".to_string(),
        executable: "examples/sobel/combine".to_string(),
        input_ports: vec![
            AppNodePort {
                id: "combine_input_0".to_string(),
                rate: 1,
                token_size: 12800,
                ..Default::default()
            },
            AppNodePort {
                id: "combine_input_1".to_string(),
                rate: 1,
                token_size: 12800,
                ..Default::default()
            },
        ],
        output_ports: vec![
            AppNodePort {
                id: "combine_output".to_string(),
                rate: 1,
                token_size: 3200,
                ..Default::default()
            },
        ],
        ..Default::default()
    });

    db.app_graph.nodes.push(AppNode {
        id: "store".to_string(),
        func: "func_store".to_string(),
        executable: "examples/sobel/store".to_string(),
        input_ports: vec![
            AppNodePort {
                id: "store_input".to_string(),
                rate: 1,
                token_size: 3200,
                ..Default::default()
            },
        ],
        ..Default::default()
    });

    db.app_graph.edges.push(AppEdge {
        id: "edge_load_copy".to_string(),
        source_node: "load".to_string(),
        target_node: "copy".to_string(),
        source_port: "load_output".to_string(),
        target_port: "copy_input".to_string(),
        token_size: 3200,
        ..Default::default()
    });
    
    db.app_graph.edges.push(AppEdge {
        id: "edge_copy_gx".to_string(),
        source_node: "copy".to_string(),
        target_node: "gx".to_string(),
        source_port: "copy_output_0".to_string(),
        target_port: "gx_input".to_string(),
        token_size: 3200,
        ..Default::default()
    });
    
    db.app_graph.edges.push(AppEdge {
        id: "edge_copy_gy".to_string(),
        source_node: "copy".to_string(),
        target_node: "gy".to_string(),
        source_port: "copy_output_1".to_string(),
        target_port: "gy_input".to_string(),
        token_size: 3200,
        ..Default::default()
    });
    
    db.app_graph.edges.push(AppEdge {
        id: "edge_gx_combine".to_string(),
        source_node: "gx".to_string(),
        target_node: "combine".to_string(),
        source_port: "gx_output".to_string(),
        target_port: "combine_input_0".to_string(),
        token_size: 12800,
        ..Default::default()
    });
    
    db.app_graph.edges.push(AppEdge {
        id: "edge_gy_combine".to_string(),
        source_node: "gy".to_string(),
        target_node: "combine".to_string(),
        source_port: "gy_output".to_string(),
        target_port: "combine_input_1".to_string(),
        token_size: 12800,
        ..Default::default()
    });
    
    db.app_graph.edges.push(AppEdge {
        id: "edge_combine_store".to_string(),
        source_node: "combine".to_string(),
        target_node: "store".to_string(),
        source_port: "combine_output".to_string(),
        target_port: "store_input".to_string(),
        token_size: 3200,
        ..Default::default()
    });

    db.app_graph.global_mem_image = "examples/sobel/mem/global_mem_image.json".to_string();
    db.app_graph.global_mem_reference = "examples/sobel/mem/global_mem_reference.json".to_string();

    Ok(())
}
