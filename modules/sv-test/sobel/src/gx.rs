use sv_lib::sim::{MemoryList, Memory};
use sv_lib::file_handler::{load_json_file, write_json_file};
use clap::Parser;


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


const HEIGHT = 320;
const WIDTH = 320;
const CHUNK_SIZE = 32; 


fn decode_image(data: &MemoryList) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
    if data.line.len() * CHUNK_SIZE < HEIGHT * WIDTH {
        return Err("Insufficient pixel data");
    }
    
    let mut flat_pixels: Vec<u8> = Vec::with_capacity(HEIGHT * WIDTH);

    for chunk in data.line.iter() {
        let hex_str = chunk.value.trim();
        let bytes = hex::decode(hex_str)?;
        
        if bytes.len() != CHUNK_SIZE {
            return Err(format!("Chunk size mismatch: expected {}, got {}", CHUNK_SIZE, bytes.len()).into());
        }
        
        flat_pixels.extend_from_slice(&bytes);
        
        if flat_pixels.len() >= HEIGHT * WIDTH {
            break;
        }
    }

    flat_pixels.truncate(HEIGHT * WIDTH);

    // reshape to Vec<Vec<u8>>
    let mut reshaped: Vec<Vec<u8>> = Vec::with_capacity(HEIGHT);
    for row in flat_pixels.chunks(WIDTH) {
        reshaped.push(row.to_vec());
    }

    Ok(reshaped)
}



fn encode_image(data: Vec<Vec<u32>>) -> Result<MemoryList, Box<dyn std::error::Error>> {
    let mut flat: Vec<u32> = data.into_iter().flatten().collect();

    // pad with zeros if needed
    let remainder = (flat.len() * 4) % CHUNK_SIZE;
    if remainder != 0 {
        flat.extend(vec![0u8; CHUNK_SIZE - remainder]);
    }

    let mut memory = MemoryList { line: Vec::new() };

    for (i, chunk) in flat.chunks(CHUNK_SIZE).enumerate() {
        let hex_string = hex::encode(chunk); // turns 32 bytes into 64-character hex
        memory.line.push(Memory {
            address: i,
            value: hex_string,
        });
    }

    Ok(memory)
}



fn gx(img: &Vec<Vec<u8>>) -> Vec<Vec<i32>> {
    let height = img.len();
    let width = img[0].len();

    let kernel: [[i32; 3]; 3] = [
        [-1, 0, 1],
        [-2, 0, 2],
        [-1, 0, 1],
    ];

    let mut gx = vec![vec![0i32; width]; height];

    for i in 1..height - 1 {
        for j in 1..width - 1 {
            let mut sum = 0i32;
            for ki in 0..3 {
                for kj in 0..3 {
                    let pixel = img[i + ki - 1][j + kj - 1] as i32;
                    sum += pixel * kernel[ki][kj];
                }
            }
            gx[i][j] = sum; 
        }
    }

    gx
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let input: MemoryList = load_json_file(&args.in_mem)?; 
    let image: Vec<Vec<u8>> = decode_image(&input)?;   
    let gx_image: Vec<Vec<i32>> = gx(&image);
    let output: MemoryList = encode_image(&gx_image)?;

    write_json_file(&args.out_mem, &output)?;

    let program_name = std::env::args().next().unwrap_or_else(|| "<program>".to_string());
    println!("{} completed", program_name);

    Ok(())
}

