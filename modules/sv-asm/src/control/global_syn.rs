use sv_lib::model::{DataBase, AlimpDataFormat, 
                    HardwareIOConfig, HardwareTPConfig, 
                    ArchConfig, CPUSettings};
use sv_lib::file_handler;
use crate::control::utils;
use log::{error};
use std::process::Command;
use std::collections::HashMap;
use serde::Serialize;
use tera::{Tera, Context};
use once_cell::sync::Lazy;
use regex::Regex;

static TERA: Lazy<Tera> = Lazy::new(|| {
    let mut tera = Tera::default();
    tera.add_raw_template(
        "alimp_top.sv",
        include_str!("templates/alimp_top.sv.tmpl")
    ).unwrap();
    tera.add_raw_template(
        "main.c",
        include_str!("templates/host_firmware.c.tmpl")
    ).unwrap();
    tera.add_raw_template(
        "sections.lds",
        include_str!("templates/host_sections.lds.tmpl")
    ).unwrap();
    tera.add_raw_template(
        "system_scheduling_tb.sv",
        include_str!("templates/system_scheduling_tb.sv.tmpl")
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
struct AlimpTopTemplateCtx {
    #[serde(rename = "N_ALIMP")]
    n_alimp: u32,
    #[serde(rename = "MAX_COLS")]
    max_cols: u32,
    #[serde(rename = "BASE_CFG")]
    base_cfg: String,
    #[serde(rename = "IN_IO_CFG")]
    in_io_cfg: String,
    #[serde(rename = "OUT_IO_CFG")]
    out_io_cfg: String,
    #[serde(rename = "OUT_TP_CFG")]
    out_tp_cfg: String,
    #[serde(rename = "APP_INTERFACE_CONNECT")]
    app_interface_connect: String,
}


fn hardware_system_generation(
    db: &mut DataBase, 
    dir: &String,
) -> Result<(), Box<dyn std::error::Error>> {
     
    let cfg = &mut db.synthesized_information.control_synthesis;

    // --------------------------------
    // Basic configuration
    // --------------------------------
    let n_alimp = cfg.alimp_id.len() as u32;
    let max_cols: u32 = 16; // FIXED
    
    // --------------------------------
    // Base - Common, PICO, DRRA - settings 
    // --------------------------------
    let mut base_cfg = String::from("'{\n");

    let pico_settings = 
"           pico: '{
                ENABLE_COUNTERS: 0,
                ENABLE_COUNTERS64: 0,
                ENABLE_REGS_16_31: 1,
                ENABLE_REGS_DUALPORT: 0,
                LATCHED_MEM_RDATA: 0,
                TWO_STAGE_SHIFT: 0,
                BARREL_SHIFTER: 0,
                TWO_CYCLE_COMPARE: 0,
                TWO_CYCLE_ALU: 0,
                COMPRESSED_ISA: 0,
                CATCH_MISALIGN: 0,
                CATCH_ILLINSN: 0,
                ENABLE_PCPI: 0,
                ENABLE_MUL: 1,
                ENABLE_FAST_MUL: 0,
                ENABLE_DIV: 0,
                ENABLE_IRQ: 0,
                ENABLE_IRQ_QREGS: 0,
                ENABLE_IRQ_TIMER: 0,
                ENABLE_TRACE: 0,
                REGS_INIT_ZERO: 0,
                MASKED_IRQ: 32'h0000_0000,
                LATCHED_IRQ: 32'hffff_ffff,
                PROGADDR_RESET: 32'h0000_0000,
                PROGADDR_IRQ: 32'h0000_0010,
                STACKADDR: 32'hffff_ffff
            }";
    
    for (idx, id) in cfg.alimp_id.iter().enumerate() {
        let alimp_cfg = cfg
            .alimp_control_synthesis
            .get(id)
            .ok_or_else(|| format!("Alimp {} is not found in control synthesis", id))?;
    
        let hardware_cfg = &alimp_cfg.hardware_config.hardware_common_config;
        let drra_cfg = &alimp_cfg.hardware_config.hardware_drra_config;
    
        let cmn_settings = format!(
"            cmn: '{{
                AXI_ADDR_WIDTH: {},
                AXI_DATA_WIDTH: {},
                CPU_ADDR_WIDTH: {},
                CPU_DATA_WIDTH: {},
                CPU_INSTMEM_DEPTH: {},
                CPU_DATAMEM_DEPTH: {},
                CPU_SHAREMEM_DEPTH: {},
                CHUNK_ADDR_WIDTH: {},
                CHUNK_DATA_WIDTH: {},
                ID_BITS: {},
                TLB_PROGRAM_ADDR_WIDTH: {},
                TLB_AGU_INTERNAL_WIDTH: {},
                TP_START_BITS: {},
                TP_INTERNAL_COL_MSB: {},
                TP_INTERNAL_COL_LSB: {},
                TP_INTERNAL_DATA_WIDTH: {},
                TP_PROGRAM_ADDR_WIDTH: {},
                TP_PROGRAM_DATA_WIDTH: {}
            }}",
            hardware_cfg.axi_addr_width,
            hardware_cfg.axi_data_width,
            hardware_cfg.cpu_addr_width,
            hardware_cfg.cpu_data_width,
            hardware_cfg.cpu_instmem_depth,
            hardware_cfg.cpu_datamem_depth,
            hardware_cfg.cpu_sharemem_depth,
            hardware_cfg.chunk_addr_width,
            hardware_cfg.chunk_data_width,
            hardware_cfg.id_bits,
            hardware_cfg.tlb_program_addr_width,
            hardware_cfg.tlb_agu_internal_width,
            hardware_cfg.tp_start_bits,
            hardware_cfg.tp_internal_col_msb,
            hardware_cfg.tp_internal_col_lsb,
            hardware_cfg.tp_internal_data_width,
            hardware_cfg.tp_program_addr_width,
            hardware_cfg.tp_program_data_width
        );
    
        let drra_settings = format!(
"           drra: '{{
                ROWS: {},
                COLS: {},
                INSTR_DATA_WIDTH: {},
                INSTR_ADDR_WIDTH: {},
                INSTR_HOPS_WIDTH: {},
                IO_ADDR_WIDTH: {},
                DM_ADDR_WIDTH: {}
            }}",
            drra_cfg.rows,
            drra_cfg.cols,
            drra_cfg.instr_data_width,
            drra_cfg.instr_addr_width,
            drra_cfg.instr_hops_width,
            drra_cfg.io_addr_width,
            drra_cfg.dm_addr_width
        );
    
        let alimp_settings = format!(
"      '{{\n{},\n{},\n{}\n
        }}",
            cmn_settings,
            pico_settings,
            drra_settings
        );
    
        base_cfg.push_str(&alimp_settings);
    
        // Add comma except for last element
        if idx != cfg.alimp_id.len() - 1 {
            base_cfg.push_str(",\n");
        } else {
            base_cfg.push('\n');
        }
    }
    
    base_cfg.push_str(
"   };\n");

    // --------------------------------
    // IO settings 
    // --------------------------------
    let mut in_io_cfg = String::from("'{\n");
    let mut out_io_cfg = String::from("'{\n");
    let mut out_tp_cfg = String::from("'{\n");

    for (i, id) in cfg.alimp_id.iter().enumerate() {
        let alimp_cfg = cfg
            .alimp_control_synthesis
            .get(id)
            .ok_or_else(|| format!("Alimp {} is not found in control synthesis", id))?;
    
        let ib_cfgs = &alimp_cfg.hardware_config.in_io_config;
        let ob_cfgs = &alimp_cfg.hardware_config.out_io_config;
        let tp_cfgs = &alimp_cfg.hardware_config.out_tp_config;
        
        // -------------------- IB ----------------------------
        let mut ib_settings = String::new();

        for j in 0..max_cols as usize {
            let ib_cfg = if j < ib_cfgs.len() {
                &ib_cfgs[j]
            } else {
                &HardwareIOConfig::default()
            };

            let setting = if ib_cfg.active || ib_cfg.skip {
                format!(
"              '{{
                    active:{}, skip:{}, buf_type:{}, buf_size:{}, block_size:{},
                    input1:{}, input2:{}, output1:{}, output2:{},
                    tlb1:'{{ tlb_type:{}, program_size:{} }},
                    tlb2:'{{ tlb_type:{}, program_size:{} }}
                }}",
                    ib_cfg.active as u32,
                    ib_cfg.skip as u32,
                    ib_cfg.buf_type,
                    ib_cfg.buf_size,
                    ib_cfg.block_size,
                    ib_cfg.input1,
                    ib_cfg.input2,
                    ib_cfg.output1,
                    ib_cfg.output2,
                    ib_cfg.tlb1.tlb_type,
                    ib_cfg.tlb1.program_size,
                    ib_cfg.tlb2.tlb_type,
                    ib_cfg.tlb2.program_size
                )
            } else {
"               IO_DEFAULT".to_string()
            };

            ib_settings.push_str(&setting);
            if j != max_cols as usize - 1 {
                ib_settings.push_str(",\n");
            } 
        }

        in_io_cfg.push_str(&format!(
"       '{{\n{}\n
        }}", ib_settings));
        if i != cfg.alimp_id.len() - 1 {
            in_io_cfg.push_str(",\n");
        } else {
            in_io_cfg.push_str("\n");
        }
            
        // -------------------- OB ----------------------------
        let mut ob_settings = String::new();

        for j in 0..max_cols as usize {
            let ob_cfg = if j < ob_cfgs.len() {
                &ob_cfgs[j]
            } else {
                &HardwareIOConfig::default()
            };

            let setting = if ob_cfg.active || ob_cfg.skip {
                format!(
"               '{{
                    active:{}, skip:{}, buf_type:{}, buf_size:{}, block_size:{},
                    input1:{}, input2:{}, output1:{}, output2:{},
                    tlb1:'{{ tlb_type:{}, program_size:{} }},
                    tlb2:'{{ tlb_type:{}, program_size:{} }}
                }}",
                    ob_cfg.active as u32,
                    ob_cfg.skip as u32,
                    ob_cfg.buf_type,
                    ob_cfg.buf_size,
                    ob_cfg.block_size,
                    ob_cfg.input1,
                    ob_cfg.input2,
                    ob_cfg.output1,
                    ob_cfg.output2,
                    ob_cfg.tlb1.tlb_type,
                    ob_cfg.tlb1.program_size,
                    ob_cfg.tlb2.tlb_type,
                    ob_cfg.tlb2.program_size
                ) 
            } else {
"               IO_DEFAULT".to_string()
            };

            ob_settings.push_str(&setting);
            if j != max_cols as usize - 1 {
                ob_settings.push_str(",\n");
            }
        }
            
        out_io_cfg.push_str(&format!(
"       '{{\n{}\n
        }}", ob_settings));

        if i != cfg.alimp_id.len() - 1 {
            out_io_cfg.push_str(",\n");
        } else {
            out_io_cfg.push_str("\n");
        }

        // -------------------- TP ----------------------------
        let mut tp_settings = String::new();

        for j in 0..max_cols as usize {
            let tp_cfg = if j < tp_cfgs.len() {
                &tp_cfgs[j]
            } else {
                &HardwareTPConfig::default()
            };

            let setting = if tp_cfg.active {
                format!("'{{active:{}, program_size:{}, last:{}}}",
                    tp_cfg.active as u32,
                    tp_cfg.program_size,
                    tp_cfg.last
                ) 
            } else {
                "TP_DEFAULT".to_string()
            };

            tp_settings.push_str(&setting); 

            if j != max_cols as usize - 1 {
                tp_settings.push_str(", ");
            } 
        }
            
        out_tp_cfg.push_str(&format!(
"       '{{ {} }}", tp_settings));
        if i != cfg.alimp_id.len() - 1 {
            out_tp_cfg.push_str(",\n");
        } else {
            out_tp_cfg.push_str("\n");
        }
    }

    in_io_cfg.push_str("    };\n");
    out_io_cfg.push_str("   };\n");
    out_tp_cfg.push_str("   };\n");

    // --------------------------------
    // Application data interface  
    // --------------------------------

    let mut app_interface_connect = String::new();
    let routes = &db.synthesized_information.routing_paths;

    for route in routes.iter() {
        let (source_name, source_col) = &route.source;
        let (target_name, target_col) = &route.target;

        let delay = if route.delay < 0 {
            return Err(format!("Negative delay for route {:?}->{:?}", source_name, target_name).into());
        } else {
            route.delay as u32
        };

        // map Alimp names to IDs
        let source_id = cfg.alimp_id
            .iter()
            .enumerate()
            .find(|(_, n)| *n == source_name)
            .map(|(idx, _)| idx)
            .ok_or_else(|| format!("Node {} is not found in Control Synthesis Info", source_name))?;

        let target_id = cfg.alimp_id
            .iter()
            .enumerate()
            .find(|(_, n)| *n == target_name)
            .map(|(idx, _)| idx)
            .ok_or_else(|| format!("Node {} is not found in Control Synthesis Info", target_name))?;

        let connect = format!(
"    app_if_connect #(.A(32), .W(128), .DEPTH({})) app_connect_{}x{}_{}x{} (
        .clk    (clk), 
        .rst_n  (rst_n),
        .in_if  (app_out[{}][{}]),
        .out_if (app_in[{}][{}]) 
    );\n\n",
            delay,
            source_id,
            source_col,
            target_id,
            target_col,
            source_id,
            source_col,
            target_id,
            target_col
        );

        app_interface_connect.push_str(&connect);
    }

    // --------------------------------
    // Generate the file 
    // --------------------------------
    let alimp_top_ctx = AlimpTopTemplateCtx {
        n_alimp: n_alimp,
        max_cols: max_cols,
        base_cfg: base_cfg,
        in_io_cfg: in_io_cfg,
        out_io_cfg: out_io_cfg,
        out_tp_cfg: out_tp_cfg,
        app_interface_connect: app_interface_connect
    };

    let alimp_top_context = Context::from_serialize(&alimp_top_ctx)?;
    let alimp_top_rendered = TERA.render("alimp_top.sv", &alimp_top_context)
        .map_err(|e| format!("Tera error:\n{}", e))?;

    let alimp_top_path = std::path::Path::new(dir).join("alimp_top.sv");
    file_handler::write_file(&alimp_top_path, alimp_top_rendered)?;
    
    // update alimp_top path
    cfg.alimp_top_hardware_path = alimp_top_path;
    
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


