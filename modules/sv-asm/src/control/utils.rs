use sv_lib::model::{ArchConfig};
use log::{error};
use std::collections::{HashMap};
use std::fmt::UpperHex;
use std::time::Duration;
use wait_timeout::ChildExt;


pub const TOOLCHAIN: &str = "riscv32-unknown-elf-";

pub fn to_hex<T>(v: T) -> String
where
    T: UpperHex,
{
    format!("0x{:X}", v)
}

pub fn to_hex_sv<T>(v: T) -> String
where
    T: UpperHex + Copy,
{
    let bits = std::mem::size_of::<T>() * 8;
    format!("{}'h{:0width$X}", bits, v, width = bits / 4)
}

pub fn vec_to_c_array<T>(v: &[T]) -> String
where
    T: UpperHex + Copy,
{
    v.iter()
        .map(|x| format!("0x{:X}", *x))
        .collect::<Vec<_>>()
        .join(", ")
}


pub fn vec2d_to_c<T>(v: &[Vec<T>]) -> String
where
    T: UpperHex + Copy,
{
    v.iter()
        .map(|row| {
            let row_str = row.iter()
                .map(|x| format!("0x{:X}", *x))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{{{}}}", row_str)
        })
        .collect::<Vec<_>>()
        .join(", ")
}


pub fn runc(
    cmd: &mut std::process::Command
) -> Result<(), Box<dyn std::error::Error>> {
    
    let output = cmd.output()?;  // capture stdout + stderr

    if !output.status.success() {
        error!("Command failed: {:?}", cmd);

        error!("--- stdout ---");
        error!("{}", String::from_utf8_lossy(&output.stdout));

        error!("--- stderr ---");
        error!("{}", String::from_utf8_lossy(&output.stderr));

        return Err("Command execution failed".into());
    }
    
    Ok(())
}


pub fn elf_parse_sections(
    output: &str
) -> Result<HashMap<String, (u32, u32)>, Box<dyn std::error::Error>> {
    let mut sections = HashMap::new();

    for line in output.lines() {
        let parts: Vec<_> = line.split_whitespace().collect();

        if parts.len() < 4 {
            continue;
        }

        if let Ok(_idx) = parts[0].parse::<u32>() {
            let name = parts[1].to_string();
            let size = u32::from_str_radix(parts[2], 16)?;
            let addr = u32::from_str_radix(parts[3], 16)?;

            sections.insert(name, (size, addr));
        }
    }

    Ok(sections)
}



pub fn validate_sections(
    name: &str,
    arch: &ArchConfig,
    sections: &HashMap<String, (u32, u32)>,
) -> Result<(), Box<dyn std::error::Error>> {
   
    let required: &[&str] = if arch.part1_size != 0 {
        &[".part1", ".part2", ".data"]
    } else {
        &[".text", ".data"]
    };

    for r in required {
        let (size, addr) = sections
            .get(*r)
            .ok_or_else(|| format!("{} missing section {}", name, r))?;

        match *r {
            ".text" => {
                if *size > arch.inst_mem_length || *addr != 0 {
                    return Err(format!("{} invalid configuration of {}", name, r).into());
                }
            }
            ".part1" => {
                if *size > arch.part1_size || *addr != 0 {
                    return Err(format!("{} invalid configuration of {}", name, r).into());
                }
            }
            ".part2" => {
                if *size > arch.part2_size || *addr != arch.part1_size {
                    return Err(format!("{} invalid configuration of {}", name, r).into());
                }
            }
            ".data" => {
                if *size > arch.data_mem_length || *addr != arch.data_offset {
                    return Err(format!("{} invalid configuration of {}", name, r).into());
                }
            }
            _ => unreachable!(),
        }
    }

    // .bss must be zero if present
    if let Some((size, _)) = sections.get(".bss") {
        if *size != 0 {
            return Err(format!("{}: .bss is not zero", name).into());
        }
    }

    // .shared must exist AND be correct
    if arch.share_mem_length != 0 {
        match sections.get(".shared") {
            Some((size, addr)) if *size == 0 && *addr == arch.share_offset => {}
            _ => {
                return Err(format!("{}: .shared missing or invalid", name).into());
            }
        }
    }

    // Allowed extra sections
    let allowed_prefix = [".comment", ".note", ".attributes"];

    for key in sections.keys() {
        if required.contains(&key.as_str())
            || key == ".bss"
            || key == ".shared"
            || allowed_prefix.iter().any(|p| key.starts_with(p))
            || allowed_prefix.iter().any(|p| key.ends_with(p))
        {
            continue;
        }

        return Err(format!("{}: unexpected section {}", name, key).into());
    }

    Ok(())
}



pub fn run_vsim(
    tb_name: &str,
    time_limit: u32,
    output_file: &std::path::Path,
    sim_dir: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {

    // ------------------------------------
    // handle Bender
    runc(
        std::process::Command::new("bender")
            .arg("clean")
            .current_dir(sim_dir)
    )?;

    let compile_path = sim_dir.join("compile.tcl");
    let compile_file = std::fs::File::create(&compile_path)?;
    runc(
        std::process::Command::new("bender")
            .arg("script")
            .arg("vsim")
            .arg("-t")
            .arg("tb")
            .current_dir(sim_dir)
            .stdout(std::process::Stdio::from(compile_file))
    )?;

    // ------------------------------------
    // clean vsim work directory
    let work_dir = sim_dir.join("work");

    if work_dir.is_dir() {
        runc(
            std::process::Command::new("vdel")
                .arg("-all")
                .current_dir(sim_dir)
        )?;
    }

    // ------------------------------------
    // running rtl simulation
    let stdout_file = std::fs::File::create(&output_file)?;
    let run_command = format!("set tb_name {}; do run.tcl", tb_name);
    let mut child = std::process::Command::new("vsim")
        .arg("-c")
        .arg("-do")
        .arg(&run_command)
        .current_dir(sim_dir)
        .stdout(std::process::Stdio::from(stdout_file))
        .spawn()?;

    let timeout = Duration::from_secs(60 * time_limit as u64);

    match child.wait_timeout(timeout)? {
        Some(status) => {
            if !status.success() {
                return Err(format!("Command failed with status {}", status).into());
            }
        }
        None => {
            child.kill()?;
            child.wait()?; // reap zombie

            return Err(format!("Command timed out after {:?}", timeout).into());
        }
    }

    Ok(())
}

