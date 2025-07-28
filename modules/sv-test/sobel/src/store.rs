use sv_lib::sim::{MemoryList, Memory};
use sv_lib::file_handler::{load_json_file, write_json_file};
use clap::Parser;


// Arguments 
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long = "global-image", help="path to global memory image")]
    global_image: String,

    #[arg(long = "in-mem", help="path to input memory")]
    in_mem: String,
    
    #[arg(long = "out-mem", help="path to output memory")]
    out_mem: Option<String>,
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut global_memory: MemoryList = load_json_file(&args.global_image)?; 
    let input: MemoryList = load_json_file(&args.in_mem)?; 
    
    let mut next_address = global_memory.line
        .iter()
        .map(|m| m.address)
        .max()
        .unwrap_or(0) + 1;
    
    for chunk in &input.line {
        global_memory.line.push(Memory {
            address: next_address,
            value: chunk.value.clone(),
        });
        next_address += 1;
    }

    write_json_file(&args.global_image, &global_memory)?;

    let program_name = std::env::args().next().unwrap_or_else(|| "<program>".to_string());
    println!("{} completed", program_name);

    Ok(())
}

