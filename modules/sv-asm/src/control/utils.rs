use sv_lib::model::{ArchConfig};
use log::{error};
use std::collections::{HashMap};
use std::fmt::UpperHex;

pub const TOOLCHAIN: &str = "riscv32-unknown-elf-";

pub fn to_hex<T>(v: T) -> String
where
    T: UpperHex,
{
    format!("0x{:X}", v)
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
    let required = [".part1", ".part2", ".data"];

    for r in required {
        let (size, addr) = sections
            .get(r)
            .ok_or_else(|| format!("{} missing section {}", name, r))?;

        match r {
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
                if *addr != 0x1_0000 {
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
    match sections.get(".shared") {
        Some((size, addr)) if *size == 0 && *addr == 0x20000 => {}
        _ => {
            return Err(format!("{}: .shared missing or invalid", name).into());
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


