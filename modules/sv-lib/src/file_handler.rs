use std::fs;
use std::error::Error;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{from_str, to_string_pretty};

/// Loads a file and returns its contents as a `String`, with error reporting.
#[allow(dead_code)]
pub fn load_file(filename: &str) -> Result<String, Box<dyn Error>> {
    fs::read_to_string(filename)
        .map_err(|e| -> Box<dyn Error> {format!("Error reading file '{}': {}", filename, e).into()})
}

/// Loads and deserializes a JSON file into a generic Rust type, with filename-aware error reporting.
#[allow(dead_code)]
pub fn load_json_file<T>(filename: &str) -> Result<T, Box<dyn Error>>
where
    T: DeserializeOwned,
{
    let file_content = load_file(filename)?;
    from_str(&file_content)
        .map_err(|e| -> Box<dyn Error> {format!("Error parsing JSON in file '{}': {}", filename, e).into()})
}

/// Writes a string to a file, with filename-aware error reporting.
#[allow(dead_code)]
pub fn write_file(filename: &str, string: String) -> Result<(), Box<dyn Error>> {
    fs::write(filename, string)
        .map_err(|e| -> Box<dyn Error> {format!("Error writing to file '{}': {}", filename, e).into()})
}

/// Serializes a Rust object to a pretty JSON string and writes it to a file.
#[allow(dead_code)]
pub fn write_json_file<T>(filename: &str, content: &T) -> Result<(), Box<dyn Error>>
where
    T: Serialize,
{
    let json_string = to_string_pretty(content)
        .map_err(|e| -> Box<dyn Error> {format!("Error serializing JSON for file '{}': {}", filename, e).into()})?;
    write_file(filename, json_string)
}

