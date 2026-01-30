use sv_lib::model::{DataBase, TransporterTable, TransporterISA}; 
use sv_lib::file_handler;
use log::{info, error};
use std::collections::{HashMap, BTreeMap};

pub mod utils;
mod register_allocation;


// map a sequence of data patterns to assembly
// without optimisation and register allocation is ideal
// (unlimited number of registers)
fn pass0(
   transporter_table: &mut TransporterTable,
) -> Result<(), Box<dyn std::error::Error>> {

    let mut tmp_regs: BTreeMap<i32, (i32, i32)> = BTreeMap::new(); 
  
    // 1. Sort entries by relativeTime (stable keeps original order for ties)
    transporter_table.entries.sort_by_key(|e| e.relative_time);

    // 2. Build indices and required values
    for entry in &transporter_table.entries {
        let time = entry.relative_time;

        tmp_regs.insert(
            time,
            (
                entry.source_address,
                entry.target_address,
            ),
        );
    }

    // 3. temporal register allocation 
    let mut register_allocs: Vec<i32> = Vec::new();
    let mut ir: BTreeMap<i32, TransporterISA> = BTreeMap::new();

    while let Some((&time_index, _)) = tmp_regs.iter_mut().next() {
        let (source, target) = tmp_regs.remove(&time_index).unwrap();

        register_allocs.push(source);
        let reg_source = register_allocs.len() as u32; // preserving r0 for zero value 
        
        register_allocs.push(target);
        let reg_target = register_allocs.len() as u32; // preserving r0 for zero value

        if ir.contains_key(&time_index) {
            return Err(format!("Time slot {} already occupied", time_index).into());
        }

        // MOV at time_index
        ir.insert(
            time_index,
            TransporterISA::MOV {
                r0: reg_target,
                r1: reg_source,
                immediate: 0,
            },
        );

        let free_times = utils::find_free_times_before(&ir, 0, 2);
        let t1 = free_times[0];
        let t2 = free_times[1];
        
        if ir.contains_key(&t1) || ir.contains_key(&t2) {
            return Err(format!("find_free_times_before failed to find free time slots").into());
        }

        ir.insert(
            t1,
            TransporterISA::LDI {
                r0: reg_source,
                immediate: source,
            },
        );

        ir.insert(
            t2,
            TransporterISA::LDI {
                r0: reg_target,
                immediate: target,
            },
        );
    }
    
    transporter_table.ir = ir;
    Ok(())
}


// get rid of the redundant registers 
fn pass1(
   transporter_table: &mut TransporterTable,
) -> Result<(), Box<dyn std::error::Error>> {
            
    let mut ir = transporter_table.ir.clone(); 
    
    let mov_indices: Vec<i32> = utils::list_all_mov_indices(&ir);
    let mut allocated_regs: HashMap<u32, i32> = HashMap::new(); // number to value

    for index in mov_indices.iter() {
        // get mutable reference to MOV
        let (reg0_number, reg1_number) = {
            let inst = ir.get(index)
                .ok_or_else(|| format!("No instruction at time {}", index))?;

            match inst {
                TransporterISA::MOV { r0, r1, .. } => (*r0, *r1),
                other => {
                    return Err(format!(
                        "Expect MOV, found {:?}", other
                    ).into());
                }
            }
        };
    
        let mut new_reg0_number = reg0_number;
        let mut new_reg1_number = reg1_number;
        let mut remove_reg0 = false;
        let mut remove_reg1 = false;

        // find LDI for both registers
        let (reg0_time, reg0_value) =
            utils::find_load_register_with_number(&ir, reg0_number)
                .ok_or_else(|| format!("No LDI found for r{}", reg0_number))?;

        let (reg1_time, reg1_value) =
            utils::find_load_register_with_number(&ir, reg1_number)
                .ok_or_else(|| format!("No LDI found for r{}", reg1_number))?;

        // zero-value optimisation
        if reg0_value == 0 {
            new_reg0_number = 0;
            remove_reg0 = true;
        }    
    
        if reg1_value == 0 {
            new_reg1_number = 0;
            remove_reg1 = true;
        }

        // same immediate optimisation 
        if reg0_value == reg1_value {
            new_reg1_number = new_reg0_number;
            remove_reg1 = true;
        } 
        
        // reuse already allocated registers 
        for (&allocated_reg, &allocated_value) in allocated_regs.iter() {
            if reg0_value == allocated_value {
                new_reg0_number = allocated_reg;
                remove_reg0 = true;
            }
            if reg1_value == allocated_value {
                new_reg1_number = allocated_reg;
                remove_reg1 = true;
            }
        }

        // remove or keep LDIs
        if remove_reg0 {
            ir.remove(&reg0_time);
        } else {
            allocated_regs.insert(reg0_number, reg0_value);
        }

        if remove_reg1 {
            ir.remove(&reg1_time);
        } else {
            allocated_regs.insert(reg1_number, reg1_value);
        }

        // rewrite the current MOV instruction
        if let Some(TransporterISA::MOV { r0, r1, .. }) = ir.get_mut(index) {
            *r0 = new_reg0_number;
            *r1 = new_reg1_number;
        }
    }

    transporter_table.ir = ir;
    Ok(())
}


