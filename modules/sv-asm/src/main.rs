use sv_lib::{file_handler};
use log::{info, error};
use clap::Parser;

mod memsyn;


/// Arguments to get the configuration files and output directory 
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 'o', long = "object", help="Input database object")]
    object: String,

    #[arg(short = 'b', long = "binary", help="output binary directory")]
    output: String,
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    env_logger::init();

    // read configuration files and store the information in ds 
    let mut db = file_handler::load_json_file(&args.object)?;
    
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
    // precise routing 
    // Noc synthesis with fixed delay
    // Transporter instructions:

    /* save synthesized information */
    let bin_file = format!("{}/db_asm.bin", args.output);
    file_handler::write_json_file(&bin_file, &db)?;
    //db = file_handler::load_json_file(&bin_file)?;
    
    info!("Sylva Assembly finished successfully!");
    Ok(())
}
