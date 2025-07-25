use sv_lib::model::*;
use sv_lib::file_handler;
use clap::Parser;


// Arguments 
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 'n', long = "name", help="name of the generating example")]
    name: String,

    #[arg(short = 'o', long = "output", help="output directory")]
    dir: String,
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    let mut db = DataBase::new();
    
    match args.name.as_str() {
        "minimum" => minimum(&mut db)?,
        _ => {
            eprintln!("Unknown example name: {}", args.name);
            std::process::exit(1);
        }
    }
    add_common_technology(&mut db)?;

    save_files(&db, &args.dir)?;

    Ok(())
}


fn save_files(db: &DataBase, dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    // ensure the directory exists
    std::fs::create_dir_all(dir)?; 
    
    fn write_json<T: serde::Serialize>(dir: &str, filename: &str, content: &T) -> Result<(), Box<dyn std::error::Error>> {
        let path = format!("{}/{}", dir, filename);
        file_handler::write_json_file(&path, content)
    }

    write_json(dir, "app_graph.json", &db.app_graph)?;
    write_json(dir, "global_constraint.json", &db.global_constraint)?;
    write_json(dir, "alimp_lib.json", &db.alimp_lib)?;
    write_json(dir, "hyper_parameter.json", &db.hyper_parameter)?;
    write_json(dir, "technology_constraint.json", &db.technology_constraint)?;
    
    Ok(())
}


