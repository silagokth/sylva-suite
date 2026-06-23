use sv_lib::model::{TransporterISA}; 
use std::collections::{HashMap, HashSet, BTreeMap};
use std::fmt;

#[derive(Debug)]
pub enum RegAllocError {
    SpilledRegister(u32),
    InvalidGraph,
    SpillingFail,
}


impl fmt::Display for RegAllocError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegAllocError::SpilledRegister(n) =>
                write!(f, "spilled register (needed {n})"),
            RegAllocError::InvalidGraph =>
                write!(f, "invalid graph"),
            RegAllocError::SpillingFail =>
                write!(f, "failed spilling"),
        }
    }
}


impl std::error::Error for RegAllocError {}


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


pub fn format_set(set: &HashSet<u32>) -> String {
    let mut elems: Vec<u32> = set.iter().copied().collect();
    elems.sort_unstable();

    let body = elems
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join(",");

    format!("[{}]", body)
}


pub fn format_liveness(
    vertices: &[i32],
    live_in: &[HashSet<u32>],
    live_out: &[HashSet<u32>],
) -> String {
    let mut out = String::new();
    out.push_str("== LIVENESS ==\n");
    out.push_str(format!("{:<8} {:<50} {:<50}\n", "time", "live_in", "live_out").as_str());

    for i in 0..vertices.len() {
        out.push_str(&format!(
            "{:<8} {:<50} {:<50}\n",
            vertices[i],
            format_set(&live_in[i]),
            format_set(&live_out[i]),
        ));
    }
    out
}


pub fn format_interference_graph(
    nodes: &[u32],
    edges: &HashSet<(u32, u32)>,
) -> String {
    let mut out = String::new();
    out.push_str("\n== INTERFERENCE GRAPH ==\n");
    out.push_str("Nodes:\n");
    out.push_str(&format!("{:?}\n", nodes));

    out.push_str("Edges:\n");
    for (a, b) in edges {
        out.push_str(&format!("  r{} -- r{}\n", a, b));
    }

    out
}


pub fn format_colouring(colouring: &Option<HashMap<u32, u32>>) -> String {
    let mut out = String::new();
    out.push_str("\n== GRAPH COLOURING ==\n");

    match colouring {
        Some(colour) => {
            for (vreg, preg) in colour {
                out.push_str(&format!("  r{} -> p{}\n", vreg, preg));
            }
        }, 
        None => {
            out.push_str("Failed to spill registers - number of physical registers isn't enough\n");
        }
    }

    out
}


pub fn fits_signed_n_bit(n: u32, val: i32) -> bool {
    val >= -(1 << (n - 1)) && val <= (1 << (n - 1)) - 1
}

pub fn fits_unsigned_n_bit(n: u32, val: i32) -> bool {
    val >= 0 && val <= (1 << n) - 1
}


pub fn find_free_times_before(
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

pub fn find_unused_register(
    ir: &BTreeMap<i32, TransporterISA>, 
) -> u32 {
    list_all_registers(ir)
        .iter()
        .map(|(reg, _)| *reg)
        .max()
        .map(|max_reg| max_reg + 1)
        .unwrap_or(u32::MAX)
}

pub fn list_all_mov_indices(
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


pub fn list_all_nop_indices(
    ir: &BTreeMap<i32, TransporterISA>,
) -> Vec<i32> {
    ir.iter()
        .filter(|(_, inst)| {
            matches!(
                inst,
                TransporterISA::NOP { .. }
            )
        })
        .map(|(time, _)| *time)
        .collect()
}


pub fn list_all_registers(
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


pub fn get_number_of_dependencies(
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


pub fn list_mov_indices_to_reg(
    ir: &BTreeMap<i32, TransporterISA>,
    mov_indices: &Vec<i32>,
    reg_num: u32, 
) -> Result<Vec<i32>, Box<dyn std::error::Error>> {
    let mut list = Vec::new();

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
            list.push(*index)
        }
    }

    Ok(list)
}



pub fn find_load_register_with_number(
    ir: &BTreeMap<i32, TransporterISA>,
    number: u32,
) -> Option<(i32, i32)> { // (time, value)
    // special r0
    if number == 0 {
        return Some((i32::MIN, 0));
    }

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


pub fn use_count(
    ir: &BTreeMap<i32, TransporterISA>,
    number: u32,
) -> Result<u32, RegAllocError> {

    let mov_indices = list_all_mov_indices(&ir);
    get_number_of_dependencies(ir, &mov_indices, number)
        .map_err(|_| RegAllocError::InvalidGraph) 
}


pub fn live_range(
    ir: &BTreeMap<i32, TransporterISA>,
    number: u32,
) -> Result<i32, RegAllocError> {
    let def_time = find_load_register_with_number(ir, number)
        .map(|(a, _)| a)
        .ok_or(RegAllocError::InvalidGraph)?;

    let mov_indices = list_all_mov_indices(ir);

    let selected_indices =
        list_mov_indices_to_reg(ir, &mov_indices, number)
            .map_err(|_| RegAllocError::InvalidGraph)?;

    let last_use_time = selected_indices
        .iter()
        .max()
        .ok_or(RegAllocError::InvalidGraph)?;

    Ok(last_use_time - def_time)
}


pub fn tighten_ir_timing(
    ir: &mut BTreeMap<i32, TransporterISA>,
) -> Result<(), Box<dyn std::error::Error>> {

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

    let mut reg_deps_list: Vec<(u32, u32)> = register_dependencies
        .iter()
        .map(|(&idx, &deps)| (idx, deps))
        .collect();

    reg_deps_list.sort_by(|a, b| b.1.cmp(&a.1));

    let sorted_reg_deps: Vec<u32> = 
        reg_deps_list.iter().map(|&(r, _)| r).collect(); 

    // map: reg -> MOV/MOVC dependency times
    let mut mov_deps_times: HashMap<u32, Vec<i32>> = HashMap::new();
    
    for &reg_num in &sorted_reg_deps {
        let times = list_mov_indices_to_reg(&ir, &mov_indices, reg_num)?;
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

            let current_time = match find_load_register_with_number(&ir, reg_num) {
                Some((time, _)) => time,
                None => return Err(format!("Expect LDI for r{}", reg_num).into()),
            };

            let new_slots = find_free_times_before(&ir, min_time, 1); 
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
    
    Ok(())
}


