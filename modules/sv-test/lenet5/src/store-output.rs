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

    #[arg(long = "addr", help="starting addresss")]
    addr: i32,

    #[arg(long = "size", help="memory size")]
    size: i32
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut global_memory: MemoryList = load_json_file(&args.global_image)?; 
    let input: MemoryList = load_json_file(&args.in_mem)?;     
    
    for i in args.addr..args.addr+args.size {
        let value = input.line
            .iter()
            .find(|entry| entry.address == (i - args.addr) as i64)
            .map(|entry| entry.value.clone())
            .expect(&format!("Cannot find an entry for address {}", i - args.addr));

        match global_memory.line.iter_mut().find(|entry| entry.address == i as i64) {
            Some(entry) => {
                entry.value = value;
            }
            None => {
                global_memory.line.push(Memory {
                    address: i as i64,
                    value: value,
                });
            }
        }
    }

    write_json_file(&args.global_image, &global_memory)?;

    Ok(())
}

