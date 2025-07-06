use std::fs;
use serde_json::{from_str, to_string_pretty};
use serde::Serialize;
use serde::de::DeserializeOwned;

/// Loads and deserializes a JSON file into a generic Rust type, with filename-aware error reporting.
pub fn load_json_file<T>(filename: &str) -> Result<T, Box<dyn std::error::Error>>
where
    T: DeserializeOwned,
{
    let file_content = fs::read_to_string(filename)
        .map_err(|e| Box::<dyn std::error::Error>::from(format!("Error reading file '{}': {}", filename, e)))?;
    let data: T = from_str(&file_content)
        .map_err(|e| Box::<dyn std::error::Error>::from(format!("Error parsing JSON in file '{}': {}", filename, e)))?;
    Ok(data)
}

/// Writes a serializable Rust object to a JSON file, with filename-aware error reporting.
pub fn write_json_file<T>(filename: &str, content: &T) -> Result<(), Box<dyn std::error::Error>>
where
    T: Serialize,
{
    let json_string = to_string_pretty(content)
        .map_err(|e| Box::<dyn std::error::Error>::from(format!("Error serializing JSON for file '{}': {}", filename, e)))?;

    fs::write(filename, json_string)
        .map_err(|e| Box::<dyn std::error::Error>::from(format!("Error writing to file '{}': {}", filename, e)))?;

    Ok(())
}

