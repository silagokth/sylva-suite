use crate::model::{TechConstrainti, Rows};
use std::collections::{HashMap, HashSet};

pub fn path_satisfied(
    tech_const: TechConstraint,
    path: Vec<String>,
    slew: f64,
) -> Result<bool, Box<dyn std::error::Error>> {

    // maximum number of blocks 
    if path.len() > tech_const.timing_table.len() {
        return Ok(false)
    }

    // segments[start_index] = end_index, excluding buffers
    let mut segments: HashMap<i32, i32> = HashMap::new();
    let mut current = 0;
    
    for i in 0..path.len() {
        if path[i] == "b" {
            if i == 0 || i == path.len() - 1 {
                return Err(format!("NoC start and end blocks should be wire").into());
            }
            segments.insert(current, i - 1);
            current = i + 1;
        }

        if i == path.len() - 1 {
            segments.insert(current, i);
        }
    }

    // calculate delays in each segment
    let mut segment_delay: Vec<f64> = Vec::new();
    let mut slowdown_slew = slew;
    
    for (start_index, end_index) in segments.iter() {
        // A col index is K intervening number of wire blocks
        let col = end_index - start_index;
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
    let mut improved_delay: f64 = path.iter()
        .filter(|&&ref p| p == "b")
        .count() as 64 * tech_const.buffer_delay_improved_factor;

    // constraints
    let requirements = [
        tech_const.required_period > sum_delay - improved_delay,
        tech_const.required_slew < slowdown_slew,
    ];

    Ok(requirements.into_iter().all(|r| r))
}





















#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_satisfied() {
        let mut constraints = TechConstraint {
            required_period: 100.0,
            required_slew: 0.5,
            initial_slew: 1.0,
            buffer_slew_declined_factor: 0.05,
            buffer_delay_improved_factor: 10,   
            register_slew_constant: 1.0,
            slew_rates: vec![],
            timing_table: vec![],
        }
        for i in 0..10 {
            constaints.slew_rates.push(0.40 + 0.05 * i);
        }
        for i in 0..10 {
            let mut rows: Rows = vec![];
            for j in 0..10 {
                constaints.slew_rates.push(10 + 5 * i);
            }
        }

    }
}