// merging MOVs by using MOVC
fn pass2(
   transporter_table: &mut TransporterTable,
) -> Result<(), Box<dyn std::error::Error>> {
            
    let mut ir = transporter_table.ir.clone(); 
    
    let mov_indices: Vec<i32> = utils::list_all_mov_indices(&ir);
    let mut cursor = 0;

    while cursor < mov_indices.len() {
        let index = mov_indices[cursor];

        // ---------- Phase 1: analyze first MOV ----------
        let (reg0, reg1) = match ir.get(&index) {
            Some(TransporterISA::MOV { r0, r1, .. }) => (*r0, *r1),
            _ => {
                cursor += 1;
                continue;
            }
        };

        let (_, reg0_value) =
            utils::find_load_register_with_number(&ir, reg0)
                .ok_or_else(|| format!("No LDI for r{}", reg0))?;

        let (_, reg1_value) =
            utils::find_load_register_with_number(&ir, reg1)
                .ok_or_else(|| format!("No LDI for r{}", reg1))?;

        // objective: 
        // 1. finding c where r0_next = r0 + c and r1_next = r1 + c
        // 2. finding the number of iteration
        let mut c: Option<i32> = None;
        let mut iter_count = 0;


        // ---------- Phase 2: scan forward ----------
        let mut lookahead = cursor + 1;

        while lookahead < mov_indices.len() {
            let next_index = mov_indices[lookahead];
            
            // continuity check
            if next_index != index + iter_count as i32 + 1 {
                break;
            }

            let (r0n, r1n) = match ir.get(&next_index) {
                Some(TransporterISA::MOV { r0, r1, .. }) => (*r0, *r1),
                _ => break,
            };

            let (_, r0v) =
                utils::find_load_register_with_number(&ir, r0n)
                    .ok_or_else(|| format!("No LDI for r{}", r0n))?;

            let (_, r1v) =
                utils::find_load_register_with_number(&ir, r1n)
                    .ok_or_else(|| format!("No LDI for r{}", r1n))?;

            if iter_count == 0 {
                let delta = (r0v + r1v) - (reg0_value + reg1_value);
                if delta % 2 != 0 {
                    break;
                }
                c = Some(delta / 2);
            } else {
                let c = c.unwrap();
                if r0v != reg0_value + (iter_count as i32) * c
                    || r1v != reg1_value + (iter_count as i32) * c
                {
                    break;
                }
            }

            iter_count += 1;
            lookahead += 1;
        }

        // ---------- Phase 3: rewrite ----------
        if iter_count > 0 {
            let c = c.unwrap();
            
            // remove merged MOVs
            for i in 1..=iter_count {
                let t = mov_indices[cursor + i];
                ir.insert(t, TransporterISA::OCCUPIED);
            }
            
            // find or create register holding c
            let mut reg2 = None;
            for (r, v) in utils::list_all_registers(&ir) {
                if c == 0 {
                    reg2 = Some(0);
                    break;
                }
                if c == v {
                    reg2 = Some(r);
                    break;
                }
            }

            let reg2 = if let Some(r) = reg2 {
                r
            } else {
                let new_reg = utils::list_all_registers(&ir)
                    .iter()
                    .map(|(r, _)| *r)
                    .max()
                    .unwrap_or(0) + 1;

                let time_index = utils::find_free_times_before(&ir, 0, 1);     
                ir.insert(time_index[0], TransporterISA::LDI {
                    r0: new_reg,
                    immediate: c,
                });
                new_reg
            };

            // replace first MOV with MOVC
            ir.insert(
                index,
                TransporterISA::MOVC {
                    r0: reg0,
                    r1: reg1,
                    r2: reg2,
                    immediate: (iter_count + 1) as i32,
                },
            );

            cursor += iter_count + 1;
        } else {
            cursor += 1;
        }
    }

    transporter_table.ir = ir;
    Ok(())
}

