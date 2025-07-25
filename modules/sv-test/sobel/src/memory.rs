use sv_lib::sim::{MemoryList, Memory};
use sv_lib::file_handler::{write_json_file};
use clap::Parser;


// Arguments 
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long = "image", help="global memory image")]
    image: String,
    
    #[arg(long = "reference", help="global memory reference")]
    reference: String,
}




fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Simulated example memory values
    let memory_image = vec![0, 100, 200, 300, 400, 500, 600, 700, 800, 900, 1000, 1100, 1200, 1300, 1400, 1500];
    
    // Create memory_reference as memory_image 
    let mut memory_reference = Vec::new();
    memory_reference.extend_from_slice(&memory_image);
    memory_reference.extend_from_slice(&memory_image);

    let mut output_image = MemoryList { line: Vec::new() };  
    for i in 0..memory_image.len() {
        output_image.line.push(Memory {
            address: i as i64,
            value: format!("{:0>64}", memory_image[i]),
        }); 
    }

    let mut output_reference = MemoryList { line: Vec::new() };  
    for i in 0..memory_reference.len() {
        output_reference.line.push(Memory {
            address: i as i64,
            value: format!("{:0>64}", memory_reference[i]),
        }); 
    }

    write_json_file(&args.image, &output_image)?;
    write_json_file(&args.reference, &output_reference)?;

    Ok(())
}