fn add_common_technology(db: &mut DataBase) -> Result<(), Box<dyn std::error::Error>> {
    
    db.technology_constraint.required_period = 4.0e-9;
    db.technology_constraint.required_slew = 0.4;
    db.technology_constraint.initial_slew = 0.7;
    db.technology_constraint.buffer_slew_declined_factor = 0.05;
    db.technology_constraint.buffer_delay_improved_factor = 0.6e-9;
    db.technology_constraint.register_slew_constant = 1.0;
    
    // slew rates in steps of 0.05 from 0.40 to 1.85
    db.technology_constraint.slew_rates = (40..=185)
        .step_by(5)
        .map(|v| v as f64 / 100.0)
        .collect();   

    // timing table rows (30 columns each, 30 rows total)
    db.technology_constraint.timing_table = vec![
            vec![1000,1300,1600,1900,2200,2500,2800,3100,3400,3700,4000,4300,4600,4900,5200,5500,5800,6100,6400,6700,7000,7300,7600,7900,8200,8500,8800,9100,9400,9700],
            vec![970,1270,1570,1870,2170,2470,2770,3070,3370,3670,3970,4270,4570,4870,5170,5470,5770,6070,6370,6670,6970,7270,7570,7870,8170,8470,8770,9070,9370,9670],
            vec![940,1240,1540,1840,2140,2440,2740,3040,3340,3640,3940,4240,4540,4840,5140,5440,5740,6040,6340,6640,6940,7240,7540,7840,8140,8440,8740,9040,9340,9640],
            vec![910,1210,1510,1810,2110,2410,2710,3010,3310,3610,3910,4210,4510,4810,5110,5410,5710,6010,6310,6610,6910,7210,7510,7810,8110,8410,8710,9010,9310,9610],
            vec![880,1180,1480,1780,2080,2380,2680,2980,3280,3580,3880,4180,4480,4780,5080,5380,5680,5980,6280,6580,6880,7180,7480,7780,8080,8380,8680,8980,9280,9580],
            vec![850,1150,1450,1750,2050,2350,2650,2950,3250,3550,3850,4150,4450,4750,5050,5350,5650,5950,6250,6550,6850,7150,7450,7750,8050,8350,8650,8950,9250,9550],
            vec![820,1120,1420,1720,2020,2320,2620,2920,3220,3520,3820,4120,4420,4720,5020,5320,5620,5920,6220,6520,6820,7120,7420,7720,8020,8320,8620,8920,9220,9520],
            vec![790,1090,1390,1690,1990,2290,2590,2890,3190,3490,3790,4090,4390,4690,4990,5290,5590,5890,6190,6490,6790,7090,7390,7690,7990,8290,8590,8890,9190,9490],
            vec![760,1060,1360,1660,1960,2260,2560,2860,3160,3460,3760,4060,4360,4660,4960,5260,5560,5860,6160,6460,6760,7060,7360,7660,7960,8260,8560,8860,9160,9460],
            vec![730,1030,1330,1630,1930,2230,2530,2830,3130,3430,3730,4030,4330,4630,4930,5230,5530,5830,6130,6430,6730,7030,7330,7630,7930,8230,8530,8830,9130,9430],
            vec![700,1000,1300,1600,1900,2200,2500,2800,3100,3400,3700,4000,4300,4600,4900,5200,5500,5800,6100,6400,6700,7000,7300,7600,7900,8200,8500,8800,9100,9400],
            vec![670,970,1270,1570,1870,2170,2470,2770,3070,3370,3670,3970,4270,4570,4870,5170,5470,5770,6070,6370,6670,6970,7270,7570,7870,8170,8470,8770,9070,9370],
            vec![640,940,1240,1540,1840,2140,2440,2740,3040,3340,3640,3940,4240,4540,4840,5140,5440,5740,6040,6340,6640,6940,7240,7540,7840,8140,8440,8740,9040,9340],
            vec![610,910,1210,1510,1810,2110,2410,2710,3010,3310,3610,3910,4210,4510,4810,5110,5410,5710,6010,6310,6610,6910,7210,7510,7810,8110,8410,8710,9010,9310],
            vec![580,880,1180,1480,1780,2080,2380,2680,2980,3280,3580,3880,4180,4480,4780,5080,5380,5680,5980,6280,6580,6880,7180,7480,7780,8080,8380,8680,8980,9280],
            vec![550,850,1150,1450,1750,2050,2350,2650,2950,3250,3550,3850,4150,4450,4750,5050,5350,5650,5950,6250,6550,6850,7150,7450,7750,8050,8350,8650,8950,9250],
            vec![520,820,1120,1420,1720,2020,2320,2620,2920,3220,3520,3820,4120,4420,4720,5020,5320,5620,5920,6220,6520,6820,7120,7420,7720,8020,8320,8620,8920,9220],
            vec![490,790,1090,1390,1690,1990,2290,2590,2890,3190,3490,3790,4090,4390,4690,4990,5290,5590,5890,6190,6490,6790,7090,7390,7690,7990,8290,8590,8890,9190],
            vec![460,760,1060,1360,1660,1960,2260,2560,2860,3160,3460,3760,4060,4360,4660,4960,5260,5560,5860,6160,6460,6760,7060,7360,7660,7960,8260,8560,8860,9160],
            vec![430,730,1030,1330,1630,1930,2230,2530,2830,3130,3430,3730,4030,4330,4630,4930,5230,5530,5830,6130,6430,6730,7030,7330,7630,7930,8230,8530,8830,9130],
            vec![400,700,1000,1300,1600,1900,2200,2500,2800,3100,3400,3700,4000,4300,4600,4900,5200,5500,5800,6100,6400,6700,7000,7300,7600,7900,8200,8500,8800,9100],
            vec![370,670,970,1270,1570,1870,2170,2470,2770,3070,3370,3670,3970,4270,4570,4870,5170,5470,5770,6070,6370,6670,6970,7270,7570,7870,8170,8470,8770,9070],
            vec![340,640,940,1240,1540,1840,2140,2440,2740,3040,3340,3640,3940,4240,4540,4840,5140,5440,5740,6040,6340,6640,6940,7240,7540,7840,8140,8440,8740,9040],
            vec![310,610,910,1210,1510,1810,2110,2410,2710,3010,3310,3610,3910,4210,4510,4810,5110,5410,5710,6010,6310,6610,6910,7210,7510,7810,8110,8410,8710,9010],
            vec![280,580,880,1180,1480,1780,2080,2380,2680,2980,3280,3580,3880,4180,4480,4780,5080,5380,5680,5980,6280,6580,6880,7180,7480,7780,8080,8380,8680,8980],
            vec![250,550,850,1150,1450,1750,2050,2350,2650,2950,3250,3550,3850,4150,4450,4750,5050,5350,5650,5950,6250,6550,6850,7150,7450,7750,8050,8350,8650,8950],
            vec![220,520,820,1120,1420,1720,2020,2320,2620,2920,3220,3520,3820,4120,4420,4720,5020,5320,5620,5920,6220,6520,6820,7120,7420,7720,8020,8320,8620,8920],
            vec![190,490,790,1090,1390,1690,1990,2290,2590,2890,3190,3490,3790,4090,4390,4690,4990,5290,5590,5890,6190,6490,6790,7090,7390,7690,7990,8290,8590,8890],
            vec![160,460,760,1060,1360,1660,1960,2260,2560,2860,3160,3460,3760,4060,4360,4660,4960,5260,5560,5860,6160,6460,6760,7060,7360,7660,7960,8260,8560,8860],
            vec![130,430,730,1030,1330,1630,1930,2230,2530,2830,3130,3430,3730,4030,4330,4630,4930,5230,5530,5830,6130,6430,6730,7030,7330,7630,7930,8230,8530,8830],
        ]
        .into_iter()
        .map(|row| TimingRow {
            rows: row.into_iter().map(|val| val as f64 * 1e-12).collect(),
        })
        .collect();

    Ok(())
}


