use std::process::Command;

pub fn run_command(command_with_args: &str) -> Result<String, String> {
    // split the command and arguments 
    let mut parts = command_with_args.split_whitespace();
    let command = match parts.next() {
        Some(cmd) => cmd,
        None => return Err("No command provided.".to_string()),
    };
    let args: Vec<&str> = parts.collect();
    
    // Execute the command
    let output = Command::new(command)
        .args(&args)
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stdout).to_string();
        Err(stderr)
    }
}

#[cfg(test)]
mod tests { 
    use super::*;

    #[test]
    fn list_dir() {
        let result = run_command("ls");
        match result {
            Ok(_) => return,
            Err(_) => panic!("Unexpected result from ls command")
        }
    }
    
    #[test]
    fn list_dir_with_arguments() {
        let result = run_command("ls -la");
        match result {
            Ok(_) => return,
            Err(_) => panic!("Unexpected result from ls command")
        }
    }

    #[test]
    fn no_input() {
        let result = run_command("");
        match result {
            Ok(_) => panic!("Should return error when there's no command"),
            Err(_) => return
        }
    }

    #[test]
    fn unknown_command() {
        let result = run_command("./_this_weird_name"); // I hope that no one would use this weird name
        match result {
            Ok(_) => panic!("Should return error for invalid command"),
            Err(_) => return
        }
    }
}
