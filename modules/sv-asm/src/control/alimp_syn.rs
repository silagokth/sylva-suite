use sv_lib::model::{DataBase, AlimpControlSynthesis, DrraConfig,
                    ArchConfig, TlbBlock, TpBlock, TranslationTable,
                    HardwareCommonConfig, HardwareDrraConfig, HardwareIOConfig, 
                    HardwareTPConfig, HardwareTLBConfig, MemoryStructure, TLBImplementation}; 
use sv_lib::{file_handler};
use crate::control::utils;
use std::collections::{HashMap};
use log::{error};
use serde::Serialize;
use tera::{Tera, Context};
use once_cell::sync::Lazy;


static TERA: Lazy<Tera> = Lazy::new(|| {
    let mut tera = Tera::default();
    tera.add_raw_template(
        "drra_config.c",
        include_str!("templates/alimp_drra_config.c.tmpl")
    ).unwrap();
    tera.add_raw_template(
        "sections.lds",
        include_str!("templates/alimp_sections.lds.tmpl")
    ).unwrap();
    tera
});

fn get_drra_config(
    db: &mut DataBase, 
    node_id: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    
    let alimp = &db
        .synthesized_information
        .alimp_bindings
        .iter()
        .find(|b| b.app_node_id == *node_id)
        .ok_or_else(|| format!("{} cannot be found in AlImp library", node_id))?
        .alimp_instance;

    let cols = alimp.width as usize;
    let rows = alimp.height as usize;
    let error_string = format!("DRRA configuration of {} is not compatible", node_id);

    // --- validation ---
    if alimp.instruction_offsets.len() != rows {
        return Err(error_string.clone().into());
    }
    for row in &alimp.instruction_offsets {
        if row.len() != cols {
            return Err(error_string.clone().into());
        }
    }

    if alimp.number_of_instructions.len() != rows {
        return Err(error_string.clone().into());
    }
    for row in &alimp.number_of_instructions {
        if row.len() != cols {
            return Err(error_string.clone().into());
        }
    }

    if alimp.start_address_cells.len() != rows {
        return Err(error_string.clone().into());
    }
    for row in &alimp.start_address_cells {
        if row.len() != cols {
            return Err(error_string.clone().into());
        }
    }
    
    if alimp.kernel_object.name.is_empty() {
        return Err(format!("No kernel object available for node {}", node_id).into());
    }

    // --- get mutable reference to config ---
    let cfg = db
        .synthesized_information
        .control_synthesis
        .alimp_control_synthesis
        .get_mut(node_id)
        .ok_or_else(|| format!("DRRA config for {} not initialized", node_id))?;

    cfg.ir_drra_config.rows = rows as u32;
    cfg.ir_drra_config.cols = cols as u32;
    cfg.ir_drra_config.insts_raw = alimp.instruction_code.clone();
    cfg.ir_drra_config.insts_offset_cells = alimp.instruction_offsets.clone();
    cfg.ir_drra_config.num_insts_cells = alimp.number_of_instructions.clone();
    cfg.ir_drra_config.start_addr_cells = alimp.start_address_cells.clone();
    cfg.kernel_object = alimp.kernel_object.clone();

    Ok(())
}



