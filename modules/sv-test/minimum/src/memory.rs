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
    let memory_image = vec![300, 400, 200, 150, 320, 310, 290, 410, 500, 280, 270, 260, 330, 340, 360, 370];
    
    // Create memory_reference as memory_image + minimum value
    let min_value = *memory_image.iter().min().expect("Non-empty memory image");
    let mut memory_reference = memory_image.clone();
    memory_reference.push(min_value);
   
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

