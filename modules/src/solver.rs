use crate::file_handler::{write_file};
use std::process::{Command};
use log::{error};
use std::fs::File;
use std::io::{BufReader, BufRead};
use serde::Deserialize;
use serde_json;
use std::collections::HashMap;
use regex::Regex;
use std::time::{Duration, Instant};
use std::thread;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

use std::os::unix::process::CommandExt; // for pre_exec
use nix::sys::signal::{killpg, Signal};
use nix::unistd::Pid;


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

    // kill the process after duration_seconds by ctrl-c 
    // this is because cp-sat in minizinc cannot produce a feasible solution of an optimal problem
    // after killing it with timeout flag, so we need to kill it with ctrl-c to allow minizinc to 
    // get the converging solution.
    pub fn solve(
        &mut self, 
        args: &str, 
        duration_seconds: u64, 
        interrupted: &Arc<AtomicBool>,
    ) -> Result<(String, Vec<String>), Box<dyn std::error::Error>> {
        let terminated = Arc::new(AtomicBool::new(false));

        // write minizinc file 
        let minizinc_filename = format!("{}/{}.mzn", self.dir, self.name);
        write_file(&minizinc_filename, self.model.clone())?;

        // run minizinc with cp-sat solver and save the output as a file
        let output_filename = format!("{}/{}_output.json", self.dir, self.name);

        let mut child = unsafe { 
            Command::new("minizinc")
            .arg(&minizinc_filename)
            .arg("--output-time")
            .arg("--json-stream")
            .args(args.split_whitespace()) // if args is a string like "--some-flag val"
            .arg("--solver").arg("cp-sat")
            .arg("-o").arg(&output_filename)
            .pre_exec(|| {
                libc::setsid(); 
                Ok(())
            })
            .spawn()
            .map_err(|e| format!("Failed to spawn minizinc: {}", e))?
        };
       
        let pgid = Pid::from_raw(child.id() as i32);
        let start = Instant::now();
        let timeout = Duration::from_secs(duration_seconds);
        let was_killed = Arc::new(AtomicBool::new(false));

        let was_killed_clone = was_killed.clone();    
        let interrupted_clone = interrupted.clone();
        let terminated_clone = terminated.clone();
        
        // spawn timeout/interrupt monitor
        let monitor = thread::spawn(move || {
            while start.elapsed() < timeout {
                if interrupted_clone.load(Ordering::SeqCst) {
                    // received Ctrl-C. terminating MiniZinc process
                    if let Err(e) = killpg(pgid, Signal::SIGINT) {
                        eprintln!("Failed to kill process group: {}", e);
                    }
                    return;
                }

                if terminated_clone.load(Ordering::SeqCst) {
                    return; // ended normally
                }

                thread::sleep(Duration::from_secs(1));
            }
       
            // killing N times: required for some big parallel minizinc problems  
            for i in 0..3 {
                if let Err(e) = killpg(pgid, Signal::SIGINT) {
                    eprintln!("Failed to kill process group: {}", e);
                }

                thread::sleep(Duration::from_millis(20000));

                if terminated_clone.load(Ordering::SeqCst) {
                    break;
                }

                if i == 2 {
                    error!("Failed to kill minizinc process. This needs to be done manually");
                }
            }
            was_killed_clone.store(true, Ordering::SeqCst);
        });
    
        // Wait for process to exit
        let process_status = child.wait()?;
        terminated.store(true, Ordering::SeqCst);
        monitor.join().unwrap();
        
        if !process_status.success() && !was_killed.load(Ordering::SeqCst) {
            return Err(format!("MiniZinc exited with non-zero status: {:?}", process_status.code()).into());
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
