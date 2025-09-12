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
    in_mem: Option<String>,
    
    #[arg(long = "out-mem", help="path to output memory")]
    out_mem: String,
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let global_memory: MemoryList = load_json_file(&args.global_image)?; 
    
    let mut output = MemoryList { line: Vec::new() };     
    
    for i in 0..16 {
        let value = global_memory.line
            .iter()
            .find(|entry| entry.address == i)
            .map(|entry| entry.value.clone())
            .expect(&format!("Cannot find an entry for address {}", i));

        output.line.push(Memory {
            address: i,
            value,
        }); 
    }

    write_json_file(&args.out_mem, &output)?;

    let program_name = std::env::args().next().unwrap_or_else(|| "<program>".to_string());
    println!("{} completed", program_name);

    Ok(())
}

