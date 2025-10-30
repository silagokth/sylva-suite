use sv_lib::model::{TechConstraint};
use std::collections::{HashMap, HashSet};

pub fn is_path_satisfied(
    tech_const: &TechConstraint,
    path: &String,
    slew: f64,
) -> Result<bool, Box<dyn std::error::Error>> {

    // acceptable range of blocks 
    if path.chars().count() == 0 || 
        path.chars().count() > tech_const.timing_table.len() {
        return Ok(false)
    }

    // segments[start_index] = end_index, excluding buffers
    let mut segments: HashMap<i32, i32> = HashMap::new();
    let mut current = 0;
    
    for (i, c) in path.chars().enumerate() {
        if c == 'b' {
            if i == 0 || i == path.chars().count() - 1 {
                return Err(format!("NoC start and end blocks should be wire").into());
            }
            segments.insert(current, (i - 1) as i32);
            current = (i + 1) as i32;
        }

        if i == path.chars().count() - 1 {
            segments.insert(current, i as i32);
        }
    }

    // calculate delays in each segment
    let mut segment_delay: Vec<f64> = Vec::new();
    let mut slowdown_slew = slew;
    
    for (start_index, end_index) in segments.iter() {
        // A col index is K intervening number of wire blocks
        let col = (end_index - start_index) as usize;
        // A row index is selected based on the input slew (pessimistic method)
        // find max slew rate greater than slowdown_slew and get index
        // TODO: improvement by linear interpolation
        let row = match tech_const.slew_rates.iter().enumerate()
                        .filter(|&(_i, &x)| x > slowdown_slew)
                        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap()) {
            Some((i, _)) => i,
            None => return Ok(false),
        };
    
        if *start_index > 0 {
            slowdown_slew -= tech_const.buffer_slew_declined_factor;
        } 
       
        // lookup the delay time in the table
        segment_delay.push(tech_const.timing_table[row].rows[col]);
    }
    let sum_delay: f64 = segment_delay.iter().sum();

    // delay improvement from buffers
    let improved_delay: f64 = path.matches('b')
        .count() as f64 * tech_const.buffer_delay_improved_factor;

    // constraints
    let requirements = [
        // change clock frequency to required period 
        (1.0 / tech_const.clock_frequency) > sum_delay - improved_delay,
        tech_const.required_slew < slowdown_slew,
    ];

    Ok(requirements.into_iter().all(|r| r))
}



pub fn insert_buffer(
    tech_const: &TechConstraint,
    wire_length: u32,
    slew: f64,
) ->  Result<String, Box<dyn std::error::Error>> {
    let mut design: String = "w".repeat(wire_length as usize);
    let mut number_of_buffers: u32 = 0;

    // check the design and add buffers
    // TODO: Need to consider about timing tables 
    //       for different types (R-R, B-B, R-B, B-R)
    while !is_path_satisfied(tech_const, &design, slew)? {
        // add buffers until wbwbwbwb..wbw
        if number_of_buffers < wire_length / 2 && wire_length > 2 {
            number_of_buffers += 1;
            let mut buffer_indices = HashSet::new();
            for i in 0..number_of_buffers {
                buffer_indices.insert(((i + 1) * wire_length) / (number_of_buffers + 1));
            }

            design = (0..wire_length)
                .map(|i| if buffer_indices.contains(&i) { 'b' } else { 'w' })
                .collect();
        } else {
            return Ok(String::new()); // fail to find a valid design
        }
    }
    
    Ok(design)
}



fn add_register(
    wire_length: u32,
    number_of_registers: u32,
) -> Result<Vec<u32>, Box<dyn std::error::Error>> {
    
    if number_of_registers == 0 {
        return Ok(vec![]);
    }

    if wire_length / (number_of_registers + 1) < 3 {
        return Err(format!("NoC wire length is too short to add registers").into());
    }

    // return indices to add registers
    let step = wire_length as f64 / (number_of_registers + 1) as f64;

    // use ceil because we assume that the first drive stregth is stronger than others
    let indices: Vec<u32> = (1..=number_of_registers)
        .map(|i| (i as f64 * step).ceil() as u32)
        .collect();

    Ok(indices)
}



pub fn main(
    tech_const: &TechConstraint,
    wire_length: u32,
    fixed_delay: u32,
) -> Result<String, Box<dyn std::error::Error>> {

    let register_indices: Vec<u32> = add_register(wire_length, fixed_delay)?;
    
    let mut segments: Vec<String> = Vec::new();
    let mut path = String::new();

    for i in 0..wire_length {
        if register_indices.contains(&i) {
            segments.push(path.clone());
            path.clear();
        } else {
            path.push('w');
        }
    }
    if !path.is_empty() {
        segments.push(path);
    }

    // insert buffers into each segment 
    for (i, each) in segments.iter_mut().enumerate() {
        let slew = if i == 0 {
            tech_const.initial_slew
        } else {
            tech_const.register_slew_constant
        };

        *each = insert_buffer(tech_const, each.len() as u32, slew)?;
    }

    // combine all segments + registers into one string
    let mut result = String::new();
    for (i, segment) in segments.iter().enumerate() {
        result.push_str(segment);
        if i < register_indices.len() {
            result.push('r'); // insert register
        }
    }
    
    Ok(result)
}

