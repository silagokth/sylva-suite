use sv_lib::solver::{Solver};
use log::{debug};
use std::collections::{HashMap};
use ndarray::{Array2, Axis, concatenate, s};
use std::fmt::Display;


#[derive(Debug)]
pub struct MemoryConstraint {
    pub edge_id: String,
    pub src_fire_time: i32,
    pub dst_fire_time: i32,
    pub routing_delay: i32,
    pub output_buffer_size: i32,
    pub input_buffer_size: i32,
    pub channel_width_size: i32,
    pub input_memory_type: String,
    pub output_memory_type: String,
    pub fixed: bool,
}

#[derive(Debug)]
pub struct MemoryBankInfo {
    pub cost: i32,
    pub ob_type: String,
    pub ib_type: String,
    pub ob_size: i32,
    pub ib_size: i32,
    pub comm_cap: i32,
    // [channel, token]
    pub t0: Array2<i32>,
    pub t1: Array2<i32>,
    pub t2: Array2<i32>,
    pub t3: Array2<i32>,
}


fn format_matrix<T: Display>(matrix: &Array2<T>) -> String {
    let mut s = String::from("[|");

    for (i, row) in matrix.outer_iter().enumerate() {
        if i != 0 {
            s.push('\n');
            s.push_str("  |"); // indent new rows with a | (optional: 2 spaces)
        }

        let row_str = row.iter()
                         .map(|v| v.to_string())
                         .collect::<Vec<_>>()
                         .join(",");
        s.push_str(&row_str);
    }

    s.push_str("|]");
    s
}