// analysing the MOV instructions to make use of immediate values
fn pass3(
   transporter_table: &mut TransporterTable,
) -> Result<(), Box<dyn std::error::Error>> {
 
    let mut ir = transporter_table.ir.clone(); 
        
    // order the MOV operations to be checked 
    // with the number of register dependency counts
    let mut register_dependencies: HashMap<u32, u32> = HashMap::new();
    
    let mov_indices = utils::list_all_mov_indices(&ir);
    let reg_nums: Vec<u32> = utils::list_all_registers(&ir)
        .into_iter()
        .map(|(r, _)| r)
        .collect();

    for &reg_num in &reg_nums {
        let deps = utils::get_number_of_dependencies(
            &ir,
            &mov_indices,
            reg_num,
        )?;

        register_dependencies.insert(reg_num, deps);
    }

    // MOV index -> (dependency count, used flag)
    let mut mov_reg_deps: HashMap<i32, (u32, bool)> = HashMap::new();
    
    for &index in &mov_indices {
        let (reg0, reg1) = match ir.get(&index) {
            Some(TransporterISA::MOV { r0, r1, .. }) => (*r0, *r1),
            Some(TransporterISA::MOVC { .. }) => continue,
            _ => return Err(format!("Expect MOV, found at {}", index).into()),
        };
        
        let deps_of_reg0 = *register_dependencies.get(&reg0).unwrap_or(&0); 
        let deps_of_reg1 = *register_dependencies.get(&reg1).unwrap_or(&0); 
        
        mov_reg_deps.insert(
            index,
            (deps_of_reg0 + deps_of_reg1, false),
        ); 
    }

    // sort MOVs by dependency count
    let mut movs_by_deps: Vec<(i32, u32)> = mov_reg_deps
        .iter()
        .map(|(&idx, &(deps, _))| (idx, deps))
        .collect();
    
    movs_by_deps.sort_by_key(|&(_, deps)| deps);

    let sorted_low_to_high_indices: Vec<i32> =
        movs_by_deps.iter().map(|&(i, _)| i).collect();

    let sorted_high_to_low_indices: Vec<i32> =
        movs_by_deps.iter().rev().map(|&(i, _)| i).collect();
  
    // matching algorithm
    for &index in &sorted_low_to_high_indices {
        // This MOV is already consumed
        if let Some((_, true)) = mov_reg_deps.get(&index) {
            continue;
        }
        
        let (reg0, reg1) = match ir.get(&index) {
            Some(TransporterISA::MOV { r0, r1, .. }) => (*r0, *r1),
            _ => return Err(format!("Expect MOV, found at {}", index).into()),
        };
    
        let (_, reg0_value) =
            utils::find_load_register_with_number(&ir, reg0)
                .ok_or_else(|| format!("No LDI for r{}", reg0))?;
        
        let (_, reg1_value) =
            utils::find_load_register_with_number(&ir, reg1)
                .ok_or_else(|| format!("No LDI for r{}", reg1))?;
       
        for &match_index in &sorted_high_to_low_indices {
            if index == match_index {
                continue;
            }
            
            let (reg2, reg3) = match ir.get(&match_index) {
                Some(TransporterISA::MOV { r0, r1, .. }) => (*r0, *r1),
                _ => return Err(format!("Expect MOV, found at {}", match_index).into()),
            };
     
            let (_, reg2_value) =
                utils::find_load_register_with_number(&ir, reg2)
                    .ok_or_else(|| format!("No LDI for r{}", reg2))?;

            let (_, reg3_value) =
                utils::find_load_register_with_number(&ir, reg3)
                    .ok_or_else(|| format!("No LDI for r{}", reg3))?;
            
            if reg3_value - reg2_value == reg1_value - reg0_value {
                let immediate_value = reg1_value - reg3_value;            
    
                if utils::fits_signed_n_bit(7, immediate_value) {
                    if let Some(TransporterISA::MOV { r0, r1, immediate }) = ir.get_mut(&index) {
                        *r0 = reg2;
                        *r1 = reg3;
                        *immediate = reg1_value - reg3_value;
                    }
                    
                    mov_reg_deps.get_mut(&index).unwrap().1 = true;
                    mov_reg_deps.get_mut(&match_index).unwrap().1 = true;

                    break;
                }
            }
        }
    }

    transporter_table.ir = ir;
    Ok(())
}