fn minimum(db: &mut DataBase) -> Result<(), Box<dyn std::error::Error>> {
    
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
            AlimpInstance { width: 2, height: 2, energy: 1, latency: 16,
                input_addr_time_patterns: vec![],
                output_addr_time_patterns: vec![PairIntInt { key: 0, value: 0 }, PairIntInt { key: 1, value: 1 }, PairIntInt { key: 2, value: 2 }, PairIntInt { key: 3, value: 3 }, PairIntInt { key: 4, value: 4 }, PairIntInt { key: 5, value: 5 }, PairIntInt { key: 6, value: 6 }, PairIntInt { key: 7, value: 7 }, PairIntInt { key: 8, value: 8 }, PairIntInt { key: 9, value: 9 }, PairIntInt { key: 10, value: 10 }, PairIntInt { key: 11, value: 11 }, PairIntInt { key: 12, value: 12 }, PairIntInt { key: 13, value: 13 }, PairIntInt { key: 14, value: 14 }, PairIntInt { key: 15, value: 15 }],
                ..Default::default()
            },
        ],
    });
    
    db.alimp_lib.entries.push(AlimpEntry {
        func: "FB".to_string(),
        instances: vec![
            AlimpInstance { width: 1, height: 1, energy: 1, latency: 32,
                input_addr_time_patterns: vec![PairIntInt { key: 0, value: 0 }, PairIntInt { key: 1, value: 1 }, PairIntInt { key: 2, value: 2 }, PairIntInt { key: 3, value: 3 }, PairIntInt { key: 4, value: 4 }, PairIntInt { key: 5, value: 5 }, PairIntInt { key: 6, value: 6 }, PairIntInt { key: 7, value: 7 }, PairIntInt { key: 8, value: 8 }, PairIntInt { key: 9, value: 9 }, PairIntInt { key: 10, value: 10 }, PairIntInt { key: 11, value: 11 }, PairIntInt { key: 12, value: 12 }, PairIntInt { key: 13, value: 13 }, PairIntInt { key: 14, value: 14 }, PairIntInt { key: 15, value: 15 }],
                output_addr_time_patterns: vec![PairIntInt { key: 0, value: 0 }, PairIntInt { key: 1, value: 1 }, PairIntInt { key: 2, value: 2 }, PairIntInt { key: 3, value: 3 }, PairIntInt { key: 4, value: 4 }, PairIntInt { key: 5, value: 5 }, PairIntInt { key: 6, value: 6 }, PairIntInt { key: 7, value: 7 }, PairIntInt { key: 8, value: 8 }, PairIntInt { key: 9, value: 9 }, PairIntInt { key: 10, value: 10 }, PairIntInt { key: 11, value: 11 }, PairIntInt { key: 12, value: 12 }, PairIntInt { key: 13, value: 13 }, PairIntInt { key: 14, value: 14 }, PairIntInt { key: 15, value: 15 }],
                ..Default::default()
            },
        ],
    });
    
    db.alimp_lib.entries.push(AlimpEntry {
        func: "FC".to_string(),
        instances: vec![
            AlimpInstance { width: 4, height: 4, energy: 10, latency: 16,
                input_addr_time_patterns: vec![PairIntInt { key: 0, value: 0 }, PairIntInt { key: 1, value: 1 }, PairIntInt { key: 2, value: 2 }, PairIntInt { key: 3, value: 3 }, PairIntInt { key: 4, value: 4 }, PairIntInt { key: 5, value: 5 }, PairIntInt { key: 6, value: 6 }, PairIntInt { key: 7, value: 7 }, PairIntInt { key: 8, value: 8 }, PairIntInt { key: 9, value: 9 }, PairIntInt { key: 10, value: 10 }, PairIntInt { key: 11, value: 11 }, PairIntInt { key: 12, value: 12 }, PairIntInt { key: 13, value: 13 }, PairIntInt { key: 14, value: 14 }, PairIntInt { key: 15, value: 15 }],
                output_addr_time_patterns: vec![],
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




