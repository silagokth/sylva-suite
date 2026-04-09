use sv_lib::model::{DataBase, AlimpControlSynthesis, DrraConfig,
                    ArchConfig, TlbBlock, TpBlock, TranslationTable}; 
use sv_lib::{file_handler};
use std::collections::{HashMap};
use log::{error};
use serde::Serialize;
use tera::{Tera, Context};
use once_cell::sync::Lazy;

static TERA: Lazy<Tera> = Lazy::new(|| {
    let mut tera = Tera::default();
    tera.add_raw_template(
        "drra_config.c",
        include_str!("templates/drra_config.c.tmpl")
    ).unwrap();
    tera.add_raw_template(
        "sections.lds",
        include_str!("templates/sections.lds.tmpl")
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

fn to_hex(u: u32) -> String {
    format!("0x{:08X}", u)
}

fn vec_to_c_array(v: &[u32]) -> String {
    v.iter()
        .map(|x| format!("0x{:08X}", x))
        .collect::<Vec<_>>()
        .join(", ")
}

fn vec2d_to_c(v: &[Vec<u32>]) -> String {
    v.iter()
        .map(|row| {
            let row_str = row.iter()
                .map(|x| format!("{}", x))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{{{}}}", row_str)
        })
        .collect::<Vec<_>>()
        .join(", ")
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
        inst_mem_length: 1024,  // FIXED
        data_mem_length: 0,     // FLEXIBLE
        share_mem_length: 128,  // FIXED
        part1_size: 768,        // FIXED
        part2_size: 256,        // FIXED
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
        drra_insts_raw: vec_to_c_array(&cfg.ir_drra_config.insts_raw),
        drra_offset_cells: vec2d_to_c(&cfg.ir_drra_config.insts_offset_cells),
        drra_num_insts: vec2d_to_c(&cfg.ir_drra_config.num_insts_cells),
        drra_start_addrs: vec2d_to_c(&cfg.ir_drra_config.start_addr_cells),
    
        in_tlb_pre_ptrs: vec_to_c_array(&in_tlb_pre_ptrs),
        in_tlb_pres: vec_to_c_array(&in_tlb_pres),
        in_tlb_offsets: vec_to_c_array(&in_tlb_offsets),
        in_tlb_lengths: vec_to_c_array(&in_tlb_lengths),
        in_tlb_code_flat: vec_to_c_array(&in_tlb_code_flat),
    
        out_tlb_pre_ptrs: vec_to_c_array(&out_tlb_pre_ptrs),
        out_tlb_pres: vec_to_c_array(&out_tlb_pres),
        out_tlb_offsets: vec_to_c_array(&out_tlb_offsets),
        out_tlb_lengths: vec_to_c_array(&out_tlb_lengths),
        out_tlb_code_flat: vec_to_c_array(&out_tlb_code_flat),
    
        out_tp_offsets: vec_to_c_array(&out_tp_offsets),
        out_tp_lengths: vec_to_c_array(&out_tp_lengths),
        out_tp_code_flat: vec_to_c_array(&out_tp_code_flat),
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
        inst_mem_length: to_hex(cfg.arch_config.inst_mem_length),
        data_mem_length: to_hex(cfg.arch_config.data_mem_length),
        share_mem_length: to_hex(cfg.arch_config.share_mem_length),
        part1_size: to_hex(cfg.arch_config.part1_size),
        part2_size: to_hex(cfg.arch_config.part2_size),
    };

    let linker_context = Context::from_serialize(&linker_ctx)?;
    let linker_rendered = TERA.render("sections.lds", &linker_context)
        .map_err(|e| {
        eprintln!("Tera error:\n{}", e);
        e
    })?;
    let linker_file_name = format!("{}/{}", dir, "sections.lds");
    file_handler::write_file(&linker_file_name, linker_rendered)?;

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

    generate_firmware_code(db, node_id, &module_dir)?;

    Ok(())
}