#[allow(unused_assignments)]
pub fn optimise_memory(
    constraints: &MemoryConstraint,
    id: &str, 
    output_patterns: &Vec<(i32, i32, i32)>,
    input_patterns: &Vec<(i32, i32, i32)>,
    module_dir: String,
) -> Result<MemoryBankInfo, Box<dyn std::error::Error>> {

    let number_of_tokens = output_patterns.len();
       
    // creating the producing matrix 
    let mut working_output_channels: Vec<i32> = output_patterns.iter().map(|&(_, c, _)| c).collect();
    working_output_channels.sort();
    working_output_channels.dedup();

    let mapping_output_channels: HashMap<i32, i32> = working_output_channels
        .iter()
        .enumerate()
        .map(|(i, &ch)| (ch, i as i32))
        .collect();

    let mut producing_matrix = 
        Array2::<i32>::from_elem((mapping_output_channels.len(), number_of_tokens), -1);

    for (col, &(_, channel, time)) in output_patterns.iter().enumerate() {   
        let row = mapping_output_channels[&channel];
        producing_matrix[(row as usize, col)] = time + constraints.src_fire_time;
    }

    // creating the consuming matrix 
    let mut working_input_channels: Vec<i32> = input_patterns.iter().map(|&(_, c, _)| c).collect();
    working_input_channels.sort();
    working_input_channels.dedup();

    let mapping_input_channels: HashMap<i32, i32> = working_input_channels
        .iter()
        .enumerate()
        .map(|(i, &ch)| (ch, i as i32))
        .collect();

    let mut consuming_matrix = 
        Array2::<i32>::from_elem((mapping_input_channels.len(), number_of_tokens), -1);

    for (col, &(_, channel, time)) in input_patterns.iter().enumerate() {   
        let row = mapping_input_channels[&channel];
        consuming_matrix[(row as usize, col)] = time + constraints.dst_fire_time;
    }

    // maximum delay time in the scope 
    let maximum_delay = std::cmp::max(
        producing_matrix.iter().max().unwrap(), 
        consuming_matrix.iter().max().unwrap()
    ) + 1;
    
    // memory costs
    let mut memory_costs: Vec<i32> = Vec::new();
    
    match constraints.output_memory_type.as_str() {
        "fifo" => memory_costs.push(1),
        "rf" => memory_costs.push(5),
        "ram" => memory_costs.push(3),
        &_ => return Err(format!("cannot parse the constraint for memory synthesis").into()),
    };

    match constraints.input_memory_type.as_str() {
        "fifo" => memory_costs.push(1),
        "rf" => memory_costs.push(5),
        "ram" => memory_costs.push(3),
        &_ => return Err(format!("cannot parse the constraint for memory synthesis").into()),
    };
    
    // communication cost
    let communication_cost = 30;

    // number of ports/channels
    let number_of_producers = mapping_output_channels.len() as i32;
    let number_of_consumers = mapping_input_channels.len() as i32;

    let number_of_communications = *vec![
        number_of_producers,
        number_of_consumers,
        constraints.channel_width_size,
    ].iter().min().unwrap();

    fn next_power_of_two(x: i32) -> i32 {
        if x <= 1 {
            return 2;
        }
        ((x as u32).next_power_of_two()) as i32
    }

    // ----------------------------------------------------------------------
    // Solving 
    
    // breaking into many small problem of size "window_size"
    let mut current_position: usize = 0; 
    let mut window_size: usize = 200; 
    let number_of_iterations: usize = ((number_of_tokens - 1) / window_size) + 1;

    // define an empty struct to be returned
    let mut ret = MemoryBankInfo {
        cost: -1,
        ob_type: constraints.output_memory_type.clone(),
        ib_type: constraints.input_memory_type.clone(),
        ob_size: 2,
        ib_size: 2,
        comm_cap: 1,
        t0: producing_matrix.clone(),
        t1: Array2::zeros((0, 0)),
        t2: Array2::zeros((0, 0)),
        t3: consuming_matrix.clone(),
    };

    let mut prev_d01: Vec<i32> = Vec::new();
    let mut prev_d01_start: Vec<i32> = Vec::new();
    let mut prev_d23: Vec<i32> = Vec::new();
    let mut prev_d23_start: Vec<i32> = Vec::new();
    let mut prev_d01_end_last = 0;
    let mut prev_d23_end_last = 0;

    for i in 0..number_of_iterations {
        current_position = window_size * i;
        // last iteration
        if i == number_of_iterations - 1 {
            window_size = number_of_tokens - current_position;
        }

        // sorted column indices  
        let output_slice: &_ = &output_patterns[current_position..current_position + window_size];
        let mut sorted_producing_indices: Vec<_> = (0..output_slice.len()).collect();
        sorted_producing_indices.sort_by_key(|&i| output_slice[i].2);
        sorted_producing_indices = sorted_producing_indices.into_iter().map(|i| i + 1).collect();

        let input_slice: &_ = &input_patterns[current_position..current_position + window_size];
        let mut sorted_consuming_indices: Vec<_> = (0..input_slice.len()).collect();
        sorted_consuming_indices.sort_by_key(|&i| input_slice[i].2);
        sorted_consuming_indices = sorted_consuming_indices.into_iter().map(|i| i + 1).collect();

        let mut statements = String::new();

        statements += &format!(r#"include "cumulative.mzn";
% problem to select memory banks and map channels to minimize cost 

% Parameters 
int: WINDOW_SIZE = {WINDOW_SIZE};
int: PREVIOUS_SIZE = {PREVIOUS_SIZE};
int: M = {PRODUCING_CHANNELS}; % number of proceducer channels to OB
int: N = {CONSUMING_CHANNELS}; % number of consumer channels from IB
int: MAX_K = {MAXIMUM_CHANNEL_WIDTH}; % communication channels between OB and IB
int: MAX_OB_SIZE = {MAXIMUM_OUTPUT_BUFFER_SIZE};
int: MAX_IB_SIZE = {MAXIMUM_INPUT_BUFFER_SIZE};
int: MAX_DELAY = {MAXIMUM_DELAY};
int: comm_delay = {WIRE_DELAY};

% addresss patterns 
array [1..M,1..WINDOW_SIZE] of int: T0 = {PRODUCING_MATRIX}; % including fire time 
array [1..N,1..WINDOW_SIZE] of int: T3 = {CONSUMING_MATRIX}; % including fire time 
array [1..WINDOW_SIZE] of int: SORTED_T0 = {SORTED_OUTPUT_PATTERN_INDICES:?};
array [1..WINDOW_SIZE] of int: SORTED_T3 = {SORTED_INPUT_PATTERN_INDICES:?};

array [1..MAX_K,1..PREVIOUS_SIZE] of -1..MAX_DELAY: PREV_T1 = {PREVIOUS_T1};
array [1..PREVIOUS_SIZE] of -1..MAX_DELAY: PREV_D01_START = {PREVIOUS_D01_START:?};
array [1..PREVIOUS_SIZE] of -1..MAX_DELAY: PREV_D01 = {PREVIOUS_D01:?};
array [1..PREVIOUS_SIZE] of -1..MAX_DELAY: PREV_D23_START = {PREVIOUS_D23_START:?};
array [1..PREVIOUS_SIZE] of -1..MAX_DELAY: PREV_D23 = {PREVIOUS_D23:?};
int: PREV_D01_START_LAST = {PREVIOUS_D01_START_LAST};
int: PREV_D01_END_LAST = {PREVIOUS_D01_END_LAST};
int: PREV_D23_START_LAST = {PREVIOUS_D23_START_LAST};
int: PREV_D23_END_LAST = {PREVIOUS_D23_END_LAST};

% memory characteristics 1:output, 2:input 
array [1..2] of int: port_capacity = {MEMORY_PORT_CAPACITY:?};
array [1..2] of int: memory_cost = {MEMORY_COST:?};
int: comm_width_cost = {COMMUNICATION_COST};

%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
% Decision variables and constraints

%%%%%%%%%%%%%%%%%%%
% memory detailed variables
{COMM_CAP_VARIABLE}
{OB_CAP_VARIABLE}
{IB_CAP_VARIABLE}
var 1..16: ob_cap_power;
var 1..16: ib_cap_power;
"#,
            WINDOW_SIZE = window_size,
            PREVIOUS_SIZE = ret.t1.ncols(),
            PRODUCING_CHANNELS = number_of_producers,
            CONSUMING_CHANNELS = number_of_consumers,
            MAXIMUM_CHANNEL_WIDTH = number_of_communications,
            MAXIMUM_OUTPUT_BUFFER_SIZE = next_power_of_two(constraints.output_buffer_size),
            MAXIMUM_INPUT_BUFFER_SIZE = next_power_of_two(constraints.input_buffer_size),
            MAXIMUM_DELAY = maximum_delay,
            WIRE_DELAY = constraints.routing_delay,
            PRODUCING_MATRIX = format_matrix(&producing_matrix.slice(s![.., current_position..current_position+window_size]).to_owned()),
            CONSUMING_MATRIX = format_matrix(&consuming_matrix.slice(s![.., current_position..current_position+window_size]).to_owned()),
            SORTED_OUTPUT_PATTERN_INDICES = sorted_producing_indices,
            SORTED_INPUT_PATTERN_INDICES = sorted_consuming_indices,
            PREVIOUS_T1 = format_matrix(&ret.t1),
            PREVIOUS_D01_START = prev_d01_start,
            PREVIOUS_D01 = prev_d01,
            PREVIOUS_D23_START = prev_d23_start,
            PREVIOUS_D23 = prev_d23,
            PREVIOUS_D01_START_LAST = prev_d01_start.iter().max().unwrap_or(&0),
            PREVIOUS_D01_END_LAST = prev_d01_end_last,
            PREVIOUS_D23_START_LAST = prev_d23_start.iter().max().unwrap_or(&0),
            PREVIOUS_D23_END_LAST = prev_d23_end_last,
            MEMORY_PORT_CAPACITY = vec![number_of_producers, number_of_consumers],
            MEMORY_COST = memory_costs,
            COMMUNICATION_COST = communication_cost,
            COMM_CAP_VARIABLE = if !constraints.fixed { format!("var {}..MAX_K: comm_cap;", ret.comm_cap) } else { "int: comm_cap = MAX_K;".into() },
            OB_CAP_VARIABLE = if !constraints.fixed { format!("var {}..MAX_OB_SIZE: ob_cap;", ret.ob_size) } else { "int: ob_cap = MAX_OB_SIZE;".into() },
            IB_CAP_VARIABLE = if !constraints.fixed { format!("var {}..MAX_IB_SIZE: ib_cap;", ret.ib_size) } else { "int: ib_cap = MAX_IB_SIZE;".into() },
        );

        statements += &format!(r#"
% force the buffer to be 2 ** N size
constraint (ob_cap = 2 ^ ob_cap_power);
constraint (ib_cap = 2 ^ ib_cap_power);

int: ob_in_ports = M;
var 1..MAX_K: ob_out_ports; 
var 1..MAX_K: ib_in_ports; 
int: ib_out_ports = N;

array [1..MAX_K,1..WINDOW_SIZE] of var -1..MAX_DELAY: T1;
array [1..MAX_K,1..WINDOW_SIZE] of var -1..MAX_DELAY: T2;
array [1..MAX_K] of var bool: comm_used;

array [1..WINDOW_SIZE] of var 0..MAX_DELAY: D01_START;
array [1..WINDOW_SIZE] of var 0..MAX_DELAY: D01_END;
array [1..WINDOW_SIZE] of var 0..MAX_DELAY: D01;
array [1..WINDOW_SIZE] of var 0..MAX_DELAY: D23_START;
array [1..WINDOW_SIZE] of var 0..MAX_DELAY: D23_END;
array [1..WINDOW_SIZE] of var 0..MAX_DELAY: D23;

%%%%%%%%%%%%%%%%%%%
% Constraints

% Each column has one non -1 value
constraint forall(j in 1..WINDOW_SIZE)(
    sum(i in 1..MAX_K)(bool2int(T1[i,j] != -1)) = 1
);
    
% Each row has unique non -1 value
constraint forall(i in 1..MAX_K, j1, j2 in 1..WINDOW_SIZE where j1 < j2)(
    (T1[i,j1] != -1 /\ T1[i,j2] != -1) -> T1[i,j1] != T1[i,j2]
);
 
% preserve the constraint where each channel can be used to send only one token at a time
constraint if PREVIOUS_SIZE != 0 then 
    forall(i in 1..MAX_K, j1 in 1..WINDOW_SIZE, j2 in 1..PREVIOUS_SIZE)(
        (T1[i,j1] != -1 /\ PREV_T1[i,j2] != -1) -> T1[i,j1] != PREV_T1[i,j2]
    )
endif;

% bind the variable to minimize the channel size 
constraint forall(i in 1..MAX_K)(
    comm_used[i] <-> (
        exists(j in 1..WINDOW_SIZE)(T1[i,j] != -1) \/
        exists(j in 1..PREVIOUS_SIZE)(PREV_T1[i,j] != -1) 
    )
);

% geometry constraints - start placing transporters from position 0..K
constraint forall(i in 2..MAX_K)(
    comm_used[i-1] >= comm_used[i]
);

% get communication capacity
constraint comm_cap = sum(i in 1..MAX_K)(bool2int(comm_used[i]));

% map to input/output ports
constraint (
    ob_out_ports = comm_cap /\
    ib_in_ports = comm_cap
);

% bind T2 to T1
constraint forall(i in 1..MAX_K, j in 1..WINDOW_SIZE)(
    if T1[i,j] != -1 then
        T2[i,j] = T1[i,j] + comm_delay
    else
        T2[i,j] = -1
    endif
); 

% time interval in the buffers
% constraint that T1 > T0 and T3 > T2
% bind the banking to all T

constraint forall(j in 1..WINDOW_SIZE)(
    let {{
        % i1 is a valid input channel and i2 is an output channel
        var int: selected_i1 = sum(i1 in 1..M, i2 in 1..MAX_K)(
            i1 * bool2int(T0[i1,j] != -1 /\ T1[i2,j] != -1)
        );
        var int: selected_i2 = sum(i1 in 1..M, i2 in 1..MAX_K)(
            i2 * bool2int(T0[i1,j] != -1 /\ T1[i2,j] != -1)
        );
    }} in 
    if (selected_i1 != 0 /\ selected_i2 != 0) then
        D01[j] = T1[selected_i2, j] - T0[selected_i1, j] /\
        D01[j] >= 1 /\
        D01_START[j] = T0[selected_i1, j] /\
        D01_END[j] = T1[selected_i2, j] 
    else 
        D01[j] = 0 /\
        D01_START[j] = 0 /\
        D01_END[j] = 0 
    endif
);


constraint forall(j in 1..WINDOW_SIZE)(
    let {{
        % i1 is a valid input channel and i2 is an output channel
        var int: selected_i1 = sum(i1 in 1..MAX_K, i2 in 1..N)(
            i1 * bool2int(T2[i1,j] != -1 /\ T3[i2,j] != -1)
        );
        var int: selected_i2 = sum(i1 in 1..MAX_K, i2 in 1..N)(
            i2 * bool2int(T2[i1,j] != -1 /\ T3[i2,j] != -1)
        );
    }} in 
    if (selected_i1 != 0 /\ selected_i2 != 0) then
        D23[j] = T3[selected_i2, j] - T2[selected_i1, j] /\
        D23[j] >= 1 /\
        D23_START[j] = T2[selected_i1, j] /\
        D23_END[j] = T3[selected_i2, j]
    else 
        D23[j] = 0 /\
        D23_START[j] = 0 /\
        D23_END[j] = 0
    endif
);

constraint forall(j in 1..WINDOW_SIZE)(D01[j] != 0);
constraint forall(j in 1..WINDOW_SIZE)(D23[j] != 0);

% apply NDF ordering for each OB bank to pressure the OB
constraint forall(k in 1..WINDOW_SIZE-1)(
    D01_END[SORTED_T3[k]] <= D01_END[SORTED_T3[k + 1]]
);

% constraints for limiting the buffer size 
constraint cumulative(
    [
        if i <= PREVIOUS_SIZE then 
            PREV_D01_START[i] 
        else 
            D01_START[i - PREVIOUS_SIZE] 
        endif 
        | i in 1..PREVIOUS_SIZE + WINDOW_SIZE
    ],
    [
        if i <= PREVIOUS_SIZE then 
            PREV_D01[i]  
        else 
            D01[i - PREVIOUS_SIZE]  
        endif
        | i in 1..PREVIOUS_SIZE + WINDOW_SIZE
    ],
    [
        1 | i in 1..PREVIOUS_SIZE + WINDOW_SIZE
    ],
    ob_cap
);

constraint cumulative(
    [
        if i <= PREVIOUS_SIZE then 
            PREV_D23_START[i] 
        else 
            D23_START[i - PREVIOUS_SIZE] 
        endif 
        | i in 1..PREVIOUS_SIZE + WINDOW_SIZE
    ],
    [
        if i <= PREVIOUS_SIZE then 
            PREV_D23[i]  
        else 
            D23[i - PREVIOUS_SIZE]  
        endif
        | i in 1..PREVIOUS_SIZE + WINDOW_SIZE
    ],
    [
        1 | i in 1..PREVIOUS_SIZE + WINDOW_SIZE
    ],
    ib_cap
);

%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
% Memory selection
"#);

        if constraints.output_memory_type == "fifo" {
            statements += &format!(r#"
% output fifo
constraint forall(j in 1..WINDOW_SIZE-1)(
    D01_END[SORTED_T0[j]] < D01_END[SORTED_T0[j + 1]]
);

% 1) constraint that previous final write time < current first wirte time 
constraint if PREVIOUS_SIZE != 0 then
    PREV_D01_START_LAST < D01_START[SORTED_T0[1]]
endif;

% 2) constraint that previous final read time < current first read time 
constraint if PREVIOUS_SIZE != 0 then
    PREV_D01_END_LAST < D01_END[SORTED_T0[1]]
endif;
"#);
        }

        if constraints.input_memory_type == "fifo" {
            statements += &format!(r#"
% input fifo
constraint forall(j in 1..WINDOW_SIZE-1)(
    D23_START[SORTED_T3[j]] < D23_START[SORTED_T3[j + 1]] 
);

% 1) constraint that previous final write time < current first wirte time 
constraint if PREVIOUS_SIZE != 0 then
    PREV_D23_START_LAST < D23_START[SORTED_T3[1]]
endif;

% 2) constraint that previous final read time < current first read time 
constraint if PREVIOUS_SIZE != 0 then
    PREV_D23_END_LAST < D23_END[SORTED_T0[1]]
endif;
"#);
        }
    
        statements += &format!(r#"
%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
% objective
% define costs of memory
var int: cost_output_buffers =
    memory_cost[1] * ob_cap * (ob_in_ports + ob_out_ports);

var int: cost_input_buffers =
    memory_cost[2] * ib_cap * (ib_in_ports + ib_out_ports);

var int: cost_wire = comm_width_cost * comm_cap;

var int: cost_function = cost_output_buffers + cost_input_buffers + cost_wire;
solve minimize cost_function;

% output
output [
  "cost_function:\(cost_function), comm_cap:\(comm_cap), ob_cap:\(ob_cap), ib_cap:\(ib_cap), "
  ++ "T0:\(T0), T1:\(T1), T2:\(T2), T3:\(T3), "
  ++ "D01_START:\(D01_START), D01_END:\(D01_END), D01:\(D01), "
  ++ "D23_START:\(D23_START), D23_END:\(D23_END), D23:\(D23)"
];
"#);

        /* solving the model */
        let mut solver = Solver::new(format!("solve_{}_{}_window-{}", constraints.edge_id, id, i), module_dir.clone());
        solver.add(statements);

        let (status, solutions) = solver.solve("cp-sat", 180, "-p 16")?;
        match status.as_str() {
            "OPTIMAL_SOLUTION" | "FEASIBLE" => {},
            _ => { 
                ret.cost = -1;
                return Ok(ret); 
            }
        };
        let parsed_json_value: serde_json::Value = serde_json::from_str(&solutions[0])?;

        // formatting output 
        let parse_vec_int_helper = |name: &str| -> Result<Vec<i32>, Box<dyn std::error::Error>> {
            let json_array = parsed_json_value
                .get(name)
                .and_then(|v| v.as_array())
                .ok_or_else(|| format!("{} not found or not an array", name))?;
            
            Ok(json_array
                .iter()
                .map(|v| v.as_i64().unwrap_or(0) as i32)
                .collect()
            )
        };

        let parse_int_helper = |name: &str| -> Result<i32, Box<dyn std::error::Error>> {
            Ok(parsed_json_value
                .get(name)
                .and_then(|v| v.as_i64())
                .map(|i| i as i32)
                .ok_or_else(|| format!("{} not found or not an int", name))?
            ) 
        };

        ret.cost = parse_int_helper("cost_function")?;
        ret.ob_size = parse_int_helper("ob_cap")?;
        ret.ib_size = parse_int_helper("ib_cap")?;
        ret.comm_cap = parse_int_helper("comm_cap")?;

        let mut tmp_array: Vec<i32> = vec![];

        tmp_array = parse_vec_int_helper("T1")?;
        if i == 0 {
            ret.t1 = Array2::from_shape_vec((number_of_communications as usize, window_size as usize), tmp_array)?;
        } else {
            ret.t1 = concatenate![Axis(1), ret.t1, Array2::from_shape_vec((number_of_communications as usize, window_size as usize), tmp_array)?];
        }

        tmp_array = parse_vec_int_helper("T2")?;
        if i == 0 {
            ret.t2 = Array2::from_shape_vec((number_of_communications as usize, window_size as usize), tmp_array)?;
        } else {
            ret.t2 = concatenate![Axis(1), ret.t2, Array2::from_shape_vec((number_of_communications as usize, window_size as usize), tmp_array)?];
        }

        tmp_array = parse_vec_int_helper("D01")?;
        prev_d01.extend(&tmp_array);       

        tmp_array = parse_vec_int_helper("D01_START")?;
        prev_d01_start.extend(&tmp_array);

        tmp_array = parse_vec_int_helper("D01_END")?;
        prev_d01_end_last = tmp_array.iter().copied().max().unwrap_or(0);
        
        tmp_array = parse_vec_int_helper("D23")?;
        prev_d23.extend(&tmp_array);
        
        tmp_array = parse_vec_int_helper("D23_START")?;
        prev_d23_start.extend(&tmp_array);
 
        tmp_array = parse_vec_int_helper("D23_END")?;
        prev_d23_end_last = tmp_array.iter().copied().max().unwrap_or(0);   

    }

    Ok(ret)
}