// 1. remove unused registers
// 2. tighten the timing 
// 3. actual register allocation
// 4. fill NOPs
// 5. check value bound 
// 6. check/change/report fire time and latency
fn pass_sanity(
    transporter_table: &mut TransporterTable,
    module_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    
    // define constraints
    let maximum_number_of_registers: u32 = 16;

    fn dump_error_debug(
        transporter_table: &mut TransporterTable,
        ir: &BTreeMap<i32, TransporterISA>,
        module_dir: &str,
        error_text: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        transporter_table.ir = ir.clone();
        let _ = dump_transporter_ir(
                &transporter_table, 
                &module_dir,
                "pass_sanity_debug_ir",
                false,
        );
        return Err(error_text.into())
    }
    


    let mut ir = transporter_table.ir.clone(); 
 
    // 1. remove unused registers
    let mut register_dependencies: HashMap<u32, u32> = HashMap::new();
    
    let mov_indices = utils::list_all_mov_indices(&ir);
    let reg_nums: Vec<u32> = utils::list_all_registers(&ir)
        .into_iter()
        .map(|(r, _)| r)
        .collect();

    for &reg_num in &reg_nums {
        let deps = utils::get_number_of_dependencies(
            &ir,
            &mov_indices,
            reg_num,
        )?;

        register_dependencies.insert(reg_num, deps);
    }
    
    let unused_regs: Vec<u32> = register_dependencies
        .iter()
        .filter_map(|(&num, &deps)| {
            if deps == 0 {
                Some(num)
            } else {
                None
            }
        })
        .collect();
    
    for reg_num in unused_regs {
        if let Some((index, _)) = utils::find_load_register_with_number(&ir, reg_num) {
            ir.remove(&index);
        } else {
            return Err(format!("Expect LDI for r{}", reg_num).into());
        }
        
        register_dependencies.remove(&reg_num);
    }

    // 2. tighten the timing 
    let mut reg_deps_list: Vec<(u32, u32)> = register_dependencies
        .iter()
        .map(|(&idx, &deps)| (idx, deps))
        .collect();

    reg_deps_list.sort_by_key(|&(_, deps)| deps);

    let sorted_reg_deps: Vec<u32> = 
        reg_deps_list.iter().map(|&(r, _)| r).collect(); 

    // map: reg -> MOV/MOVC dependency times
    let mut mov_deps_times: HashMap<u32, Vec<i32>> = HashMap::new();
    
    for &reg_num in &sorted_reg_deps {
        let times = utils::list_mov_indices_to_reg(&ir, &mov_indices, reg_num)?;
        if times.is_empty() {
            return Err(format!("Expect MOV(C) dependencies").into());
        }

        mov_deps_times.insert(reg_num, times);
    }

    // iterative tightening
    let mut improvement = true;
    
    while improvement {    
        improvement = false;

        for &reg_num in &sorted_reg_deps {
            let time_indices = &mov_deps_times[&reg_num];
            let min_time = *time_indices.iter().min().unwrap();

            let current_time = match utils::find_load_register_with_number(&ir, reg_num) {
                Some((time, _)) => time,
                None => return Err(format!("Expect LDI for r{}", reg_num).into()),
            };

            let new_slots = utils::find_free_times_before(&ir, min_time, 1); 
            let new_time = new_slots[0];
            
            if new_time > current_time {
                let inst = ir
                    .get(&current_time)
                    .ok_or_else(|| format!("No instruction at {}", current_time))?
                    .clone();

                ir.insert(new_time, inst);
                ir.remove(&current_time);
                
                improvement = true;
            } 
        }
    }

    // 3. actual register allocation
    let mut number_of_spills = 0;
    let mut live_in; 
    let mut live_out;
    let mut graph_nodes; 
    let mut graph_edges;
    let mut colouring_result;
    let mut spilled_register;
    let mut colouring: Option<HashMap<u32, u32>> = None;
    loop {
        (live_in, live_out) = register_allocation::liveness(&ir);
        (graph_nodes, graph_edges) = register_allocation::interference_graph_construction(
            &ir,
            &live_out,
        );
        
        colouring_result = register_allocation::graph_colouring(
            &ir,
            &graph_nodes,
            &graph_edges,
            maximum_number_of_registers - 1,
        );

        match &colouring_result {
            Ok(a) => {
                colouring = Some(a.clone());
                break;
            }
            Err(utils::RegAllocError::SpilledRegister(reg)) => {
                spilled_register = *reg;
            }
            Err(_) => {       
                break;
            }
        }

        match register_allocation::transforming_ir(&mut ir, spilled_register) {
            Ok(_) => {},
            Err(e) => {
                colouring_result = Err(e);
                break;
            },
        }
        number_of_spills += 1;
    }

    let mut colouring = colouring
        .ok_or("Register allocation failed")?
        .into_iter()
        .map(|(k, v)| (k, v + 1)) // shift physical registers
        .collect::<HashMap<_, _>>();

    colouring.insert(0, 0); // r0 is special

    {
        // For debug purpose
        let mut debug_text = String::new();
        let _vertices: Vec<i32> = ir.keys().cloned().collect();
        debug_text.push_str(&format!("Number of spills = {}\n\n", number_of_spills));
        debug_text.push_str(&utils::format_liveness(&_vertices, &live_in, &live_out));
        debug_text.push_str(&utils::format_interference_graph(&graph_nodes, &graph_edges));
        debug_text.push_str(&utils::format_colouring(&colouring));
        
        let path = format!(
            "{}/{}_pass_sanity_debug_algo.txt",
            module_dir,
            transporter_table.transporter_id,
        );
        file_handler::write_file(&path, debug_text)?;
    }

    match colouring_result {
        Err(e) => {
            dump_error_debug(
                transporter_table,
                &ir,
                &module_dir,
                "Fail to allocate registers, please look at debug files",
            )?;
            return Err(format!("Register allocation failed - {}", e).into());
        },
        _ => {},
    }

    let register_mapping: HashMap<u32, u32> = colouring;

    for inst in ir.values_mut() {
        // all register number is to +1 because r0 is exclusive
        match inst {
            TransporterISA::LDI { r0, .. } => {
                *r0 = register_mapping[r0];
            }
            TransporterISA::MOV { r0, r1, .. } => {
                *r0 = register_mapping[r0];
                *r1 = register_mapping[r1];
            }
            TransporterISA::MOVC { r0, r1, r2, .. } => {
                *r0 = register_mapping[r0];
                *r1 = register_mapping[r1];
                *r2 = register_mapping[r2];
            }
            TransporterISA::CAL { r0, r1, r2, .. }
            | TransporterISA::CALI { r0, r1, r2, .. } => {
                *r0 = register_mapping[r0];
                *r1 = register_mapping[r1];
                *r2 = register_mapping[r2];
            }
            TransporterISA::BRN { r0, .. }
            | TransporterISA::NOPR { r0, .. } => {
                *r0 = register_mapping[r0];
            }
            _ => {}
        }
    }

    // 4. fill NOPs 
    let mut gap_count = 0;
    let mut cursor = *ir.keys().next().unwrap();
    let max_index = *ir.keys().next_back().unwrap();

    while cursor <= max_index {
        if !ir.contains_key(&cursor) {
            gap_count += 1;
        } else {
            if gap_count > 0 {
                let nop_pos = cursor - gap_count;
                ir.insert(
                    nop_pos,
                    TransporterISA::NOP {
                        immediate: gap_count,
                    }
                );
                gap_count = 0;
            }
        }
        cursor += 1;
    }

    if gap_count > 0 {
        return Err("Failed to fill NOP instructions because of trailing gap".into());
    }

    // fill wait forever at the end
    ir.insert(
        max_index + 1,
        TransporterISA::NOP {
            immediate: 0,
        }
    );

    // 5. check value bound 
    let all_registers: Vec<(u32, i32)> = utils::list_all_registers(&ir);
    
    for (_, value) in &all_registers {
        if !utils::fits_signed_n_bit(10, *value) {
            dump_error_debug(
                transporter_table,
                &ir,
                &module_dir,
                "Transporter code: LDI immediate out of signed 10-bit range",
            )?;
        }
    }

    let all_nop_indices = utils::list_all_nop_indices(&ir);

    for &index in &all_nop_indices {
        let immediate = match ir.get(&index) {
            Some(TransporterISA::NOP { immediate }) => *immediate,
            _ => {
                return Err(format!(
                    "Expected NOP at index {}, found something else",
                    index
                )
                .into())
            }
        };

        if !utils::fits_unsigned_n_bit(13, immediate) {
            dump_error_debug(
                transporter_table,
                &ir,
                &module_dir,
                "Transporter code: NOP immediate out of unsigned 13-bit range",
            )?;
        }
    }

    let all_mov_indices = utils::list_all_mov_indices(&ir);
    let mut movc_indices = Vec::new();
   
    for &index in &all_mov_indices {
        match ir.get(&index) {
            Some(TransporterISA::MOV { immediate, .. }) => {
                if !utils::fits_signed_n_bit(7, *immediate) {
                    dump_error_debug(
                        transporter_table,
                        &ir,
                        &module_dir,
                        "Transporter code: MOV immediate out of signed 7-bit range",
                    )?;
                }
            }
            Some(TransporterISA::MOVC { .. }) => {
                movc_indices.push(index);
            }
            Some(other) => {
                return Err(format!(
                    "Expected MOV or MOVC at {}, found {:?}",
                    index, other
                ).into());
            }
            None => {
                return Err(format!(
                    "No instruction at MOV index {}",
                    index
                ).into());
            }
        }
    }

    let max_movc_range = (1 << 4) - 1; 
    
    for &movc_index in &movc_indices {
        // spilling for MOVC
        let (r0, r1, r2, immediate) = match ir.get(&movc_index) {
            Some(TransporterISA::MOVC { r0, r1, r2, immediate }) => {
                (*r0, *r1, *r2, *immediate)
            }
            _ => continue,
        };
    
        if immediate <= max_movc_range {
            continue; // already legal
        }
    
        let loop_count = immediate / max_movc_range;
        let last_iter = immediate % max_movc_range;
    
        // Remove original instruction
        ir.remove(&movc_index);
    
        let mut cursor = movc_index;
    
        for i in 0..loop_count {
            let iter_imm = if i == loop_count - 1 && last_iter != 0 {
                last_iter
            } else {
                max_movc_range
            };
    
            ir.insert(
                cursor,
                TransporterISA::MOVC {
                    r0,
                    r1,
                    r2,
                    immediate: iter_imm,
                },
            );
    
            cursor += iter_imm;
        }
    }


    // 6. check/change/report fire time and latency
    let current_fire_time = transporter_table.fire_time;

    let mut new_ir = BTreeMap::new();

    for (time_index, inst) in ir.into_iter() {
        new_ir.insert(time_index + current_fire_time, inst);
    }
    
    ir = new_ir;

    let min_time_index = *ir.keys().next().unwrap();
    let max_time_index = *ir.keys().next_back().unwrap();

    if min_time_index < 0 {
        return Err("Failed to generate transport code with the current time constraints, needed optimisation".into());
    }

    // update transporter metadata
    transporter_table.fire_time = min_time_index;
    transporter_table.end_time = max_time_index;
    transporter_table.latency = (max_time_index - min_time_index) as u32;

    transporter_table.ir = ir;
    Ok(())
}


