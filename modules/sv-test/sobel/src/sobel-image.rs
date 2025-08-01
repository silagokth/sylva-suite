use sv_lib::sim::{MemoryList};
use sv_lib::file_handler::{load_json_file};
use image::{GrayImage, ImageBuffer, Luma};
use clap::Parser;

// Arguments 
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long = "global-image", help="path to global memory image")]
    global_image: String,
    
    #[arg(long = "output", help="path to output image")]
    output: String,
}


const HEIGHT: usize = 320;
const WIDTH: usize = 320;
const CHUNK_SIZE: usize = 32;

fn decode_image(data: &MemoryList, offset: i32) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
    if data.line.len() * CHUNK_SIZE < HEIGHT * WIDTH {
        return Err("Insufficient pixel data".into());
    }

    let mut flat_pixels: Vec<u8> = Vec::with_capacity(HEIGHT * WIDTH);

    for chunk in data.line.iter() {
        if chunk.address < offset as i64 {
            continue;
        }

        let hex_str = chunk.value.trim();
        let mut bytes = hex::decode(hex_str)?;

        if bytes.len() != CHUNK_SIZE {
            return Err(format!("Chunk size mismatch: expected {}, got {}", CHUNK_SIZE, bytes.len()).into());
        }

        bytes.reverse();
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


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let input: MemoryList = load_json_file(&args.global_image)?; 
    let image_data: Vec<Vec<u8>> = decode_image(&input, 3200)?;
    let mut img_buffer: GrayImage = ImageBuffer::new(WIDTH as u32, HEIGHT as u32);

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let pixel_value = image_data[y][x];
            img_buffer.put_pixel(x as u32, y as u32, Luma([pixel_value]));
        }
    }

    let output_path = std::path::Path::new(&args.output);
    img_buffer.save(output_path)?;

    println!("saved image to {}", args.output);

    Ok(())
}