fn generate_partition(
    remaining_indices: &[i32],
    current_partition: Vec<Vec<i32>>,
    solution: &mut Vec<Vec<Vec<i32>>>,
) {

    // base case
    if remaining_indices.is_empty() {
        solution.push(current_partition);
        return;
    }

    // k - block length
    for k in 1..=remaining_indices.len() {
        let new_block_slice = &remaining_indices[0..k];
        let new_block: Vec<i32> = new_block_slice.to_vec();
        let remainder_slice = &remaining_indices[k..];

        let mut next_partition = current_partition.clone();
        next_partition.push(new_block);

        // recurse with the remainder and updated partition
        generate_partition(
            remainder_slice,
            next_partition,
            solution,
        );
    }
}


/// Select up to 5 representative partitions:
/// - Always include the first and last partitions.
/// - Evenly sample the middle ones if there are more than 5.
/// - Handle cases with fewer than 5 or just one partition.
fn select_partitions(
    partitions: &mut Vec<Vec<Vec<i32>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let n = partitions.len();

    // handle 0 partition
    if n == 0 {
        return Err(format!("no partition is provided for memory synthesis").into());
    // if we have fewer than 5 partitions, return them all
    } else if n <= 5 {
        return Ok(());
    }

    // otherwise, pick 5 evenly spaced partitions (including first & last)
    let mut selected = Vec::new();
    selected.push(partitions[0].clone()); // always include first

    // compute evenly spaced indices for middle partitions
    let step = (n - 1) as f64 / 4.0;
    for i in 1..4 {
        let idx = (i as f64 * step).round() as usize;
        selected.push(partitions[idx].clone());
    }

    selected.push(partitions[n - 1].clone()); // always include last
    
    *partitions = selected;
    
    Ok(())
}


