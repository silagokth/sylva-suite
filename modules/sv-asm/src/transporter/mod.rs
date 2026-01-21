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

        return Err("Test stop!!!".into())
    }


    info!("Finish: transporter code generation");
    Ok(())
}
