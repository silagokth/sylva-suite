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


const HEIGHT: usize = 320;
const WIDTH: usize = 320;
const CHUNK_SIZE: usize = 32; 
const TOKEN_SIZE: usize = 4 * 3200;

fn decode_image(data: &MemoryList, offset: i32) -> Result<Vec<Vec<i32>>, Box<dyn std::error::Error>> {
    if data.line.len() * (CHUNK_SIZE / 4) < HEIGHT * WIDTH {
        return Err("Insufficient pixel data".into());
    }
    
    let mut flat_pixels: Vec<i32> = Vec::with_capacity(HEIGHT * WIDTH);

    for chunk in data.line.iter() {
        if chunk.address < offset as i64 {
            continue;    
        }

        let hex_str = chunk.value.trim();
        let bytes = hex::decode(hex_str)?;
        
        if bytes.len() != CHUNK_SIZE {
            return Err(format!("Chunk size mismatch: expected {}, got {}", CHUNK_SIZE, bytes.len()).into());
        }

        for i in 0..(CHUNK_SIZE / 4) {
            let start = i * 4;
            let raw_bytes = &bytes[start..start + 4];
            let val = i32::from_le_bytes(raw_bytes.try_into()?);
            flat_pixels.push(val);
        }

        if flat_pixels.len() >= HEIGHT * WIDTH {
            break;
        }
    }

    flat_pixels.truncate(HEIGHT * WIDTH);

    let mut reshaped: Vec<Vec<i32>> = Vec::with_capacity(HEIGHT);
    for row in flat_pixels.chunks(WIDTH) {
        reshaped.push(row.to_vec());
    }

    Ok(reshaped)
}



fn encode_image(data: Vec<Vec<u8>>) -> Result<MemoryList, Box<dyn std::error::Error>> {
    let mut flat: Vec<u8> = data.into_iter().flatten().collect();

    // pad with zeros if needed
    let remainder = flat.len() % CHUNK_SIZE;
    if remainder != 0 {
        flat.extend(std::iter::repeat(0u8).take(CHUNK_SIZE - remainder));
    }

    let mut memory = MemoryList { line: Vec::new() };

    for (i, chunk) in flat.chunks(CHUNK_SIZE).enumerate() {
        let hex_string = hex::encode(chunk); // turns 32 bytes into 64-character hex
        memory.line.push(Memory {
            address: i as i64,
            value: hex_string,
        });
    }

    Ok(memory)
}



fn combine(gx: &Vec<Vec<i32>>, gy: &Vec<Vec<i32>>) -> Vec<Vec<u8>> {
    let height = gx.len();
    let width = gx[0].len();
    let mut result = vec![vec![0u8; width]; height];

    for i in 0..height {
        for j in 0..width {
            let gx_val = gx[i][j];
            let gy_val = gy[i][j];
            let magnitude = ((gx_val * gx_val + gy_val * gy_val) as f64).sqrt().round();
            result[i][j] = magnitude.clamp(0.0, 255.0) as u8;
        }
    }

    result
}



fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let input: MemoryList = load_json_file(&args.in_mem)?; 
    let gx: Vec<Vec<i32>> = decode_image(&input, 0)?;   
    let gy: Vec<Vec<i32>> = decode_image(&input, TOKEN_SIZE as i32)?;   
    let result: Vec<Vec<u8>> = combine(&gx, &gy);
    let output: MemoryList = encode_image(result)?;

    write_json_file(&args.out_mem, &output)?;

    let program_name = std::env::args().next().unwrap_or_else(|| "<program>".to_string());
    println!("{} completed", program_name);

    Ok(())
}

