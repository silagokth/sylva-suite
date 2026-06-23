use std::fs;
use std::error::Error;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{from_str, to_string_pretty};
use std::path::Path;

/// Loads a file and returns its contents as a `String`, with error reporting.
#[allow(dead_code)]
pub fn load_file<P>(filename: P) -> Result<String, Box<dyn Error>> 
where 
    P: AsRef<Path>
{
    let path = filename.as_ref();

    fs::read_to_string(path)
        .map_err(|e| -> Box<dyn Error> {
            format!("Error reading file '{}': {}", path.display(), e).into()
        })
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
pub fn write_file<P, C>(filename: P, contents: C) -> Result<(), Box<dyn Error>>
where
    P: AsRef<Path>,
    C: AsRef<[u8]>,
{
    let path = filename.as_ref();

    fs::write(path, contents)
        .map_err(|e| -> Box<dyn Error> {
            format!("Error writing to file '{}': {}", path.display(), e).into()
        })
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

