use sv_lib::model::{DataBase, TransporterTable, TransporterISA}; 
use sv_lib::file_handler;
use log::{info, error};
use std::collections::{HashMap, BTreeMap};


pub fn pretty_format(
    insts: &BTreeMap<i32, TransporterISA>,
) -> Result<String, String> {
    let mut out = String::new();

    out.push_str(&format!("{:<12}{}\n", "time", "code"));

    for (time, inst) in insts.iter() {
        out.push_str(&format!(
            "{:<12}{}\n",
            time,
            inst
        ));
    }

    Ok(out)
}



fn find_free_times_before(
    ir: &BTreeMap<i32, TransporterISA>,
    before: i32,
    count: usize,
) -> Vec<i32> {
    let mut result = Vec::with_capacity(count);
    let mut t = before - 1;

    while result.len() < count {
        if !ir.contains_key(&t) {
            result.push(t);
        }
        t -= 1;
    }

    result
}


fn list_all_mov_indices(
    ir: &BTreeMap<i32, TransporterISA>,
) -> Vec<i32> {
    ir.iter()
        .filter(|(_, inst)| {
            matches!(
                inst,
                TransporterISA::MOV { .. } | TransporterISA::MOVC { .. }
            )
        })
        .map(|(time, _)| *time)
        .collect()
}


fn list_all_registers(
    ir: &BTreeMap<i32, TransporterISA>,
) -> Vec<(u32, i32)> { // (reg_num, reg_value)
    ir.iter()
        .filter_map(|(_, inst)| {
            if let TransporterISA::LDI { r0, immediate } = inst {
                Some((*r0, *immediate))
            } else {
                None
            }
        })
        .collect()
}


fn get_number_of_dependencies(
    ir: &BTreeMap<i32, TransporterISA>,
    mov_indices: &Vec<i32>,
    reg_num: u32, 
) -> Result<u32, Box<dyn std::error::Error>> {
    let mut dependency_count = 0;

    for index in mov_indices {
        let (reg0, reg1, reg2): (u32, u32, Option<u32>) = match ir.get(&index) {
            Some(TransporterISA::MOV { r0, r1, .. }) => (*r0, *r1, None),
            Some(TransporterISA::MOVC { r0, r1, r2, ..}) => (*r0, *r1, Some(*r2)),
            _ => return Err(format!("Expect MOV(C), found at {}", index).into()),
        };
        
        if reg0 == reg_num ||
            reg1 == reg_num ||
            reg2 == Some(reg_num) 
        {
            dependency_count += 1;
        }
    }

    Ok(dependency_count)
}




fn find_load_register_with_number(
    ir: &BTreeMap<i32, TransporterISA>,
    number: u32,
) -> Option<(i32, i32)> { // (time, value)
    ir.iter()
        .find_map(|(time, inst)| {
            if let TransporterISA::LDI { r0, immediate } = inst {
                if *r0 == number {
                    return Some((*time, *immediate));
                }
            }
            None
        })
}


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
        let reg_source = (register_allocs.len() - 1) as u32;
        
        register_allocs.push(target);
        let reg_target = (register_allocs.len() - 1) as u32;

        if ir.contains_key(&time_index) {
            return Err(format!("Time slot {} already occupied", time_index).into());
        }

        // MOV at time_index
        ir.insert(
            time_index,
            TransporterISA::MOV {
                r1: reg_source,
                r0: reg_target,
                immediate: 0,
            },
        );

        let free_times = find_free_times_before(&ir, 0, 2);
        let t1 = free_times[0];
        let t2 = free_times[1];
        
        if ir.contains_key(&t1) || ir.contains_key(&t2) {
            return Err(format!("find_free_times_before failed to find free time slots").into());
        }

        ir.insert(
            t1,
            TransporterISA::LDI {
                r0: reg_target,
                immediate: target,
            },
        );

        ir.insert(
            t2,
            TransporterISA::LDI {
                r0: reg_source,
                immediate: source,
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
    
    let mov_indices: Vec<i32> = list_all_mov_indices(&ir);
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
            find_load_register_with_number(&ir, reg0_number)
                .ok_or_else(|| format!("No LDI found for r{}", reg0_number))?;

        let (reg1_time, reg1_value) =
            find_load_register_with_number(&ir, reg1_number)
                .ok_or_else(|| format!("No LDI found for r{}", reg1_number))?;

        // same immediate optimisation 
        if reg0_value == reg1_value {
            new_reg1_number = reg0_number;
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
    
    let mov_indices: Vec<i32> = list_all_mov_indices(&ir);
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
            find_load_register_with_number(&ir, reg0)
                .ok_or_else(|| format!("No LDI for r{}", reg0))?;

        let (_, reg1_value) =
            find_load_register_with_number(&ir, reg1)
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
                find_load_register_with_number(&ir, r0n)
                    .ok_or_else(|| format!("No LDI for r{}", r0n))?;

            let (_, r1v) =
                find_load_register_with_number(&ir, r1n)
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
            for (r, v) in list_all_registers(&ir) {
                if v == c {
                    reg2 = Some(r);
                    break;
                }
            }

            let reg2 = if let Some(r) = reg2 {
                r
            } else {
                let new_reg = list_all_registers(&ir)
                    .iter()
                    .map(|(r, _)| *r)
                    .max()
                    .unwrap_or(0) + 1;

                let time_index = find_free_times_before(&ir, 0, 1);     
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
                    immediate: iter_count as i32,
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
    
    let mov_indices = list_all_mov_indices(&ir);
    let reg_nums: Vec<u32> = list_all_registers(&ir)
        .into_iter()
        .map(|(r, _)| r)
        .collect();

    for &reg_num in &reg_nums {
        let deps = get_number_of_dependencies(
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
            find_load_register_with_number(&ir, reg0)
                .ok_or_else(|| format!("No LDI for r{}", reg0))?;

        let (_, reg1_value) =
            find_load_register_with_number(&ir, reg1)
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
                find_load_register_with_number(&ir, reg2)
                    .ok_or_else(|| format!("No LDI for r{}", reg2))?;

            let (_, reg3_value) =
                find_load_register_with_number(&ir, reg3)
                    .ok_or_else(|| format!("No LDI for r{}", reg3))?;
            
            if reg3_value - reg2_value == reg1_value - reg0_value {
                
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

    transporter_table.ir = ir;
    Ok(())
}


// 1. remove unused registers
// 2. tighten the timing 
// 3. actual register allocation
// 4. check value bound 
// 5. check/change/report fire time and latency
fn pass_sanity(
   transporter_table: &mut TransporterTable,
) -> Result<(), Box<dyn std::error::Error>> {
 
    let mut ir = transporter_table.ir.clone(); 
 
    transporter_table.ir = ir;
    Ok(())
}



fn dump_transporter_ir(
    transporter: &TransporterTable,
    module_dir: &str,
    suffix: &str,
    print: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let formatted = pretty_format(&transporter.ir)
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

        pass_sanity(transporter)?;
        dump_transporter_ir(
            transporter,
            &module_dir,
            "pass_sanity",
            false, // print
        )?;

        return Err("Test stop!!!".into())
    }


    info!("Finish: transporter code generation");
    Ok(())
}
