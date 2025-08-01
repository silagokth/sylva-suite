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

    let input: MemoryList = load_json_file(&args.in_mem)?; 
   
    let minimal_value = input.line
        .iter()
        .find(|entry| entry.address == 0) 
        .map(|entry| entry.value.clone())
        .expect("Missing address 0 in input memory");
    
    let mut global_memory: MemoryList = load_json_file(&args.global_image)?; 
    
    // Look for address 16 and update or insert
    match global_memory.line.iter_mut().find(|entry| entry.address == 16) {
        Some(entry) => {
            entry.value = minimal_value.clone();
        }
        None => {
            global_memory.line.push(Memory {
                address: 16,
                value: minimal_value.clone(),
            });
        }
    }

    write_json_file(&args.global_image, &global_memory)?;

    let program_name = std::env::args().next().unwrap_or_else(|| "<program>".to_string());
    println!("{} completed", program_name);

    Ok(())
}

