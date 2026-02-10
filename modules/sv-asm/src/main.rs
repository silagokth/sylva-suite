use sv_lib::{file_handler};
use sv_lib::{utils};
use log::{info, error};
use clap::Parser;

mod memsyn;
mod route;
mod noc;
mod tlb;
mod transporter;


/// Arguments to get the configuration files and output directory 
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long = "cpu", help="Set CPU limit")]
    cpu_limit: Option<u64>,

    #[arg(long = "memory", help="Set memory limit in GB")]
    memory_limit: Option<u64>,

    #[arg(short = 'i', long = "intermediate-representation", help="Input Intermediate Representation Object")]
    ir_object: String,

    #[arg(short = 'o', long = "binary", help="output directory")]
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

    let log_file = format!("{}/log_asm.log", args.output);
    utils::setup_logger(&log_file).unwrap();
 
    if let Some(n) = args.cpu_limit {
        utils::set_cpu_limit(n)?;
    }
    
    if let Some(n) = args.memory_limit {
        utils::set_memory_limit(n)?;
    }   
    
    // read configuration files and store the information in ds 
    let mut db = file_handler::load_json_file(&args.ir_object)?;
    
    /* run the compilation */
    info!("Sylva Assembly starts compilation!");

    memsyn::run(&mut db, &args.output)?;
    route::run(&mut db, &args.output)?;
    noc::run(&mut db, &args.output)?;
    tlb::run(&mut db, &args.output)?;
    transporter::run(&mut db, &args.output)?;
    // Transporter instructions:
    // Simulation
    // Assembler

    /* save synthesized information */
    let bin_file = format!("{}/db_asm.bin", args.output);
    file_handler::write_json_file(&bin_file, &db)?;
    //db = file_handler::load_json_file(&bin_file)?;
    
    info!("Sylva Assembly finished successfully!");
    Ok(())
}
