use crate::file_handler::{write_file};
use crate::utils::{CPU_LIMIT, MEMORY_LIMIT};
use std::process::{Command};
use log::{error};
use std::fs::File;
use std::io::{BufReader, BufRead};
use serde::Deserialize;
use serde_json;
use std::collections::HashMap;
use regex::Regex;


#[derive(Debug, Deserialize)]
#[serde(untagged)] 
#[allow(dead_code)]
enum MiniZincStreamEntry {
    Solution(SolutionEntry),
    Status(StatusEntry),
    Unknown(HashMap<String, serde_json::Value>),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")] 
#[allow(dead_code)]
struct SolutionEntry {
    #[serde(rename = "type")]
    entry_type: String, // Should be "solution"
    output: SolutionOutput,
    sections: Vec<String>,
    #[serde(default)]
    time: u64,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SolutionOutput {
    default: String, 
    raw: String,     
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct StatusEntry {
    #[serde(rename = "type")]
    entry_type: String, // Should be "status"
    status: String, // E.g., "OPTIMAL_SOLUTION", "UNSATISFIABLE", "UNKNOWN"
    #[serde(default)]
    time: u64,
}



fn minizinc_parser(file_path: &str) -> Result<Vec<MiniZincStreamEntry>, Box<dyn std::error::Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let mut entries = Vec::new();
    
    for line_result in reader.lines() {
        let line = line_result?;
        if line.trim().is_empty() {
            continue;
        }

        match serde_json::from_str::<MiniZincStreamEntry>(&line) {
            Ok(entry) => entries.push(entry),
            Err(e) => {
                error!("Error minizinc parsing JSON line in file {} with {}", file_path, e);
                return Err(Box::new(e));
            },
        }
    }
    Ok(entries)
}


/// Helper function to convert a MiniZinc-like object string to valid JSON.
/// E.g., "{objective:33, other_var:true}" -> "{"objective":33,"other_var":true}"
fn make_json_valid(s: &str) -> Result<String, regex::Error> { 
    // Regex to find unquoted keys (word characters before a colon)
    let re = Regex::new(r"(\w+)\s*:")?;

    // Replace `key:` with `"key":`
    let json_string = re.replace_all(s, r#""$1":"#).to_string();

    // Ensure it's wrapped in {} if it was just content
    if json_string.starts_with('{') && json_string.ends_with('}') {
        Ok(json_string)
    } else {
        // This case might happen if MiniZinc output wasn't a brace-enclosed object
        // For now, assume it always is.
        Ok(format!("{{{}}}", json_string))
    }
}


pub struct Solver {
    name: String,
    model: String,
    dir: String,
}

impl Solver {
    pub fn new(name: String, work_space: String) -> Self {
        Self {
            name: name.clone(),
            model: String::new(),
            dir: work_space,
        }
    }

    pub fn add(&mut self, text: String) {
        self.model.push_str(&text);
        self.model.push('\n');
    }

    pub fn new_line(&mut self) {
        self.model.push('\n');
    }


    #[allow(dead_code)]
    pub fn to_string(&mut self) -> String {
        self.model.clone()
    }

    pub fn solve(
        &mut self, 
        solver: &str,
        duration_seconds: u64,
        args: &str, 
    ) -> Result<(String, Vec<String>), Box<dyn std::error::Error>> {
        // write minizinc file 
        let minizinc_filename = format!("{}/{}.mzn", self.dir, self.name);
        write_file(&minizinc_filename, self.model.clone())?;

        let cpu_limit = *CPU_LIMIT.lock().unwrap();
        let memory_limit = *MEMORY_LIMIT.lock().unwrap();

        // run minizinc with cp-sat solver and save the output as a file
        let output_filename = format!("{}/{}_output.json", self.dir, self.name);
        let cmd = format!(
            "ulimit -v {} && minizinc {} --output-time --json-stream --solver {} -p {} --time-limit {} -o {} {}",
            memory_limit,
            minizinc_filename,
            solver,
            cpu_limit,
            duration_seconds * 1000,
            output_filename,
            args
        );

        let output = Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .output()
            .map_err(|e| format!("Failed to execute minizinc command: {}", e))?;
       
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            error!("MiniZinc process exited with non-zero status: {}", output.status);
            error!("MiniZinc stdout: {}", stdout);
            error!("MiniZinc stderr: {}", stderr);
            return Err(format!("MiniZinc command failed. Stderr: {}", stderr).into());
        }

        // read the output file and parse the result
        let mut status = String::new();
        let mut solutions: Vec<String>= vec![];
        let parsed_entries = minizinc_parser(&output_filename)?;
        for entry in parsed_entries {
            match entry {
                MiniZincStreamEntry::Solution(sol) => {
                    let trimmed_solution = sol.output.default.trim_matches(|c| c == '{' || c == '}');
                    let valid_json_str = make_json_valid(&format!("{{{}}}", trimmed_solution))?;
                    solutions.push(valid_json_str);
                },
                MiniZincStreamEntry::Status(status_entry) => {
                    status = status_entry.status;
                },
                MiniZincStreamEntry::Unknown(map) => {
                    error!("Unknown minizinc entry type: {:?}", map);
                    return Err(format!("Unknown MiniZinc entry type: {:?}", map).into());
                },
            }
        }

        Ok((status, solutions))
    }
}