fn get_tlb_config(
    db: &mut DataBase,
    node_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {

    // --- collect port IDs ---
    let node = db
        .app_graph
        .nodes
        .iter()
        .find(|n| n.id == node_id)
        .ok_or_else(|| format!("Node {} not found", node_id))?;

    let in_edges: std::collections::HashSet<_> =
        node.input_ports.iter().map(|p| p.id.clone()).collect();

    let out_edges: std::collections::HashSet<_> =
        node.output_ports.iter().map(|p| p.id.clone()).collect();

    let mut in_blocks = Vec::new();
    let mut out_blocks = Vec::new();

    let mut in_offset: u32 = 0;
    let mut out_offset: u32 = 0;

    // =========================
    // IN TLB
    // =========================
    let mut in_entries: Vec<(i32, &TranslationTable)> = Vec::new();

    for at in db
        .synthesized_information
        .address_translations
        .iter()
        .filter(|t| t.app_node_id == node_id && in_edges.contains(&t.port_id))
    {
        for (ch, tt) in &at.translation_table {
            in_entries.push((*ch, tt));
        }
    }

    in_entries.sort_by_key(|(ch, _)| *ch);

    for (_ch, tt) in in_entries {
        if tt.program_code.is_empty() || tt.program_addr.is_empty() {
            return Err("Empty IN TLB program".into());
        }

        let block = TlbBlock {
            pre_ptr: tt.program_addr[0],
            pre: tt.program_code[0],
            code: tt.program_code[1..].to_vec(),
            offset: in_offset,
        };

        in_offset += block.code.len() as u32;
        in_blocks.push(block);
    }

    // =========================
    // OUT TLB
    // =========================
    let mut out_entries: Vec<(i32, &TranslationTable)> = Vec::new();

    for at in db
        .synthesized_information
        .address_translations
        .iter()
        .filter(|t| t.app_node_id == node_id && out_edges.contains(&t.port_id))
    {
        for (ch, tt) in &at.translation_table {
            out_entries.push((*ch, tt));
        }
    }

    out_entries.sort_by_key(|(ch, _)| *ch);

    for (_ch, tt) in out_entries {
        if tt.program_code.is_empty() || tt.program_addr.is_empty() {
            return Err("Empty OUT TLB program".into());
        }

        let block = TlbBlock {
            pre_ptr: tt.program_addr[0],
            pre: tt.program_code[0],
            code: tt.program_code[1..].to_vec(),
            offset: out_offset,
        };

        out_offset += block.code.len() as u32;
        out_blocks.push(block);
    }

    // =========================
    // Store
    // =========================
    let cfg = db
        .synthesized_information
        .control_synthesis
        .alimp_control_synthesis
        .get_mut(node_id)
        .ok_or_else(|| format!("DRRA config for {} not initialized", node_id))?;

    cfg.ir_drra_config.in_tlbs = in_blocks;
    cfg.ir_drra_config.out_tlbs = out_blocks;

    Ok(())
}



fn get_tp_config(
    db: &mut DataBase,
    node_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {

    // --------------------------------------------------
    // transporter table has a problem of numbering blocks
    // here we need to rename the transporters by their positions
    // --------------------------------------------------

    // Step 1: collect matching transporter indices of the same sending node
    let mut tp_indices: Vec<usize> = db
        .synthesized_information
        .transporter_tables
        .iter()
        .enumerate()
        .filter(|(_, tt)| tt.transporter_id.starts_with(&format!("transporter_{}_", node_id)))
        .map(|(i, _)| i)
        .collect();

    // Step 2: sort by placement.x
    tp_indices.sort_by_key(|&i| {
        db.synthesized_information.transporter_tables[i].placement.x
    });

    let mut new_id = 0;

    // Step 3: rename
    for i in &tp_indices {
        let tp = &mut db.synthesized_information.transporter_tables[*i];

        let current_name = tp.transporter_id.clone();

        // format: transporter_<node_id>_<target>_<id>
        let parts: Vec<&str> = current_name.split('_').collect();
        if parts.len() != 4 {
            return Err(format!("Invalid transporter_id: {}", current_name).into());
        }

        // rebuild with new id
        let new_name = format!(
            "transporter_{}_{}_{}",
            parts[1], // node_id
            parts[2], // middle part
            new_id
        );

        // update transporter table
        tp.transporter_id = new_name.clone();

        // Step 4: update node_fire_times (HashMap assumed)
        if let Some(value) = db
            .synthesized_information
            .node_fire_times
            .remove(&current_name)
        {
            db.synthesized_information
                .node_fire_times
                .insert(new_name.clone(), value);
        }

        new_id += 1;
    }

    // ----------------------
    // config the transporter code by index
    // ----------------------
    let mut blocks = Vec::new();
    let mut offset = 0;

    for i in &tp_indices {
        let tp = &db.synthesized_information.transporter_tables[*i];

        let block = TpBlock {
            code: tp.binary.clone(),
            offset: offset,
        };

        offset += block.code.len() as u32;
        blocks.push(block);
    }

    // ----------------------
    // write into config
    // ----------------------
    let cfg = db
        .synthesized_information
        .control_synthesis
        .alimp_control_synthesis
        .get_mut(node_id)
        .ok_or_else(|| format!("DRRA config for {} not initialized", node_id))?;

    cfg.ir_drra_config.tps = blocks;

    Ok(())
}

#[derive(Serialize)]
struct DrraTemplateCtx {
    #[serde(rename = "ROWS")]
    rows: u32,
    #[serde(rename = "COLS")]
    cols: u32,
    #[serde(rename = "NUM_IN_TLBS")]
    num_in_tlbs: u32,
    #[serde(rename = "NUM_OUT_TLBS")]
    num_out_tlbs: u32,
    #[serde(rename = "NUM_OUT_TPS")]
    num_out_tps: u32,

    #[serde(rename = "DRRA_INSTS_RAW_LENGTH")]
    drra_insts_raw_length: usize,
    #[serde(rename = "DRRA_INSTS_RAW")]
    drra_insts_raw: String,
    #[serde(rename = "DRRA_OFFSET_CELLS")]
    drra_offset_cells: String,
    #[serde(rename = "DRRA_NUM_INSTS")]
    drra_num_insts: String,
    #[serde(rename = "DRRA_START_ADDRS")]
    drra_start_addrs: String,

    #[serde(rename = "IN_TLB_PRE_PTRS")]
    in_tlb_pre_ptrs: String,
    #[serde(rename = "IN_TLB_PRES")]
    in_tlb_pres: String,
    #[serde(rename = "IN_TLB_OFFSETS")]
    in_tlb_offsets: String,
    #[serde(rename = "IN_TLB_LENGTHS")]
    in_tlb_lengths: String,
    #[serde(rename = "IN_TLB_CODE_FLAT")]
    in_tlb_code_flat: String,

    #[serde(rename = "OUT_TLB_PRE_PTRS")]
    out_tlb_pre_ptrs: String,
    #[serde(rename = "OUT_TLB_PRES")]
    out_tlb_pres: String,
    #[serde(rename = "OUT_TLB_OFFSETS")]
    out_tlb_offsets: String,
    #[serde(rename = "OUT_TLB_LENGTHS")]
    out_tlb_lengths: String,
    #[serde(rename = "OUT_TLB_CODE_FLAT")]
    out_tlb_code_flat: String,

    #[serde(rename = "OUT_TP_OFFSETS")]
    out_tp_offsets: String,
    #[serde(rename = "OUT_TP_LENGTHS")]
    out_tp_lengths: String,
    #[serde(rename = "OUT_TP_CODE_FLAT")]
    out_tp_code_flat: String,
}


#[derive(Serialize)]
struct LinkerTemplateCtx {
    #[serde(rename = "INST_MEM_LENGTH")]
    inst_mem_length: String,
    #[serde(rename = "DATA_MEM_LENGTH")]
    data_mem_length: String,
    #[serde(rename = "SHARE_MEM_LENGTH")]
    share_mem_length: String,
    #[serde(rename = "PART1_SIZE")]
    part1_size: String,
    #[serde(rename = "PART2_SIZE")]
    part2_size: String,
}


fn estimate_drra_code_size(cfg: &DrraConfig) -> usize {
    let mut size = 0;

    // insts_raw
    size += cfg.insts_raw.len() * 4;

    // 2D arrays
    let rc = (cfg.rows * cfg.cols) as usize;
    size += 3 * rc * 4;

    // in_tlbs
    for b in &cfg.in_tlbs {
        size += 12; 
        size += b.code.len() * 4;
    }

    // out_tlbs
    for b in &cfg.out_tlbs {
        size += 12;
        size += b.code.len() * 4;
    }

    // tps
    for b in &cfg.tps {
        size += 4; 
        size += b.code.len() * 4;
    }

    size
}



fn generate_firmware_code(
    db: &mut DataBase, 
    node_id: &String, 
    work_dir: &String,
    dir: &String,
) -> Result<(), Box<dyn std::error::Error>> {

    // ---------- getting config information ----------
    get_drra_config(db, node_id)?;
    get_tlb_config(db, node_id)?;
    get_tp_config(db, node_id)?;

    let cfg = db
        .synthesized_information
        .control_synthesis
        .alimp_control_synthesis
        .get_mut(node_id)
        .ok_or("Missing config")?;
    
    // ---------- architecture config ----------
    let mut architecture = ArchConfig {
        inst_mem_length: 2048,  // FIXED
        data_mem_length: 0,     // FLEXIBLE
        share_mem_length: 128,  // FIXED
        part1_size: 1536,       // FIXED
        part2_size: 512,        // FIXED
        data_offset: 0x10000,   // FIXED
        share_offset: 0x20000,  // FIXED
    };
    
    let estimate_data_size = estimate_drra_code_size(&cfg.ir_drra_config);
    
    // ---------- decision ----------
    architecture.data_mem_length = match estimate_data_size {
        s if s < 512        => 1024,
        s if s < 1024       => 2 * 1024,
        s if s < 2 * 1024   => 4 * 1024,
        s if s < 4 * 1024   => 8 * 1024,
        s if s < 8 * 1024   => 16 * 1024,
        _ => {
            return Err(format!(
                "AlImp {} data memory size explodes ({} bytes)",
                node_id, estimate_data_size
            ).into());
        }
    };

    cfg.arch_config = architecture;

    // ------------------ write drra_config.c --------------
    let in_tlb_pre_ptrs: Vec<u32> = cfg.ir_drra_config.in_tlbs.iter().map(|m| m.pre_ptr).collect();
    let in_tlb_pres: Vec<u32> = cfg.ir_drra_config.in_tlbs.iter().map(|m| m.pre).collect();
    let in_tlb_offsets: Vec<u32> = cfg.ir_drra_config.in_tlbs.iter().map(|m| m.offset).collect();
    let in_tlb_lengths: Vec<u32> = cfg.ir_drra_config.in_tlbs.iter().map(|m| m.code.len() as u32).collect();
    let in_tlb_code_flat: Vec<u32> = cfg.ir_drra_config.in_tlbs.iter().flat_map(|m| m.code.iter().copied()).collect();
    
    let out_tlb_pre_ptrs: Vec<u32> = cfg.ir_drra_config.out_tlbs.iter().map(|m| m.pre_ptr).collect();
    let out_tlb_pres: Vec<u32> = cfg.ir_drra_config.out_tlbs.iter().map(|m| m.pre).collect();
    let out_tlb_offsets: Vec<u32> = cfg.ir_drra_config.out_tlbs.iter().map(|m| m.offset).collect();
    let out_tlb_lengths: Vec<u32> = cfg.ir_drra_config.out_tlbs.iter().map(|m| m.code.len() as u32).collect();
    let out_tlb_code_flat: Vec<u32> = cfg.ir_drra_config.out_tlbs.iter().flat_map(|m| m.code.iter().copied()).collect();
    
    let out_tp_offsets: Vec<u32> = cfg.ir_drra_config.tps.iter().map(|m| m.offset).collect();
    let out_tp_lengths: Vec<u32> = cfg.ir_drra_config.tps.iter().map(|m| m.code.len() as u32).collect();
    let out_tp_code_flat: Vec<u32> = cfg.ir_drra_config.tps.iter().flat_map(|m| m.code.iter().copied()).collect();
       
    let drra_ctx = DrraTemplateCtx {
        rows: cfg.ir_drra_config.rows,
        cols: cfg.ir_drra_config.cols,
        num_in_tlbs: cfg.ir_drra_config.in_tlbs.len() as u32,
        num_out_tlbs: cfg.ir_drra_config.out_tlbs.len() as u32,
        num_out_tps: cfg.ir_drra_config.tps.len() as u32,
    
        drra_insts_raw_length: cfg.ir_drra_config.insts_raw.len(),
        drra_insts_raw: utils::vec_to_c_array(&cfg.ir_drra_config.insts_raw),
        drra_offset_cells: utils::vec2d_to_c(&cfg.ir_drra_config.insts_offset_cells),
        drra_num_insts: utils::vec2d_to_c(&cfg.ir_drra_config.num_insts_cells),
        drra_start_addrs: utils::vec2d_to_c(&cfg.ir_drra_config.start_addr_cells),
    
        in_tlb_pre_ptrs: utils::vec_to_c_array(&in_tlb_pre_ptrs),
        in_tlb_pres: utils::vec_to_c_array(&in_tlb_pres),
        in_tlb_offsets: utils::vec_to_c_array(&in_tlb_offsets),
        in_tlb_lengths: utils::vec_to_c_array(&in_tlb_lengths),
        in_tlb_code_flat: utils::vec_to_c_array(&in_tlb_code_flat),
    
        out_tlb_pre_ptrs: utils::vec_to_c_array(&out_tlb_pre_ptrs),
        out_tlb_pres: utils::vec_to_c_array(&out_tlb_pres),
        out_tlb_offsets: utils::vec_to_c_array(&out_tlb_offsets),
        out_tlb_lengths: utils::vec_to_c_array(&out_tlb_lengths),
        out_tlb_code_flat: utils::vec_to_c_array(&out_tlb_code_flat),
    
        out_tp_offsets: utils::vec_to_c_array(&out_tp_offsets),
        out_tp_lengths: utils::vec_to_c_array(&out_tp_lengths),
        out_tp_code_flat: utils::vec_to_c_array(&out_tp_code_flat),
    };

    let drra_context = Context::from_serialize(&drra_ctx)?;
    let drra_rendered = TERA.render("drra_config.c", &drra_context)
        .map_err(|e| {
        eprintln!("Tera error:\n{}", e);
        e
    })?;
    let drra_file_name = format!("{}/{}", dir, "drra_config.c");
    file_handler::write_file(&drra_file_name, drra_rendered)?;

    // ------------------ write kernel.o --------------
    let kernel_file_name = format!("{}/{}", dir, cfg.kernel_object.name);
    file_handler::write_file(&kernel_file_name, &cfg.kernel_object.data)?;

    // ------------------ write sections.lds --------------
    let linker_ctx = LinkerTemplateCtx {
        inst_mem_length: utils::to_hex(cfg.arch_config.inst_mem_length),
        data_mem_length: utils::to_hex(cfg.arch_config.data_mem_length),
        share_mem_length: utils::to_hex(cfg.arch_config.share_mem_length),
        part1_size: utils::to_hex(cfg.arch_config.part1_size),
        part2_size: utils::to_hex(cfg.arch_config.part2_size),
    };

    let linker_context = Context::from_serialize(&linker_ctx)?;
    let linker_rendered = TERA.render("sections.lds", &linker_context)
        .map_err(|e| {
        eprintln!("Tera error:\n{}", e);
        e
    })?;
    let linker_file_name = format!("{}/{}", dir, "sections.lds");
    file_handler::write_file(&linker_file_name, linker_rendered)?;

    // ----------------------------------------------------
    // compilation
    // ----------------------------------------------------
    let work_path = std::path::Path::new(&work_dir);

    // make clean
    utils::runc(
        std::process::Command::new("make")
            .arg("clean")
            .current_dir(work_path)
    )?;
 
    // mkdir build
    std::fs::create_dir_all(&work_path.join("build"))?;

    // copy drra_config.c → src/
    let drra_src = std::path::Path::new(dir).join("drra_config.c");
    let drra_dst = work_path.join("src").join("drra_config.c");
    std::fs::copy(&drra_src, &drra_dst)?;
    
    // copy kernel.o → build/
    let kernel_src = std::path::Path::new(dir).join("kernel.o");
    let kernel_dst = work_path.join("build").join("kernel.o");
    std::fs::copy(&kernel_src, &kernel_dst)?;

    // copy sections.lds → ld/
    let lds_src = std::path::Path::new(dir).join("sections.lds");
    let lds_dst = work_path.join("ld").join("sections.lds");
    std::fs::copy(&lds_src, &lds_dst)?;
    
    // make manager
    utils::runc(
        std::process::Command::new("make")
            .arg("manager")
            .current_dir(work_path)
    )?;
    
    // make link
    utils::runc(
        std::process::Command::new("make")
            .arg("link")
            .current_dir(work_path)
    )?;

    // ----------------------------------------------------
    // built outputs 
    // ----------------------------------------------------
    let work_build = work_path.join("build");
    let out_build = std::path::Path::new(dir).join("build");
    
    std::fs::create_dir_all(&out_build)?;
    
    let files = ["firmware.elf"];
    
    for file in &files {
        let src = work_build.join(file);
        let dst = out_build.join(file);
    
        if !src.exists() {
            return Err(format!("Missing build artifact: {:?}", src).into());
        }
    
        std::fs::copy(&src, &dst)
            .map_err(|e| format!("Failed to copy {:?} -> {:?}: {}", src, dst, e))?;
    }
    
    // store firmware.elf path
    cfg.firmware_path = out_build.join("firmware.elf");

    Ok(())
}


fn hardware_settings(
    db: &mut DataBase, 
    node_id: &String,
) -> Result<(), Box<dyn std::error::Error>> {

    let node = db
        .app_graph
        .nodes
        .iter()
        .find(|n| n.id == *node_id)
        .ok_or_else(|| format!("Node {} not found", node_id))?;

    let in_edges: std::collections::HashSet<_> =
        node.input_ports.iter().map(|p| p.id.clone()).collect();

    let out_edges: std::collections::HashSet<_> =
        node.output_ports.iter().map(|p| p.id.clone()).collect();


    // get config object  
    let cfg = db
        .synthesized_information
        .control_synthesis
        .alimp_control_synthesis
        .get_mut(node_id)
        .ok_or("Missing config")?;

    let common_config = HardwareCommonConfig {
        axi_addr_width: 32,
        axi_data_width: 32,
        cpu_addr_width: 32,
        cpu_data_width: 32,
        cpu_instmem_depth: cfg.arch_config.inst_mem_length,
        cpu_datamem_depth: cfg.arch_config.data_mem_length,
        cpu_sharemem_depth: cfg.arch_config.share_mem_length,
        chunk_addr_width: 32,
        chunk_data_width: 128,
        id_bits: 32,
        tlb_program_addr_width: 20,
        tlb_agu_internal_width: 16,
        tp_start_bits: 32,
        tp_internal_col_msb: 19,
        tp_internal_col_lsb: 16,
        tp_internal_data_width: 16,
        tp_program_addr_width: 20,
        tp_program_data_width: 32,
    };

    let drra_config = HardwareDrraConfig {
        rows: cfg.ir_drra_config.rows,
        cols: cfg.ir_drra_config.cols,
        instr_data_width: 32,
        instr_addr_width: 16,
        instr_hops_width: 4,
        io_addr_width: 0,   // unused CPU DM memory
        dm_addr_width: 0,   // unused CPU DM memory
    };
    
    // =========================
    // IN TLB
    // =========================
    let mut in_io_config: Vec<HardwareIOConfig> = Vec::new();

    let mut in_memories: Vec<MemoryStructure> = db
        .synthesized_information
        .memory_synthesis
        .iter()
        .filter(|m| m.app_node_id == *node_id && m.memory_direction == "in")
        .flat_map(|m| m.memory_structure.clone())
        .collect();

    in_memories.sort_by_key(|m| {
        m.output_channels.iter().min().cloned().unwrap_or(u32::MAX)
    });

    let mut in_tlb_entries: HashMap<i32, &TranslationTable> = HashMap::new();

    for at in db
        .synthesized_information
        .address_translations
        .iter()
        .filter(|t| t.app_node_id == *node_id && in_edges.contains(&t.port_id))
    {
        for (ch, tt) in &at.translation_table {
            in_tlb_entries.insert(*ch, tt);
        }
    }

    let mut skip = false;
    for col in 0..drra_config.cols {
        let mut config = HardwareIOConfig {
            active: false,
            skip: skip,
            buf_type: "".to_string(),
            buf_size: 0,
            block_size: 1,
            input1: -1,
            input2: -1,
            output1: -1,
            output2: -1,
            tlb1: HardwareTLBConfig {
                tlb_type: "NONE".to_string(),
                program_size: 0,
            },
            tlb2: HardwareTLBConfig {
                tlb_type: "NONE".to_string(),
                program_size: 0,
            },
        };
       
        skip = false;

        if !in_memories.is_empty() {
            let memory_col = in_memories[0].output_channels.iter().min().cloned().unwrap_or(u32::MAX);
            
            // assign memory implementation 
            if col == memory_col {
                let memory = in_memories.remove(0);
                
                config.active = true;
                config.buf_size = memory.memory_size;
                config.buf_type = match memory.memory_type.as_str() {
                    "fifo" => "BUF_FIFO".to_string(),
                    "rf" => "BUF_RF".to_string(),
                    "ram" => "BUF_RAM".to_string(),
                    _ => return Err("Cannot parse memory_type from memory synthesis".into()),
                };
                config.block_size = std::cmp::max(
                    memory.input_channels.len(),
                    memory.output_channels.len(),
                ) as u32;
                config.input1 = memory.input_channels[0] as i32;
                if memory.input_channels.len() > 1 {
                    config.input2 = memory.input_channels[1] as i32;
                }
                config.output1 = memory.output_channels[0] as i32;
                if memory.output_channels.len() > 1 {
                    config.output2 = memory.output_channels[1] as i32;
                }

                // geometry limitations 
                if config.input1 != col as i32 {
                    return Err("Fail to assign memory implmention - geometry limitaion".into());
                }
                if config.output1 != col as i32 {
                    return Err("Fail to assign memory implmention - geometry limitaion".into());
                }
                if config.input2 >= 0 {
                    if config.input2 != col as i32 + 1 { 
                        return Err("Fail to assign memory implmention - geometry limitaion".into());
                    }
                }
                if config.output2 >= 0 {
                    if config.output2 != col as i32 + 1 { 
                        return Err("Fail to assign memory implmention - geometry limitaion".into());
                    }
                }
                
                // assign TLB information
                // For IB, TLBs sit at the output side of the memory
                let tlb1 = in_tlb_entries
                    .get(&config.input1)
                    .ok_or("Fail to assign memory implementation - TLB1")?;
                
                if config.buf_type == "BUF_FIFO" {
                    if config.block_size != 1 {
                        return Err("Fail to assign memory implmention".into());
                    }
                    config.tlb1.tlb_type = "TLB_DUMMY".to_string();
                } else {
                    match &tlb1.implementation {
                        TLBImplementation::AGU { .. } => {
                            config.tlb1.tlb_type = "TLB_AGU".to_string();
                            config.tlb1.program_size = 0;
                        }
                        TLBImplementation::TLB { size, .. } => {
                            config.tlb1.tlb_type = "TLB_TLB".to_string();
                            config.tlb1.program_size = *size;
                        }
                    }
                }

                if config.output2 != -1 {
                    let tlb2 = in_tlb_entries
                        .get(&config.input2)
                        .ok_or("Fail to assign memory implementation - TLB2")?;
                    
                    if config.buf_type == "BUF_FIFO" {
                        return Err("Fail to assign memory implmention".into());
                    } else {
                        match &tlb2.implementation {
                            TLBImplementation::AGU { .. } => {
                                config.tlb2.tlb_type = "TLB_AGU".to_string();
                                config.tlb2.program_size = 0;
                            }
                            TLBImplementation::TLB { size, .. } => {
                                config.tlb2.tlb_type = "TLB_TLB".to_string();
                                config.tlb2.program_size = *size;
                            }
                        }
                    }
                }

                // skip next channel 
                if config.block_size == 2 {
                    skip = true;
                }

            } else if col > memory_col {
                return Err("Fail to assign memory implemention".into());
            }
        }

        in_io_config.push(config);
    }

    if !in_memories.is_empty() {
        return Err("Fail to assign all memory implementions - some implementations are unassigned".into());
    }

    // =========================
    // OUT TLB
    // =========================
    let mut out_io_config: Vec<HardwareIOConfig> = Vec::new();

    let mut out_memories: Vec<MemoryStructure> = db
        .synthesized_information
        .memory_synthesis
        .iter()
        .filter(|m| m.app_node_id == *node_id && m.memory_direction == "out")
        .flat_map(|m| m.memory_structure.clone())
        .collect();

    out_memories.sort_by_key(|m| {
        m.input_channels.iter().min().cloned().unwrap_or(u32::MAX)
    });

    let mut out_tlb_entries: HashMap<i32, &TranslationTable> = HashMap::new();

    for at in db
        .synthesized_information
        .address_translations
        .iter()
        .filter(|t| t.app_node_id == *node_id && out_edges.contains(&t.port_id))
    {
        for (ch, tt) in &at.translation_table {
            out_tlb_entries.insert(*ch, tt);
        }
    }

    skip = false;

    let mut out_tp_positions: Vec<i32> = Vec::new();

    for col in 0..drra_config.cols {
        let mut config = HardwareIOConfig {
            active: false,
            skip: skip,
            buf_type: "".to_string(),
            buf_size: 0,
            block_size: 1,
            input1: -1,
            input2: -1,
            output1: -1,
            output2: -1,
            tlb1: HardwareTLBConfig {
                tlb_type: "NONE".to_string(),
                program_size: 0,
            },
            tlb2: HardwareTLBConfig {
                tlb_type: "NONE".to_string(),
                program_size: 0,
            },
        };
       
        skip = false;

        if !out_memories.is_empty() {
            let memory_col = out_memories[0].input_channels.iter().min().cloned().unwrap_or(u32::MAX);
            
            // assign memory implementation 
            if col == memory_col {
                let memory = out_memories.remove(0);
                
                config.active = true;
                config.buf_size = memory.memory_size;
                config.buf_type = match memory.memory_type.as_str() {
                    "fifo" => "BUF_FIFO".to_string(),
                    "rf" => "BUF_RF".to_string(),
                    "ram" => "BUF_RAM".to_string(),
                    _ => return Err("Cannot parse memory_type from memory synthesis".into()),
                };
                config.block_size = std::cmp::max(
                    memory.input_channels.len(),
                    memory.output_channels.len(),
                ) as u32;
                config.input1 = memory.input_channels[0] as i32;
                if memory.input_channels.len() > 1 {
                    config.input2 = memory.input_channels[1] as i32;
                }
                config.output1 = memory.output_channels[0] as i32;
                out_tp_positions.push(config.output1);
                if memory.output_channels.len() > 1 {
                    config.output2 = memory.output_channels[1] as i32;
                    out_tp_positions.push(config.output2);
                }

                // geometry limitations 
                if config.input1 != col as i32 {
                    return Err("Fail to assign memory implmention - geometry limitaion".into());
                }
                if config.output1 != col as i32 {
                    return Err("Fail to assign memory implmention - geometry limitaion".into());
                }
                if config.input2 >= 0 {
                    if config.input2 != col as i32 + 1 { 
                        return Err("Fail to assign memory implmention - geometry limitaion".into());
                    }
                }
                if config.output2 >= 0 {
                    if config.output2 != col as i32 + 1 { 
                        return Err("Fail to assign memory implmention - geometry limitaion".into());
                    }
                }

                // assign TLB information
                // For OB, TLBs sit at the input side of the memory
                let tlb1 = out_tlb_entries
                    .get(&config.input1)
                    .ok_or("Fail to assign memory implementation - TLB1")?;
                
                if config.buf_type == "BUF_FIFO" {
                    if config.block_size != 1 {
                        return Err("Fail to assign memory implmention".into());
                    }
                    config.tlb1.tlb_type = "TLB_DUMMY".to_string();
                } else {
                    match &tlb1.implementation {
                        TLBImplementation::AGU { .. } => {
                            config.tlb1.tlb_type = "TLB_AGU".to_string();
                            config.tlb1.program_size = 0;
                        }
                        TLBImplementation::TLB { size, .. } => {
                            config.tlb1.tlb_type = "TLB_TLB".to_string();
                            config.tlb1.program_size = *size;
                        }
                    }
                }

                if config.input2 != -1 {
                    let tlb2 = out_tlb_entries
                        .get(&config.input2)
                        .ok_or("Fail to assign memory implementation - TLB2")?;
                    
                    if config.buf_type == "BUF_FIFO" {
                        return Err("Fail to assign memory implmention".into());
                    } else {
                        match &tlb2.implementation {
                            TLBImplementation::AGU { .. } => {
                                config.tlb2.tlb_type = "TLB_AGU".to_string();
                                config.tlb2.program_size = 0;
                            }
                            TLBImplementation::TLB { size, .. } => {
                                config.tlb2.tlb_type = "TLB_TLB".to_string();
                                config.tlb2.program_size = *size;
                            }
                        }
                    }
                    
                }

                // skip next channel 
                if config.block_size == 2 {
                    skip = true;
                }

            } else if col > memory_col {
                return Err("Fail to assign memory implemention".into());
            }
        }

        out_io_config.push(config);
    }

    if !out_memories.is_empty() {
        return Err("Fail to assign all memory implementions - some implementations are unassigned".into());
    }

    // =========================
    // OUT TP
    // =========================
    let mut out_tp_config: Vec<HardwareTPConfig> = Vec::new();

    let mut tp_indices: Vec<usize> = db
        .synthesized_information
        .transporter_tables
        .iter()
        .enumerate()
        .filter(|(_, tt)| tt.transporter_id.starts_with(&format!("transporter_{}_", node_id)))
        .map(|(i, _)| i)
        .collect();

    tp_indices.sort_by_key(|&i| {
        db.synthesized_information.transporter_tables[i].placement.x
    });

    if tp_indices.len() != out_tp_positions.len() {
        return Err("Fail to assign transporters - mismatch number of tps".into());
    }

    for col in 0..drra_config.cols {
        let mut config = HardwareTPConfig {
            active: false,
            program_size: 0,
            last: 0,
        };

        if !out_tp_positions.is_empty() {
           if col as i32 == out_tp_positions[0] {
                let _ = out_tp_positions.remove(0);
                let tt_idx = tp_indices.remove(0);
                let tp = &db.synthesized_information.transporter_tables[tt_idx];

                config.active = true;
                config.program_size = if tp.size == 0 {
                    return Err("Fail to assign a transporter - program size".into())
                } else {
                    tp.size.next_power_of_two()
                };
                if out_tp_positions.is_empty() {
                    config.last = 1;
                }
           }
        }

        out_tp_config.push(config);
    }

    if !out_tp_positions.is_empty() {
        return Err("Fail to assign all transporter implementions - some implementations are unassigned".into()); 
    }

    // =========================
    // update hardware settings
    // =========================

    cfg.hardware_config.hardware_common_config = common_config; 
    cfg.hardware_config.hardware_drra_config = drra_config;
    cfg.hardware_config.in_io_config = in_io_config;
    cfg.hardware_config.out_io_config = out_io_config;
    cfg.hardware_config.out_tp_config = out_tp_config;

    Ok(())
}


fn local_synchronisation(
    db: &mut DataBase, 
    node_id: &String,
) -> Result<(), Box<dyn std::error::Error>> {

    let node_fire_time: i32 = *db
        .synthesized_information
        .node_fire_times
        .get(node_id)
        .ok_or_else(|| format!("node {} cannot be found in node_fire_times", node_id))?;

    let all_fire_times = &db.synthesized_information.node_fire_times;

    let cfg = db
        .synthesized_information
        .control_synthesis
        .alimp_control_synthesis
        .get_mut(node_id)
        .ok_or_else(|| format!("Missing config of node {}", node_id))?;
    
    let mut id: u32 = 0;

    for col in 0..cfg.hardware_config.out_tp_config.len() {
        if cfg.hardware_config.out_tp_config[col].active {
            let fire_time = all_fire_times
                .iter()
                .find_map(|(k, v)| {
                    let prefix = format!("transporter_{}_", node_id);
                    let suffix = format!("_{}", id);

                    if k.starts_with(&prefix) && k.ends_with(&suffix) {
                        Some(*v)
                    } else {
                        None
                    }
                })
                .ok_or_else(|| format!("Missing fire_time for transporter_{}_x_{}", node_id, id))?;
            
            // start_time = fire_time - Rd latency - start up cost - execution - col
            let tp_start_time = fire_time - 1 - 3 - 1 - col as i32;        
            let relative_delay = tp_start_time - node_fire_time;

            cfg.synchronisation.insert(id, relative_delay);
            
            id += 1;
        }
    }

    Ok(())
}




pub fn main(
    db: &mut DataBase, 
    node_id: &String,
    dir: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    let module_dir = format!("{}/{}", dir, node_id);
    match std::fs::create_dir_all(&module_dir) {
        Ok(_) => (),
        Err(e) => {
            error!("Failed to create {} with {}", module_dir, e);
            return Err(Box::new(e));
        },
    };

    if db.synthesized_information.control_synthesis.alimp_control_synthesis.is_empty() {
        db.synthesized_information.control_synthesis.alimp_control_synthesis = HashMap::new();
    }
    
    // Insert default DRRA config for this node
    db.synthesized_information
        .control_synthesis
        .alimp_control_synthesis
        .insert(node_id.clone(), AlimpControlSynthesis::default());

    let alimp_dir = format!("{}/_work/integration/alimp", dir);
    
    generate_firmware_code(db, node_id, &format!("{}/firmware", alimp_dir), &module_dir)?;
    hardware_settings(db, node_id)?;
    local_synchronisation(db, node_id)?;

    Ok(())
}