// we break down the problem into two steps:
// 1) configurations - finding what banking and memory types are best
// 2) scheduling - finding the exact schedule for the problem
pub fn explore_memory_space(
    constraints: &mut MemoryConstraint,
    src_groups: &mut Vec<Vec<i32>>,
    dst_groups: &mut Vec<Vec<i32>>,
    output_patterns: &HashMap<i32, (i32, i32)>,
    input_patterns: &HashMap<i32, (i32, i32)>,
    module_dir: String,
) -> Result<Vec<MemoryBankInfo>, Box<dyn std::error::Error>> {

    assert!(!dst_groups.is_empty());
    
    let initial_indices: Vec<i32> = (0..dst_groups.len() as i32).collect();
    let mut partitions: Vec<Vec<Vec<i32>>> = Vec::new(); 

    // get all possible partitions
    generate_partition(&initial_indices, Vec::new(), &mut partitions);

    // the number of partitions can grow exponentially, 
    // so we pick up only some of them.
    // This number is now limited to 5 possible partitions
    // where the first and last ones are always be picked
    select_partitions(&mut partitions)?;

    debug!("src_groups = {:?}", src_groups);
    debug!("dst_groups = {:?}", dst_groups);
    debug!("selected partitions = {:?}", partitions);

    // In the optimisation, we should be able to determine, which partitions
    // are definitely not gonna reduce the cost function, and have them removed 
    // before even passing through the logic to improve compute time.
    let mut all_costs: Vec<Vec<i32>> = Vec::new();
    let mut all_configs: Vec<Vec<(String, String)>> = Vec::new();
    let mut all_sizes: Vec<Vec<(i32, i32, i32)>> = Vec::new();

    for (partition_index, partition) in partitions.iter().enumerate() {
        let mut costs: Vec<i32> = Vec::new();
        let mut configs: Vec<(String, String)> = Vec::new();
        let mut sizes: Vec<(i32, i32, i32)> = Vec::new();
            
        // solve the cost function for each partition 
        for (segment_index, segment) in partition.iter().enumerate() {
            // collect sources/targets
            let mut current_sources: Vec<i32> = Vec::new();
            let mut current_targets: Vec<i32> = Vec::new();

            for &i in segment {
                current_sources.extend_from_slice(&src_groups[i as usize]);
                current_targets.extend_from_slice(&dst_groups[i as usize]);
            }
            
            if current_sources.is_empty() || current_targets.is_empty() {
                return Err(format!(
                    "fail to create partitions for memory synthesis for edge {}", 
                    constraints.edge_id
                ).into());
            }

            // define possible hardware configurations
            let hardware_config: Vec<(&str, &str)> = if current_sources.len() == 1 && current_targets.len() == 1 {
                vec![("fifo", "fifo"), ("fifo", "ram"), ("ram", "fifo"), ("ram", "ram")]
            } else if current_sources.len() <= 2 && current_targets.len() <= 2 {
                vec![("ram", "ram")]
            } else if current_sources.len() <= 2 {
                vec![("ram", "rf")]
            } else if current_targets.len() <= 2 {
                vec![("rf", "ram")]
            } else {
                vec![("rf", "rf")]
            };

            // get relevant patterns
            let mut working_output_patterns: Vec<_> = output_patterns
                .iter()
                .filter(|(_, (c, _))| current_sources.contains(c))
                .map(|(a, (c, t))| (*a, *c, *t)) // deref so we own i32, not refs
                .collect();

            let mut working_input_patterns: Vec<_> = input_patterns
                .iter()
                .filter(|(_, (c, _))| current_targets.contains(c))
                .map(|(a, (c, t))| (*a, *c, *t))
                .collect();

            working_output_patterns.sort_by_key(|(addr, _, _)| *addr);
            working_input_patterns.sort_by_key(|(addr, _, _)| *addr);

            let out_addrs: Vec<_> = working_output_patterns.iter().map(|(addr, _, _)| *addr).collect();
            let in_addrs: Vec<_> = working_input_patterns.iter().map(|(addr, _, _)| *addr).collect();
            
            if out_addrs != in_addrs {
                //error!("displaying out_addrs: {:?}", out_addrs);
                //error!("displaying in_addrs: {:?}", in_addrs);
                return Err(format!("cannot extract working address channel patterns").into());
            }
                
            // select hardware config with minimum cost 
            let mut best_cost = i32::MAX;
            let mut best_config = ("".to_string(), "".to_string());
            let mut best_size = (0, 0, 0);

            for (src_type, dst_type) in hardware_config.iter() {
                constraints.output_memory_type = src_type.to_string();
                constraints.input_memory_type = dst_type.to_string();
                constraints.fixed = false;
                
                let id = format!("{}_{}_{}_{}", partition_index, segment_index, src_type, dst_type);
                
                let info = optimise_memory(
                    &constraints,
                    &id,
                    &working_output_patterns,
                    &working_input_patterns,
                    module_dir.clone(),
                )?;   
                
                if info.cost != -1 && info.cost < best_cost {
                    best_cost = info.cost;
                    best_config = (src_type.to_string(), dst_type.to_string());
                    best_size = (info.ob_size, info.ib_size, info.comm_cap);
                    
                    // fifo-to-fifo should be optimal 
                    if *src_type == "fifo" && *dst_type == "fifo" {
                        break; 
                    }
                } 
            }
            
            costs.push(best_cost); 
            configs.push(best_config); 
            sizes.push(best_size); 
        }

        all_costs.push(costs);
        all_configs.push(configs);
        all_sizes.push(sizes);
    }

    if all_costs.is_empty() {
        return Err(format!(
            "fail to find the optimal hardware configurations for edge {}",
            constraints.edge_id
        )
        .into());
    }

    // find partition with minimal total cost
    let (optimal_index, _) = all_costs
        .iter()
        .enumerate()
        .filter(|(_, costs)| !costs.iter().any(|&c| c == i32::MAX))
        .min_by_key(|(_, costs)| costs.iter().sum::<i32>())
        .ok_or("cannot find minimal partition")?;

    let optimal_partition = &partitions[optimal_index];
    let optimal_segment_hardware_configs = &all_configs[optimal_index];
    let optimal_segment_sizes = &all_sizes[optimal_index];

    constraints.fixed = true;
   

    // display some information
    debug!("memory synthesis at {} with partition: {:?}, configs: {:?}, sizes: {:?}", 
        constraints.edge_id,
        optimal_partition,
        optimal_segment_hardware_configs,
        optimal_segment_sizes,
    );


    // solutions
    let mut solutions: Vec<MemoryBankInfo> = Vec::new();

    // assign new groups 
    let mut new_src_groups: Vec<Vec<i32>> = Vec::new();
    let mut new_dst_groups: Vec<Vec<i32>> = Vec::new();

    // go over the optimisation problem again with fixed settings to produce the schedules
    for (segment_index, segment) in optimal_partition.iter().enumerate() {
        // collect sources/targets
        let mut current_sources: Vec<i32> = Vec::new();
        let mut current_targets: Vec<i32> = Vec::new();

        for &i in segment {
            current_sources.extend_from_slice(&src_groups[i as usize]);
            current_targets.extend_from_slice(&dst_groups[i as usize]);
        }

        new_src_groups.push(current_sources.clone());
        new_dst_groups.push(current_targets.clone());
        
        if current_sources.is_empty() || current_targets.is_empty() {
            return Err(format!(
                "fail to create partitions for memory synthesis for edge {}", 
                constraints.edge_id
            ).into());
        }

        // get relevant patterns
        let mut working_output_patterns: Vec<_> = output_patterns
            .iter()
            .filter(|(_, (c, _))| current_sources.contains(c))
            .map(|(a, (c, t))| (*a, *c, *t)) // deref so we own i32, not refs
            .collect();

        let mut working_input_patterns: Vec<_> = input_patterns
            .iter()
            .filter(|(_, (c, _))| current_targets.contains(c))
            .map(|(a, (c, t))| (*a, *c, *t))
            .collect();

        working_output_patterns.sort_by_key(|(addr, _, _)| *addr);
        working_input_patterns.sort_by_key(|(addr, _, _)| *addr);

        let out_addrs: Vec<_> = working_output_patterns.iter().map(|(addr, _, _)| *addr).collect();
        let in_addrs: Vec<_> = working_input_patterns.iter().map(|(addr, _, _)| *addr).collect();
        
        if out_addrs != in_addrs {
            //error!("displaying out_addrs: {:?}", out_addrs);
            //error!("displaying in_addrs: {:?}", in_addrs);
            return Err(format!("cannot extract working address channel patterns").into());
        }
     
        let (src_type, dst_type) = &optimal_segment_hardware_configs[segment_index];
        constraints.output_memory_type = src_type.to_string();
        constraints.input_memory_type = dst_type.to_string();        

        let (output_size, input_size, comm_cap) = optimal_segment_sizes[segment_index];
        constraints.output_buffer_size = output_size; 
        constraints.input_buffer_size = input_size; 
        constraints.channel_width_size = comm_cap; 

        let id = format!("optimal_{}", segment_index);

        let info = optimise_memory(
            &constraints,
            &id,
            &working_output_patterns,
            &working_input_patterns,
            module_dir.clone(),
        )?;

        if info.cost == -1 {
            return Err(format!(
                "fail to rerun the optimisation problem for edge {}", 
                constraints.edge_id
            ).into());
        }
        
        solutions.push(info);
    }

    *src_groups = new_src_groups;
    *dst_groups = new_dst_groups;

    Ok(solutions) 
}