fn generate_code(
    transporter: &mut TransporterTable,
) -> Result<(), Box<dyn std::error::Error>> {
    
   let mut code: Vec<u16> = Vec::new();

   for (_, inst_isa) in &transporter.ir {
        let inst: u16 = match inst_isa {
            TransporterISA::NOP { immediate } => {
                (0b000 << 13) | 
                ((*immediate as u16) & 0x1FFF)
            }

            TransporterISA::LDI { r0, immediate } => {
                (0b010 << 13) | 
                ((*r0 as u16 & 0x7) << 10) |
                ((*immediate as u16) & 0x03FF)
            }

            TransporterISA::MOV { r0, r1, immediate } => {
                (0b101 << 13) | 
                ((*r0 as u16 & 0x7) << 10) |
                ((*r1 as u16 & 0x7) << 7) |
                ((*immediate as u16) & 0x007F)
            }

            TransporterISA::MOVC { r0, r1, r2, immediate } => {
                (0b100 << 13) | 
                ((*r0 as u16 & 0x7) << 10) |
                ((*r1 as u16 & 0x7) << 7) |
                ((*r2 as u16 & 0x7) << 4) |
                ((*immediate as u16) & 0x000F)
            }

            TransporterISA::OCCUPIED => {
                continue;
            }

            _ => return Err("Failed to generate transporter code".into())
        };

        code.push(inst);
   }

   transporter.size = code.len() as u32;
   transporter.binary = code;

   Ok(())
}

