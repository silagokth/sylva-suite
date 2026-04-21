use sv_lib::model::{DataBase, AlimpDataFormat, CPUSettings};
use sv_lib::file_handler;
use crate::control::utils;
use log::{error};
use std::collections::{HashMap};
use std::process::Command;
use serde::Serialize;
use tera::{Tera, Context};
use once_cell::sync::Lazy;

static TERA: Lazy<Tera> = Lazy::new(|| {
    let mut tera = Tera::default();
    tera.add_raw_template(
        "main.c",
        include_str!("templates/host_firmware.c.tmpl")
    ).unwrap();
    tera.add_raw_template(
        "sections.lds",
        include_str!("templates/host_sections.lds.tmpl")
    ).unwrap();
    tera
});



fn set_alimp_id(
    db: &mut DataBase,
) -> Result<(), Box<dyn std::error::Error>> {
    let alimp_controls = &db.synthesized_information
        .control_synthesis
        .alimp_control_synthesis;

    let bindings = &db.synthesized_information.alimp_bindings;
    let mut alimp_id: Vec<String> = Vec::new();

    for binding in bindings.iter() {
        let name = &binding.app_node_id;

        if !alimp_controls.contains_key(name) {
            return Err(format!("AlImp {} is not found in Control Synthesis Information", name).into());
        }

        alimp_id.push(name.clone());
    }

    db.synthesized_information.control_synthesis.alimp_id = alimp_id;

    Ok(())
}



fn generate_alimp_data(
    db: &mut DataBase,
    dir: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    
    let cfg = &mut db.synthesized_information.control_synthesis;
    
    let mut data_format: Vec<AlimpDataFormat> = Vec::new();

    let mut binary: Vec<u32> = Vec::new(); 
    let mut binary_offset: u32 = 0;

    for alimp_name in cfg.alimp_id.iter() {
        let cfg_alimp = cfg
            .alimp_control_synthesis
            .get(alimp_name)
            .ok_or_else(|| {
                format!("Node {} is not found in alimp_control_synthesis", alimp_name)
            })?;

        let elf_path = &cfg_alimp.firmware_path;
        let parent_dir = elf_path
            .parent()
            .ok_or("Invalid ELF path (no parent directory)")?;

        // ------------------------------        
        // objdump -h
        let output = Command::new(format!("{}objdump", utils::TOOLCHAIN))
            .arg("-h")
            .arg(elf_path)
            .output()?; 

        if !output.status.success() {
            return Err(format!("objdump failed for {}", elf_path.display()).into());
        }

        let stdout = String::from_utf8(output.stdout)?;
        let sections = utils::elf_parse_sections(&stdout)?;

        utils::validate_sections(alimp_name, &cfg_alimp.arch_config, &sections)?;
        
        // ------------------------------        
        // Extract sections
        let target_sections = [".part1", ".part2", ".data"];
        
        let mut offset = (0u32, 0u32, 0u32);
        let mut length = (0u32, 0u32, 0u32);
        
        for (idx, section_name) in target_sections.iter().enumerate() {
            let tmp_out = parent_dir.join(format!("{}_{}.bin", idx, section_name));
            
            let tool = format!("{}objcopy", utils::TOOLCHAIN);
            
            let output = Command::new(&tool)
                .arg("-O")
                .arg("binary")
                .arg("-j")
                .arg(section_name)
                .arg(elf_path)
                .arg(&tmp_out)
                .output()?;

            if !output.status.success() {
                let error_command = format!(
                    "Command: {} -O binary -j {} {} {}",
                    tool,
                    section_name,
                    elf_path.display(),
                    tmp_out.display()
                );
                return Err(format!("objcopy failed for {}", error_command).into());
            }

            let (size, _) = sections
                .get(*section_name)
                .ok_or_else(|| format!("Missing section {}", section_name))?;

            // Read binary file
            let raw = std::fs::read(&tmp_out)?;

            if raw.len() as u32 != *size {
                return Err(format!(
                    "{}: size mismatch for {} (expected {}, got {})",
                    alimp_name,
                    section_name,
                    size,
                    raw.len()
                )
                .into());
            }

            if raw.len() % 4 != 0 {
                return Err(format!(
                    "{}: section {} not aligned to u32",
                    alimp_name, section_name
                )
                .into());
            }

            // Convert to u32 (little endian)
            for chunk in raw.chunks_exact(4) {
                let val = u32::from_le_bytes(chunk.try_into().unwrap());
                binary.push(val);
            }

            // Store metadata
            let byte_offset = binary_offset;
            let byte_length = *size;

            match idx {
                0 => {
                    offset.0 = byte_offset;
                    length.0 = byte_length;
                }
                1 => {
                    offset.1 = byte_offset;
                    length.1 = byte_length;
                }
                2 => {
                    offset.2 = byte_offset;
                    length.2 = byte_length;
                }
                _ => unreachable!(),
            }

            binary_offset += byte_length;
        }
        
        data_format.push(AlimpDataFormat {
            section_offset: offset,
            section_length: length
        });
    }

    // output binary
    let mut out = String::new();

    for word in &binary {
        out.push_str(&format!("{:08X}", *word));
        out.push('\n');
    }

    let binary_file = std::path::Path::new(&dir).join("alimp_data.hex");
    file_handler::write_file(&binary_file, out)?;
    
    // update synthesis information
    cfg.alimp_data_format = data_format;
    cfg.alimp_data_path = binary_file; 
    cfg.alimp_data = binary; 

    Ok(())
}