fn generate_firmware(
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
    cfg.host_cpu_settings = cpu_settings.clone();

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
        cpu_settings.instruction_memory_size,
        cpu_settings.data_memory_size,
        &main_path,
        &lds_path,
        &output_firmware_path,
        &working_dir)?;

    // -----------------------------
    // Extract binary 
    // -----------------------------
    let mut host_text: Vec<u32> = Vec::new();
    let mut host_data: Vec<u32> = Vec::new();

    // ------------------------------        
    // objdump -h
    let output = Command::new(format!("{}objdump", utils::TOOLCHAIN))
        .arg("-h")
        .arg(&output_firmware_path)
        .output()?; 

    if !output.status.success() {
        return Err(format!("objdump failed for {}", output_firmware_path.display()).into());
    }

    let stdout = String::from_utf8(output.stdout)?;
    let sections = utils::elf_parse_sections(&stdout)?;

    let architecture = ArchConfig {
        inst_mem_length: cfg.host_cpu_settings.instruction_memory_size,  
        data_mem_length: cfg.host_cpu_settings.data_memory_size,     
        share_mem_length: 0,    // FIXED
        part1_size: 0,          // FIXED
        part2_size: 0,          // FIXED
        data_offset: 0x10000,   // FIXED
        share_offset: 0,        // FIXED
    };
    utils::validate_sections("host", &architecture, &sections)?;
        
    // ------------------------------        
    // Extract sections
    let target_sections = [".text", ".data"];
    let parent_dir = output_firmware_path
        .parent()
        .ok_or("Invalid ELF path (no parent directory)")?;

    for section_name in target_sections.iter() {
        let tmp_out = parent_dir.join(format!("{}.bin", section_name));
            
        let tool = format!("{}objcopy", utils::TOOLCHAIN);
            
        let output = Command::new(&tool)
            .arg("-O")
            .arg("binary")
            .arg("-j")
            .arg(&section_name)
            .arg(&output_firmware_path)
            .arg(&tmp_out)
            .output()?;

        if !output.status.success() {
            let error_command = format!(
                "Command: {} -O binary -j {} {} {}",
                tool,
                section_name,
                output_firmware_path.display(),
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
                "Host firmware: size mismatch for {} (expected {}, got {})",
                section_name,
                size,
                raw.len()
            )
            .into());
        }

        if raw.len() % 4 != 0 {
            return Err(format!(
                "Host firmware: section {} not aligned to u32", section_name
            )
            .into());
        }

        // Convert to u32 (little endian) and push to binary structs
        for chunk in raw.chunks_exact(4) {
            let val = u32::from_le_bytes(chunk.try_into().unwrap());
            match *section_name {
                ".text" => host_text.push(val),
                ".data" => host_data.push(val),
                _ => return Err(format!("Host firmware: unknown section {}", section_name).into()),
            };
        }
    }

    // ------------------------------        
    // save output hex
    let sections = ["text", "data"];
    for name in sections.iter() {
        let binary = match *name {
            "text" => &host_text,
            "data" => &host_data,
            _ => return Err(format!("Host firmware: unknown section {}", name).into()),
        };

        let mut out = String::new();
        
        for word in binary {
            out.push_str(&format!("{:08X}", *word));
            out.push('\n');
        }

        let binary_file = parent_dir.join(format!("{}.hex", name));
        file_handler::write_file(&binary_file, out)?;

        // update host firmware info
        match *name {
            "text" => cfg.host_text_path = binary_file,
            "data" => cfg.host_data_path = binary_file,
            _ => return Err(format!("Host firmware: unknown section {}", name).into()),
        };
    }

    // update host firmware raw info
    cfg.host_text = host_text;
    cfg.host_text = host_data;

    Ok(())
}


