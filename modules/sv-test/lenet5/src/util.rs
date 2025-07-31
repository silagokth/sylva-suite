use sv_lib::sim::{MemoryList, Memory};
use sv_lib::file_handler::{load_json_file};
use ndarray::{Array2, ArrayD};
use std::{fs::File, io::{Write}};
use std::error::Error;



// Convert hex string to vector of f64 values
fn hex_string_to_float_vector(hex_string: &str) -> Result<Vec<f64>, Box<dyn Error>> {
    if hex_string.len() != 64 {
        return Err("Hex string must be 64 characters".into());
    }

    let mut floats = Vec::new();
    for i in (0..64).step_by(4) {
        let hex_part = &hex_string[i..i+4];
        let raw_val = u16::from_str_radix(hex_part, 16)?;
        let int_val = if raw_val > 32767 {
            raw_val as i32 - 65536
        } else {
            raw_val as i32
        };
        floats.push(int_val as f64 / 100.0);
    }

    Ok(floats)
}


// Convert float vector to hex string
fn float_vector_to_hex_string(vector: &[f64]) -> String {
    let mut hex_string = String::new();
    for &val in vector {
        let scaled = (val * 100.0).round() as i16;
        let int_val = if scaled < 0 {
            (scaled as i32 + 65536) as u16
        } else {
            scaled as u16
        };
        hex_string.push_str(&format!("{:04x}", int_val));
    }
    hex_string
}



pub fn mat2mem(matrix: &ArrayD<f64>) -> Result<Vec<[f64; 16]>, Box<dyn Error>> {
    let shape = matrix.shape();

    if shape.len() < 1 {
        return Err("Matrix shape length is less than 1".into());
    }
    
    let (row, col, reshaped): (usize, usize, Array2<f64>) = if shape.len() == 1 {
        let col = shape[0];
        let reshaped = matrix.clone().into_shape((1, col))?; 
        (1, col, reshaped)
    } else {
        let col = *shape.last().unwrap();
        let row: usize = shape[..shape.len() - 1].iter().product();
        let reshaped = matrix.clone().into_shape((row, col))?;
        (row, col, reshaped)
    };

    let storage_row_per_image_row = (col + 15) / 16;
    let mut memory = Vec::new();

    for i in 0..row {
        let mut image_row = reshaped.row(i).to_vec();
        image_row.resize(storage_row_per_image_row * 16, 0.0);
        for j in 0..storage_row_per_image_row {
            let mut chunk = [0.0f64; 16];
            chunk.copy_from_slice(&image_row[j * 16..j * 16 + 16]);
            memory.push(chunk);
        }
    }

    Ok(memory)
}



pub fn mem2mat(memory: &Vec<Vec<f64>>, shape: &[usize]) -> Result<ArrayD<f64>, Box<dyn Error>> {
    if shape.len() < 1 {
        return Err("Shape length is less than 1".into());
    }

    let (row, col) = if shape.len() == 1 {
        (1, shape[0])
    } else {
        let col = *shape.last().unwrap();
        let row: usize = shape[..shape.len() - 1].iter().product();
        (row, col)
    };

    let storage_row_per_image_row = (col + 15) / 16;

    if memory.len() != row * storage_row_per_image_row {
        return Err(format!(
            "Memory length mismatch: expected {}, got {}",
            row * storage_row_per_image_row,
            memory.len()
        )
        .into());
    }

    let mut matrix = Array2::<f64>::zeros((row, col));

    for i in 0..row {
        let mut row_data = vec![0.0f64; col];
        for j in 0..storage_row_per_image_row {
            let index = i * storage_row_per_image_row + j;
            let chunk = &memory[index];

            if chunk.len() != 16 {
                return Err(format!("Expected each row to be of length 16, got {}", chunk.len()).into());
            }

            let start = j * 16;
            let end = (start + 16).min(col);
            for k in start..end {
                row_data[k] = chunk[k - start];
            }
        }
        matrix.row_mut(i).assign(&ndarray::Array1::from(row_data));
    }

    Ok(matrix.into_shape(shape)?)
}


// Convert MemoryList to Vec<Vec<f64>>
pub fn json_to_fpmem(file_path: &str) -> Result<Vec<Vec<f64>>, Box<dyn Error>> {
    let content: MemoryList = load_json_file(file_path)?;
    let mut result = Vec::new();
    for mem in &content.line {
        let vec = hex_string_to_float_vector(&mem.value)?;
        result.push(vec);
    }
    Ok(result)
}


// Convert Vec<Vec<f64>> to MemoryList and write to JSON
pub fn fpmem_to_json(memory: &Vec<Vec<f64>>, file_path: &str) -> Result<(), Box<dyn Error>> {
    let mut lines = Vec::new();
    for (i, vec) in memory.iter().enumerate() {
        let hex_str = float_vector_to_hex_string(vec);
        lines.push(Memory {
            address: i as i64,
            value: hex_str,
        });
    }

    let mem_list = MemoryList { line: lines };
    let json = serde_json::to_string_pretty(&mem_list)?;
    let mut file = File::create(file_path)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}


