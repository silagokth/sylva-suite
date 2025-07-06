use std::fs;
use std::error::Error;

/// Reads the entire contents of a file and returns it as a `String`.
pub fn load_file(path: &str) -> Result<String, Box<dyn Error>> {
    let content = fs::read_to_string(path)?;
    Ok(content)
}

/// Writes a string to a file. If the file exists, it will be overwritten.
pub fn write_file(path: &str, content: &str) -> Result<(), Box<dyn Error>> {
    fs::write(path, content)?;
    Ok(())
}