#[derive(Serialize)]
struct TbSchedulingTemplateCtx {
    #[serde(rename = "INST_BASE_ADDR")]
    inst_base_addr: String,
    #[serde(rename = "DATA_BASE_ADDR")]
    data_base_addr: String,
    #[serde(rename = "HOST_INSTMEM_DEPTH")]
    host_instmem_depth: String,
    #[serde(rename = "HOST_DATAMEM_DEPTH")]
    host_datamem_depth: String,
    #[serde(rename = "DATAMEM_BASE_ADDR")]
    datamem_base_addr: String,
    #[serde(rename = "DATAMEM_DEPTH")]
    datamem_depth: String,
    #[serde(rename = "PICO_CFG")]
    pico_cfg: String,
}


fn run_scheduler(
    db: &mut DataBase,
    system_dir: &String,
    module_dir: &String,
) -> Result<(), Box<dyn std::error::Error>> {

    let cfg = &mut db.synthesized_information.control_synthesis;
    
    // -----------------------------
    // generating tb file 
    // -----------------------------
    let inst_base_addr: u32 = 0x8000_0000;       // FIXED
    let data_base_addr: u32 = 0x8001_0000;       // FIXED
    let host_instmem_depth: u32 = cfg.host_cpu_settings.instruction_memory_size;  
    let host_datamem_depth: u32 = cfg.host_cpu_settings.data_memory_size;  
    let datamem_base_addr: u32 = 0x8010_0000;    // FIXED 
    let datamem_depth: u32 = 32768;              // FIXED
    let pico_cfg = "'{
            ENABLE_COUNTERS: 0,
            ENABLE_COUNTERS64: 0,
            ENABLE_REGS_16_31: 1,
            ENABLE_REGS_DUALPORT: 0,
            LATCHED_MEM_RDATA: 0,
            TWO_STAGE_SHIFT: 0,
            BARREL_SHIFTER: 0,
            TWO_CYCLE_COMPARE: 0,
            TWO_CYCLE_ALU: 0,
            COMPRESSED_ISA: 0,
            CATCH_MISALIGN: 0,
            CATCH_ILLINSN: 0,
            ENABLE_PCPI: 0,
            ENABLE_MUL: 1,
            ENABLE_FAST_MUL: 0,
            ENABLE_DIV: 0,
            ENABLE_IRQ: 0,
            ENABLE_IRQ_QREGS: 0,
            ENABLE_IRQ_TIMER: 0,
            ENABLE_TRACE: 0,
            REGS_INIT_ZERO: 0,
            MASKED_IRQ: 32'h0000_0000,
            LATCHED_IRQ: 32'hffff_ffff,
            PROGADDR_RESET: 32'h0000_0000,
            PROGADDR_IRQ: 32'h0000_0010,
            STACKADDR: 32'hffff_ffff
        }".to_string();
   
    let tb_ctx = TbSchedulingTemplateCtx {
        inst_base_addr: utils::to_hex_sv(inst_base_addr), 
        data_base_addr: utils::to_hex_sv(data_base_addr), 
        host_instmem_depth: utils::to_hex_sv(host_instmem_depth), 
        host_datamem_depth: utils::to_hex_sv(host_datamem_depth), 
        datamem_base_addr: utils::to_hex_sv(datamem_base_addr), 
        datamem_depth: utils::to_hex_sv(datamem_depth),
        pico_cfg: pico_cfg,
    };

    let tb_context = Context::from_serialize(&tb_ctx)?;
    let tb_rendered = TERA.render("system_scheduling_tb.sv", &tb_context)
        .map_err(|e| format!("Tera error:\n{}", e))?;

    let tb_file = std::path::Path::new(module_dir).join("system_scheduling_tb.sv");
    file_handler::write_file(&tb_file, tb_rendered)?;
    cfg.tb_scheduling_path = tb_file;

    // -----------------------------
    // getting the vsim framework ready 
    // -----------------------------
    let working_dir = std::path::Path::new(system_dir);

    // copy alimp_top.sv -> rtl/
    let alimp_top_dst = working_dir.join("rtl").join("alimp_top.sv");
    std::fs::copy(&cfg.alimp_top_hardware_path, &alimp_top_dst)?;
 
    // copy system_tb.sv -> tb/
    let tb_dst = working_dir.join("tb").join("system_scheduling_tb.sv");
    std::fs::copy(&cfg.tb_scheduling_path, &tb_dst)?;

    // copy all binaries -> tb/data/
    let text_dst = working_dir.join("tb").join("data").join("text.hex");
    let data_dst = working_dir.join("tb").join("data").join("data.hex");
    let alimp_data_dst = working_dir.join("tb").join("data").join("alimp_data.hex");
    std::fs::copy(&cfg.host_text_path, &text_dst)?;
    std::fs::copy(&cfg.host_data_path, &data_dst)?;
    std::fs::copy(&cfg.alimp_data_path, &alimp_data_dst)?;

    // -----------------------------
    // running vsim 
    let run_file = std::path::Path::new(module_dir).join("scheduling.txt");
    utils::run_vsim("system_scheduling_tb", 2, &run_file, working_dir)?;
    cfg.tb_output_path = run_file;

    Ok(())
}