#[derive(Serialize)]
struct HostTemplateCtx {
    #[serde(rename = "NUMBER_OF_ALIMPS")]
    number_of_alimps: u32,
    #[serde(rename = "MAXIMUM_TPS")]
    maximum_tps: u32,
    #[serde(rename = "ALIMP_PROGRAM_OFFSET")]
    alimp_program_offset: String,
    #[serde(rename = "ALIMP_PROGRAM_SIZE")]
    alimp_program_size: String,
    #[serde(rename = "ALIMP_PART2_OFFSET")]
    alimp_part2_offset: u32,
    #[serde(rename = "SCHEDULE_TIME")]
    schedule_time: String,
    #[serde(rename = "ACTIVE_TPS")]
    active_tps: String,
    #[serde(rename = "RELATIVE_TIME")]
    relative_time: String,
}


#[derive(Serialize)]
struct LinkerTemplateCtx {
    #[serde(rename = "INST_MEM_LENGTH")]
    inst_mem_length: String,
    #[serde(rename = "DATA_MEM_LENGTH")]
    data_mem_length: String,
}


fn generate_host_firmware(
    number_of_alimps: u32,
    maximum_tps: u32,
    alimp_program_offset: Vec<Vec<u32>>,
    alimp_program_size: Vec<Vec<u32>>,
    alimp_part2_offset: u32,
    schedule_time: Vec<u64>,
    active_tps: Vec<i32>,
    relative_time: Vec<Vec<i32>>,
    
    instruction_memory_size: u32,
    data_memory_size: u32,
    
    main_path: &std::path::Path,
    lds_path: &std::path::Path,
    firmware_path: &std::path::Path,
    working_dir: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
  
    // ----------------- write main.c -------------------------
    let host_ctx = HostTemplateCtx {
        number_of_alimps: number_of_alimps,
        maximum_tps: maximum_tps,
        alimp_program_offset: utils::vec2d_to_c(&alimp_program_offset),
        alimp_program_size: utils::vec2d_to_c(&alimp_program_size),
        alimp_part2_offset: alimp_part2_offset,
        schedule_time: utils::vec_to_c_array(&schedule_time),
        active_tps: utils::vec_to_c_array(&active_tps),
        relative_time: utils::vec2d_to_c(&relative_time),
    };

    let host_context = Context::from_serialize(&host_ctx)?;
    let host_rendered = TERA.render("main.c", &host_context)
        .map_err(|e| format!("Tera error:\n{}", e))?;

    file_handler::write_file(main_path, host_rendered)?;

    // ----------------- write sections.lds -------------------------
    let lds_ctx = LinkerTemplateCtx {
        inst_mem_length: utils::to_hex(instruction_memory_size),
        data_mem_length: utils::to_hex(data_memory_size),
    };

    let lds_context = Context::from_serialize(&lds_ctx)?;
    let lds_rendered = TERA.render("sections.lds", &lds_context)
        .map_err(|e| format!("Tera error:\n{}", e))?;

    file_handler::write_file(lds_path, lds_rendered)?;

    // ----------------------------------------------------
    // compilation
    // ----------------------------------------------------    
    utils::runc(
        std::process::Command::new("make")
            .arg("clean")
            .current_dir(working_dir)
    )?;
 
    // copy main.c → src/
    let main_dst = working_dir.join("src").join("main.c");
    std::fs::copy(&main_path, &main_dst)?;
 
    // copy sections.lds → ld/
    let lds_dst = working_dir.join("ld").join("sections.lds");
    std::fs::copy(&lds_path, &lds_dst)?;

    // make all
    utils::runc(
        std::process::Command::new("make")
            .arg("all")
            .current_dir(working_dir)
    )?;

    // ----------------------------------------------------
    // built output
    // ----------------------------------------------------
    let built_firmware_path = working_dir.join("build").join("firmware.elf");
    
    if !built_firmware_path.exists() {
        return Err(
            format!("Missing build artifact: {:?}", 
                built_firmware_path.display()).into()
            );
    }
    
    std::fs::copy(&built_firmware_path, &firmware_path)
        .map_err(|e| {
            format!("Failed to copy {:?} -> {:?}: {}", 
                &built_firmware_path, 
                &firmware_path, 
                e)
        })?;
 
    Ok(())
}


