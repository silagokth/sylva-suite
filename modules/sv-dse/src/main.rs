use sv_lib::model::{DataBase};
use sv_lib::{utils};
use sv_lib::{file_handler};
use log::{info, error};
use clap::Parser;

mod bind;
mod place;
mod route;
mod noc;
mod glic;
mod sim;


/// Arguments to get the configuration files and output directory 
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long = "cpu", help="Set CPU limit")]
    cpu_limit: Option<u64>,

    #[arg(long = "memory", help="Set memory limit")]
    memory_limit: Option<u64>,

    #[arg(short = 'g', long = "graph", help="SDF graph file")]
    graph: String,

    #[arg(short = 'c', long = "constraint", help="global constraint file")]
    constraint_file: String,

    #[arg(short = 'l', long = "library", help="alimp library file")]
    alimp_lib: String,

    #[arg(short = 'p', long = "parameter", help="hyper parameter file")]
    hyper_parameter: String,

    #[arg(short = 't', long = "technology", help="technology constraint file")]
    technology_constraint: String,

    #[arg(short = 'o', long = "output", help="output directory")]
    output: String,
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    // This will create the output directory if it doesn't exist
    match std::fs::create_dir_all(&args.output) {
        Ok(_) => (),
        Err(e) => {       
            error!("Failed to create {} with {}", args.output, e);
            std::process::exit(1);
        },
    };

    let log_file = format!("{}/log_dse.log", args.output);
    utils::setup_logger(&log_file).unwrap();
  
    if let Some(n) = args.cpu_limit {
        utils::set_cpu_limit(n)?;
    }
    
    if let Some(n) = args.memory_limit {
        utils::set_memory_limit(n)?;
    }

    // create empty data structure 
    let mut db = DataBase::new();

    // read configuration files and store the information in ds 
    db.app_graph = file_handler::load_json_file(&args.graph)?;
    db.global_constraint = file_handler::load_json_file(&args.constraint_file)?;
    db.alimp_lib = file_handler::load_json_file(&args.alimp_lib)?;
    db.hyper_parameter = file_handler::load_json_file(&args.hyper_parameter)?;
    db.technology_constraint = file_handler::load_json_file(&args.technology_constraint)?;
        

    /* run the compilation */
    info!("Sylva DSE starts compilation!");

    bind::run(&mut db, &args.output)?;
    place::run(&mut db, &args.output)?;
    route::run(&mut db, &args.output)?;
    noc::run(&mut db, &args.output)?;
    glic::run(&mut db, &args.output)?;
  
    /* save synthesized information */
    let bin_file = format!("{}/db.bin", args.output);
    file_handler::write_json_file(&bin_file, &db)?;
    db = file_handler::load_json_file(&bin_file)?;
    
    let sim = sim::run(&mut db, &args.output)?;
    if !sim {
        error!("Failed to verify the simulation");
        std::process::exit(1);
    }

    info!("Sylva DSE finished successfully!");
    Ok(())
}