fn dump_transporter_ir(
    transporter: &TransporterTable,
    module_dir: &str,
    suffix: &str,
    print: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let formatted = utils::pretty_format(&transporter.ir)
        .map_err(|e| format!("pretty_format failed: {}", e))?;

    let path = format!(
        "{}/{}_{}.txt",
        module_dir,
        transporter.transporter_id,
        suffix
    );

    file_handler::write_file(&path, formatted.clone())?;

    if print {
        println!("{}", formatted);
    }

    Ok(())
}


#[allow(unused_variables)]
pub fn run(
    db: &mut DataBase, 
    dir: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Start: transporter code generation");
    let module_dir = format!("{}/transporter-code", dir);
    match std::fs::create_dir_all(&module_dir) {
        Ok(_) => (),
        Err(e) => {
            error!("Failed to create {} with {}", module_dir, e);
            return Err(Box::new(e));
        },
    };

    let transporters = &mut db.synthesized_information.transporter_tables; 

    for transporter in transporters.iter_mut() {
        pass0(transporter)?;
        dump_transporter_ir(
            transporter,
            &module_dir,
            "pass0",
            false, // print
        )?;

        pass1(transporter)?;
        dump_transporter_ir(
            transporter,
            &module_dir,
            "pass1",
            false, // print
        )?;

        pass2(transporter)?;
        dump_transporter_ir(
            transporter,
            &module_dir,
            "pass2",
            false, // print
        )?;

        pass3(transporter)?;
        dump_transporter_ir(
            transporter,
            &module_dir,
            "pass3",
            false, // print
        )?;

        pass_sanity(
            transporter,
            &module_dir,
        )?;
        dump_transporter_ir(
            transporter,
            &module_dir,
            "pass_sanity",
            false, // print
        )?;

        generate_code(transporter)?;

        // information report 
        info!("[{}] report - fire_time = {}, latency = {}, code size = {}", 
            transporter.transporter_id,
            transporter.fire_time,
            transporter.latency,
            transporter.size,
        );
    }

    info!("Finish: transporter code generation");
    Ok(())
}