fn generate_scheduling_firmware(
    db: &mut DataBase,
    system_dir: &String,
    module_dir: &String,
) -> Result<(), Box<dyn std::error::Error>> {

    let cfg = &mut db.synthesized_information.control_synthesis;
    let first_alimp = cfg.alimp_id.first().ok_or("No ALIMPs found in control_synthesis->alimp_id")?;

    // -----------------------------
    // firmware parameters
    // -----------------------------
    let number_of_alimps = cfg.alimp_id.len() as u32;
    let maximum_tps: u32 = 16;                      // FIXED
    let alimp_program_dram_offset: u32 = 0x10_0000; // FIXED

    let alimp_program_offset: Vec<Vec<u32>> = cfg
        .alimp_data_format
        .iter()
        .map(|df| {
            let offsets = [df.section_offset.0, df.section_offset.1, df.section_offset.2];
            offsets
                .iter()
                .map(|&x| x + alimp_program_dram_offset)
                .collect()
        })
        .collect();

    let alimp_program_size: Vec<Vec<u32>> = cfg
        .alimp_data_format
        .iter()
        .map(|df| vec![df.section_length.0, df.section_length.1, df.section_length.2])
        .collect();
    
    let alimp_part2_offset = cfg
        .alimp_control_synthesis
        .get(first_alimp)
        .ok_or("Missing ALIMP config")?
        .arch_config
        .part1_size;
    
    // Set to almost max time for scheduling
    let schedule_time: Vec<u64> =
        vec![u64::MAX - u32::MAX as u64; number_of_alimps as usize];

    let mut active_tps: Vec<i32> =
        vec![0; number_of_alimps as usize];

    let mut relative_time: Vec<Vec<i32>> =
        vec![vec![0; maximum_tps as usize]; number_of_alimps as usize];    

    for (id, alimp_name) in cfg.alimp_id.iter().enumerate() {
        let sync = &cfg
            .alimp_control_synthesis
            .get(alimp_name)
            .ok_or_else(|| format!("Missing config for {}", alimp_name))?
            .synchronisation;
        
        active_tps[id] = sync.len() as i32;

        for (&tp_id, &time) in sync.iter() {
            if tp_id >= maximum_tps {
                return Err(format!(
                    "ALIMP {} exceeds maximum_tps ({})",
                    alimp_name, maximum_tps
                ).into());
            }

            relative_time[id][tp_id as usize] = time;
        }
    }


    // -----------------------------
    // memory and linker parameters
    // -----------------------------
    let cpu_settings = CPUSettings {
        instruction_memory_size: 32768,
        data_memory_size: 32768,
    };
    cfg.host_cpu_settings = cpu_settings;

    let instruction_memory_size = cfg.host_cpu_settings.instruction_memory_size;
    let data_memory_size = cfg.host_cpu_settings.data_memory_size;

    // -----------------------------
    // directory settings
    // -----------------------------
    let main_path = std::path::Path::new(module_dir).join("main.c");
    let lds_path = std::path::Path::new(module_dir).join("sections.lds");
    let output_firmware_path = std::path::Path::new(module_dir).join("scheduling_firmware.elf");
    let working_dir = std::path::Path::new(system_dir).join("firmware");
    
    // -----------------------------
    // generate firmware
    // -----------------------------
    generate_host_firmware(
        number_of_alimps, 
        maximum_tps,
        alimp_program_offset,
        alimp_program_size,
        alimp_part2_offset,
        schedule_time,
        active_tps,
        relative_time,
        instruction_memory_size,
        data_memory_size,
        &main_path,
        &lds_path,
        &output_firmware_path,
        &working_dir)?;

    Ok(())
}



pub fn main(
    db: &mut DataBase, 
    dir: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    let module_dir = format!("{}/_global_scheduler", dir);
    match std::fs::create_dir_all(&module_dir) {
        Ok(_) => (),
        Err(e) => {
            error!("Failed to create {} with {}", module_dir, e);
            return Err(Box::new(e));
        },
    };

    let system_dir = format!("{}/_work/integration/system", dir);
    
    set_alimp_id(db)?;
    generate_alimp_data(db, &module_dir)?;
    generate_scheduling_firmware(db, &system_dir, &module_dir)?;
    //global_scheduler(db, &system_dir, &module_dir)?;
    //host_firmware_regeneration(db, &system_dir, &module_dir)?;
    //global_scheduling_verification(db, &system_dir, &module_dir)?;

    Ok(())
}

