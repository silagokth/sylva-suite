use sv_lib::model::*;
use rand::Rng;
use rand::rngs::StdRng;
use rand::SeedableRng;
use rand::prelude::SliceRandom;

fn assign_address_patterns<FMin, FMax, R>(
    rng: &mut R,
    number_addresses: i32, 
    number_channels: usize,
    min_time_fn: FMin,
    max_time_fn: FMax,
) -> Result<Vec<AddressPatterns>, Box <dyn std::error::Error>> 
where 
    FMin: Fn(i32) -> i32,
    FMax: Fn(i32) -> i32,
    R: Rng,
{
    let mut patterns: Vec<AddressPatterns> = vec![];
    
    for i in 0..number_addresses {
        let min_time = min_time_fn(i);
        let max_time = max_time_fn(i);
        
        let mut free_slots = Vec::new();

        for t in min_time..=max_time {
            let mut used = vec![false; number_channels as usize];
            for p in patterns.iter().filter(|p| p.time == t) {
                used[p.channel as usize] = true;
            }
            for c in 0..number_channels {
                if !used[c as usize] {
                    free_slots.push((t, c));
                }
            }
        }

        if let Some(&(t, c)) = free_slots.choose(rng) {
            patterns.push(AddressPatterns {
                address: i,
                channel: c as i32,
                time: t,
            });
        } else {
            return Err("No free slot available in the given time/channel range".into())
        }
    }
    
    Ok(patterns)
}


pub fn sobel_random(db: &mut DataBase) -> Result<(), Box <dyn std::error::Error>> {
 
    println!("This example creates a very large problem size and is not yet tested");

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

    let mut rng = StdRng::seed_from_u64(100);

    let mut input_patterns: Vec<AddressPatterns> = vec![];
    let mut output_patterns: Vec<AddressPatterns> = assign_address_patterns(
        &mut rng,
        3200,
        2,
        |i| 10 + i / 10 * 5, 
        |i| 14 + i / 10 * 5, 
    )?;
    
    db.alimp_lib.entries.push(AlimpEntry {
        func: "func_load".to_string(),
        instances: vec![
            AlimpInstance { 
                width: 2, 
                height: 4, 
                energy: 4, 
                latency: std::cmp::max(
                    input_patterns.iter().map(|p| p.time).max().unwrap_or(0), 
                    output_patterns.iter().map(|p| p.time).max().unwrap_or(0)
                    ) + 1,
                input_addr_time_patterns: input_patterns.clone(),
                output_addr_time_patterns: output_patterns.clone(),
                ..Default::default()
            },
        ],
    });

    input_patterns = assign_address_patterns(
        &mut rng,
        3200,
        2,
        |i| i / 10 * 5, 
        |i| 4 + i / 10 * 5, 
    )?;
    output_patterns = assign_address_patterns(
        &mut rng,
        6400,
        4,
        |i| 20 + (i % 3200) / 10 * 5, 
        |i| 24 + (i % 3200) / 10 * 5, 
    )?;

    db.alimp_lib.entries.push(AlimpEntry {
        func: "func_copy".to_string(),
        instances: vec![
            AlimpInstance { 
                width: 4, 
                height: 1, 
                energy: 5, 
                latency: std::cmp::max(
                    input_patterns.iter().map(|p| p.time).max().unwrap_or(0), 
                    output_patterns.iter().map(|p| p.time).max().unwrap_or(0)
                    ) + 1,
                input_addr_time_patterns: input_patterns.clone(),
                output_addr_time_patterns: output_patterns.clone(),
                ..Default::default()
            },
        ],
    });

    input_patterns = assign_address_patterns(
        &mut rng,
        3200,
        2,
        |i| i / 10 * 5, 
        |i| 4 + i / 10 * 5, 
    )?;

    output_patterns = assign_address_patterns(
        &mut rng,
        12800,
        8,
        |i| 100 + i / 80 * 10, 
        |i| 109 + i / 80 * 10, 
    )?;

    db.alimp_lib.entries.push(AlimpEntry {
        func: "func_gx".to_string(),
        instances: vec![
            AlimpInstance { 
                width: 8, 
                height: 6, 
                energy: 15, 
                latency: std::cmp::max(
                    input_patterns.iter().map(|p| p.time).max().unwrap_or(0), 
                    output_patterns.iter().map(|p| p.time).max().unwrap_or(0)
                    ) + 1,
                input_addr_time_patterns: input_patterns.clone(),
                output_addr_time_patterns: output_patterns.clone(),
                ..Default::default()
            },
        ],
    });

    input_patterns = assign_address_patterns(
        &mut rng,
        3200,
        2,
        |i| i / 10 * 5, 
        |i| 4 + i / 10 * 5, 
    )?;

    output_patterns = assign_address_patterns(
        &mut rng,
        12800,
        8,
        |i| 100 + i / 80 * 10, 
        |i| 109 + i / 80 * 10, 
    )?;

    db.alimp_lib.entries.push(AlimpEntry {
        func: "func_gy".to_string(),
        instances: vec![
            AlimpInstance { 
                width: 8, 
                height: 6, 
                energy: 15, 
                latency: std::cmp::max(
                    input_patterns.iter().map(|p| p.time).max().unwrap_or(0), 
                    output_patterns.iter().map(|p| p.time).max().unwrap_or(0)
                    ) + 1,
                input_addr_time_patterns: input_patterns.clone(),
                output_addr_time_patterns: output_patterns.clone(),
                ..Default::default()
            },
        ],
    });

    input_patterns = assign_address_patterns(
        &mut rng,
        25600,
        16,
        |i| (i % 12800) / 80 * 10, 
        |i| 9 + (i % 12800) / 80 * 10, 
    )?;

    output_patterns = assign_address_patterns(
        &mut rng,
        3200,
        8,
        |i| 100 + i / 10 * 5, 
        |i| 104 + i / 10 * 5, 
    )?;

    db.alimp_lib.entries.push(AlimpEntry {
        func: "func_combine".to_string(),
        instances: vec![
            AlimpInstance { 
                width: 16, 
                height: 12, 
                energy: 18, 
                latency: std::cmp::max(
                    input_patterns.iter().map(|p| p.time).max().unwrap_or(0), 
                    output_patterns.iter().map(|p| p.time).max().unwrap_or(0)
                    ) + 1,
                input_addr_time_patterns: input_patterns.clone(),
                output_addr_time_patterns: output_patterns.clone(),
                ..Default::default()
            },
        ],
    });

    input_patterns = assign_address_patterns(
        &mut rng,
        3200,
        4,
        |i| i / 10 * 5, 
        |i| 4 + i / 10 * 5, 
    )?;
    output_patterns = vec![];

    db.alimp_lib.entries.push(AlimpEntry {
        func: "func_store".to_string(),
        instances: vec![
            AlimpInstance { 
                width: 4, 
                height: 3, 
                energy: 6, 
                latency: std::cmp::max(
                    input_patterns.iter().map(|p| p.time).max().unwrap_or(0), 
                    output_patterns.iter().map(|p| p.time).max().unwrap_or(0)
                    ) + 1,
                input_addr_time_patterns: input_patterns.clone(),
                output_addr_time_patterns: output_patterns.clone(),
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
