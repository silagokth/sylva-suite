#[allow(unused_imports)]
use sv_lib::model::{TechConstraint, TimingRow};
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
        tech_const.required_period > sum_delay - improved_delay,
        tech_const.required_slew < slowdown_slew,
    ];

    Ok(requirements.into_iter().all(|r| r))
}



pub fn insert_buffer(
    tech_const: &TechConstraint,
    wire_length: i32,
    slew: f64,
) ->  Result<String, Box<dyn std::error::Error>> {
    let mut design: String = "w".repeat(wire_length as usize);
    let mut number_of_buffers: i32 = 0;

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



pub fn insert_main(
    tech_const: &TechConstraint,
    wire_length: i32,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut design = insert_buffer(tech_const, wire_length, tech_const.initial_slew)?;
    let mut number_of_registers: i32 = 0;

    // if initial design is found, skip this.
    // otherwise, add more registers
    while design.is_empty() {
        if number_of_registers >= wire_length / 3 || wire_length <= 4 {
            return Ok(String::new()); // No valid solution
        }

        number_of_registers += 1;
        let mut current_index = 0;
        let mut current_slew = tech_const.initial_slew;
        
        // replace a register might change some characteristics
        // TODO: slew rate after a register might be a function of the input slew
        // TODO: slew rate could be deteriorated after passing through wire
        //       In that case, feed slew info from design1 to design2 
        for i in 0..(number_of_registers + 1) {
            let end_index = if i == number_of_registers {
                wire_length
            } else {
                ((i + 1) * wire_length) / (number_of_registers + 1)
            };
        
            let segment_length = end_index - current_index;
            let segment_design = insert_buffer(tech_const, segment_length, current_slew)?;

            if segment_design.is_empty() {
                design = String::new();
                break; // try to add more registers
            }

            current_index = end_index + 1;
            current_slew = tech_const.register_slew_constant;
            
            design.push_str(&segment_design);
            if i != number_of_registers {
                design.push_str("r");
            }
        }
    }

    Ok(design)
}




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_path_satisfied() {
        let mut constraints = TechConstraint {
            required_period: 100.0,
            required_slew: 0.5,
            initial_slew: 1.0,
            buffer_slew_declined_factor: 0.05,
            buffer_delay_improved_factor: 50.0,   
            register_slew_constant: 1.0,
            slew_rates: vec![],
            timing_table: vec![],
        };
        for i in 0..10 {
            constraints.slew_rates.push(0.40 + 0.10 * i as f64);
        }
        for _i in 0..10 {
            let mut rows: Vec<f64> = vec![];
            for j in 0..10 {
                rows.push(40.0 + 10.0 * j as f64);
            }
            constraints.timing_table.push(TimingRow {rows});
        }

        let mut path: String = "wwwww".to_string();
        match is_path_satisfied(&constraints, &path, 1.0) {
            Ok(x) => assert_eq!(x, true),
            Err(x) => panic!("is_path_satisfied fails 1 with {}", x),
        }

        match is_path_satisfied(&constraints, &path, 3.0) {
            Ok(x) => assert_eq!(x, false),
            Err(x) => panic!("is_path_satisfied fails 2 with {}", x),
        }
        
        match is_path_satisfied(&constraints, &path, 0.1) {
            Ok(x) => assert_eq!(x, false),
            Err(x) => panic!("is_path_satisfied fails 3 with {}", x),
        }

        path = "".to_string();
        match is_path_satisfied(&constraints, &path, 1.0) {
            Ok(x) => assert_eq!(x, false),
            Err(x) => panic!("is_path_satisfied fails 4 with {}", x),
        }
        
        path = "wwwwwwwwwww".to_string();
        match is_path_satisfied(&constraints, &path, 1.0) {
            Ok(x) => assert_eq!(x, false),
            Err(x) => panic!("is_path_satisfied fails 5 with {}", x),
        }
        
        path = "wwwwwwwww".to_string();
        match is_path_satisfied(&constraints, &path, 1.0) {
            Ok(x) => assert_eq!(x, false),
            Err(x) => panic!("is_path_satisfied fails 6 with {}", x),
        }
        
        path = "wwwwbwwww".to_string();
        match is_path_satisfied(&constraints, &path, 1.0) {
            Ok(x) => assert_eq!(x, true),
            Err(x) => panic!("is_path_satisfied fails 7 with {}", x),
        }
        
        path = "wwwwbwwwww".to_string();
        match is_path_satisfied(&constraints, &path, 1.0) {
            Ok(x) => assert_eq!(x, false),
            Err(x) => panic!("is_path_satisfied fails 8 with {}", x),
        }
        
        path = "wwbwwwbwww".to_string();
        match is_path_satisfied(&constraints, &path, 1.0) {
            Ok(x) => assert_eq!(x, true),
            Err(x) => panic!("is_path_satisfied fails 9 with {}", x),
        }

        path = "wwwww".to_string();
        match is_path_satisfied(&constraints, &path, 0.5) {
            Ok(x) => assert_eq!(x, false),
            Err(x) => panic!("is_path_satisfied fails 1 with {}", x),
        }
    }


    #[test]
    fn test_insert_buffer() {
        let mut constraints = TechConstraint {
            required_period: 100.0,
            required_slew: 0.5,
            initial_slew: 1.0,
            buffer_slew_declined_factor: 0.05,
            buffer_delay_improved_factor: 50.0,   
            register_slew_constant: 1.0,
            slew_rates: vec![],
            timing_table: vec![],
        };
        for i in 0..10 {
            constraints.slew_rates.push(0.40 + 0.10 * i as f64);
        }
        for _i in 0..10 {
            let mut rows: Vec<f64> = vec![];
            for j in 0..10 {
                rows.push(40.0 + 10.0 * j as f64);
            }
            constraints.timing_table.push(TimingRow {rows});
        }

        let expected: String = "wwwbwwbwww".to_string();
        let result = match insert_buffer(&constraints, 10, 1.0) {
            Ok(x) => x,
            Err(e) => panic!("insert_buffer fails 1 with {}", e),
        };
        assert_eq!(result, expected);
    }   

    #[test]
    fn test_insert_main() {
        let mut constraints = TechConstraint {
            required_period: 100.0,
            required_slew: 0.5,
            initial_slew: 1.0,
            buffer_slew_declined_factor: 0.05,
            buffer_delay_improved_factor: 50.0,   
            register_slew_constant: 1.0,
            slew_rates: vec![],
            timing_table: vec![],
        };
        for i in 0..10 {
            constraints.slew_rates.push(0.40 + 0.10 * i as f64);
        }
        for _i in 0..10 {
            let mut rows: Vec<f64> = vec![];
            for j in 0..10 {
                rows.push(40.0 + 10.0 * j as f64);
            }
            constraints.timing_table.push(TimingRow {rows});
        }

        let expected: String = "wwwbwwbwwwrwwwwbwwww".to_string();
        let result = match insert_main(&constraints, 20) {
            Ok(x) => x,
            Err(e) => panic!("insert_buffer fails 1 with {}", e),
        };
        assert_eq!(result, expected);
    }   


}
