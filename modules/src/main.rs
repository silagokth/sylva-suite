mod model;
mod file_handler;
mod solver;

mod bind;
mod place;

use log::{info, error};
use clap::Parser;


/// Arguments to get the configuration files and output directory 
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
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
    env_logger::init();
    
    // create empty data structure 
    let mut db = model::DataBase::new();

    // read configuration files and store the information in ds 
    db.app_graph = file_handler::load_json_file(&args.graph)?;
    db.global_constraint = file_handler::load_json_file(&args.constraint_file)?;
    db.alimp_lib = file_handler::load_json_file(&args.alimp_lib)?;
    db.hyper_parameter = file_handler::load_json_file(&args.hyper_parameter)?;
    db.technology_constraint = file_handler::load_json_file(&args.technology_constraint)?;
    
    // This will create the output directory if it doesn't exist
    match std::fs::create_dir_all(&args.output) {
        Ok(_) => (),
        Err(e) => {       
            error!("Failed to create {} with {}", args.output, e);
            std::process::exit(1);
        },
    };    

    /* run the compilation */
    info!("Sylva starts compilation!");

    bind::run(&mut db, &args.output)?;
    place::run(&mut db, &args.output)?;
     

    info!("Sylva finished successfully!");
    Ok(())
}
