use clap::Parser;

mod libs;
use libs::{MemoryList, Memory};


// Arguments 
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long = "global-image", help="path to global memory image")]
    global_image: Option<String>,

    #[arg(long = "in-mem", help="path to input memory")]
    in_mem: String,
    
    #[arg(long = "out-mem", help="path to output memory")]
    out_mem: String,
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let input: MemoryList = libs::load_json_file(&args.in_mem)?; 
    
    let mut data = Vec::new()
    for mem in &input.line {
        data.push(mem.value.parse::<i64>.expect("Failed to parse integer"));
    }

    // sort value 
    data.sort();  
    
    let mut output = MemoryList { line: Vec::new() };     
    for i in 0..data.len() {
        output.push(Memory{
            address: i,
            value: format!("{:0>64}", value),
        });
    }

    libs::write_json_file(&args.out_mem, &output);

    let program_name = std::env::args().next().unwrap_or_else(|| "<program>".to_string());
    println!("{} completed", program_name);

    Ok(())
}