fn global_scheduler(
    db: &mut DataBase,
) -> Result<(), Box<dyn std::error::Error>> {

    let cfg = &mut db.synthesis_information.control_synthesis;

    // -------------------------------------
    // Parse the TB output 
    // -------------------------------------

    let content = std::fs::read_to_string(cfg.tb_output_path)?;

    // Check markers
    let finished = content.contains("$finish");
    let test_done = content.contains("--- Test Done ---");

    if (!finished || !test_done) {
        return Err("TB result is not complete and cannot be parsed");
    }

    // Getting GLOBAL_TIME
    let global_time_re = Regex::new(r"GLOBAL_TIME:\s*(\d+)")?;
    let global_time = global_time_re
        .captures(&content)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().parse::<u64>().unwrap());
    cfg.global_time = global_time  

    // Getting Alimp_ready_time
    let alimp_re = Regex::new(r"ALIMP_(\d+)\s+ready time:\s*(\d+)")?;
    let mut alimp_ready_times: HashMap<u32, u64> = HashMap::new();
    
    for cap in alimp_re.captures_iter(&content) {
        let id: u32 = cap[1].parse()?;
        let val: u64 = cap[2].parse()?;
    
        alimp_ready_times.insert(id, val);
    }
    
    cfg.alimp_ready_times = alimp_ready_times;

    // -------------------------------------
    // Validate the results 
    // -------------------------------------
    for (_id, ready_time) in cfg.alimp_ready_times.iter() {
        if global_time < ready_time {
            return Err("TB result is incomplete - GLOBAL_TIME is less than one of the READY_TIMES");
        }
    }

    if cfg.alimp_ready_times.len() != cfg.alimp_id.len() {
        return Err("TB result is incomplete - number of ALIMP_READY_TIMES is incorrect");
    }

    // -------------------------------------
    // Global Scheduling 
    // -------------------------------------
    let max_ready_time: u64 = cfg.alimp_ready_times
        .iter()
        .map(|(_id, time)| time)
        .collect()
        .max();
    




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
    hardware_system_generation(db, &module_dir)?;
    
    // scheduling proces 
    generate_firmware(db, &system_dir, &module_dir)?;
    run_scheduler(db, &system_dir, &module_dir)?;
    global_scheduling(db)?;

    // host firmware correction and verification
    generate_firmware(db, &system_dir, &module_dir)?;
    run_verifier(db, &system_dir, &module_dir)?;
    verify_schedule(db)?;

    Ok(())
}

