use sv_lib::{file_handler};
use sv_lib::{setup};
use log::{info, error};
use clap::Parser;

mod memsyn;
mod route;
mod noc;


/// Arguments to get the configuration files and output directory 
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 'i', long = "intermediate-representation", help="Input Intermediate Representation Object")]
    ir_object: String,

    #[arg(short = 'o', long = "binary", help="output directory")]
    output: String,
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let log_file = format!("{}/log_asm.log", args.output);
    setup::setup_logger(&log_file).unwrap();
    
    // read configuration files and store the information in ds 
    let mut db = file_handler::load_json_file(&args.ir_object)?;
    
    // This will create the output directory if it doesn't exist
    match std::fs::create_dir_all(&args.output) {
        Ok(_) => (),
        Err(e) => {       
            error!("Failed to create {} with {}", args.output, e);
            std::process::exit(1);
        },
    };    

    /* run the compilation */
    info!("Sylva Assembly starts compilation!");

    memsyn::run(&mut db, &args.output)?;
    route::run(&mut db, &args.output)?;
    noc::run(&mut db, &args.output)?;
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
