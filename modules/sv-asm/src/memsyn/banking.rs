use sv_lib::model::{DataBase};
use sv_lib::solver::{Solver};
use std::collections::{HashMap};
use ndarray::Array2;
use std::fmt::Display;


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


pub fn optimise_memory(
    id: i32, 
    db: &DataBase,
    edge_id: &str,
    output_patterns: &Vec<(i32, i32, i32)>,
    input_patterns: &Vec<(i32, i32, i32)>,
    module_dir: String,
) -> Result<(), Box<dyn std::error::Error>> {

    let edge = db.app_graph.edges.iter().find(|e| e.id == edge_id).ok_or("edge not found")?;
    let number_of_tokens = output_patterns.len();
    let src_fire_time = db.synthesized_information.node_fire_times.get(&edge.source_node).ok_or(format!("cannot find fire time of {} node", &edge.source_node))?;
    let dst_fire_time = db.synthesized_information.node_fire_times.get(&edge.target_node).ok_or(format!("cannot find fire time of {} node", &edge.target_node))?;
    let routing_delay = db.synthesized_information.routing_paths.iter().find(|r| r.app_edge_id == edge_id).map(|r| r.delay).ok_or("Cannot find an edge in the routing paths")?;

    // These parameters are upper bound. Good results should acheive less than these
    let total_output_buffer = db.synthesized_information.output_buffer_size.get(&edge.source_node).ok_or(format!("cannot find output buffer size of {} node", &edge.source_node))?;
    let total_input_buffer = db.synthesized_information.input_buffer_size.get(&edge.target_node).ok_or(format!("cannot find input buffer size of {} node", &edge.target_node))?;
    let total_communication_channel = db.synthesized_information.channel_width.get(&format!("transporter_{}", edge.id)).ok_or(format!("cannot find channel width of {} edge", &edge.id))?;
     
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
        producing_matrix[(row as usize, col)] = time + src_fire_time;
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
        consuming_matrix[(row as usize, col)] = time + dst_fire_time;
    }

    // sorted column indices of consuming matrix 
    let mut sorted_consuming_indices: Vec<_> = (0..output_patterns.len()).map(|i| i as i32).collect();   
    sorted_consuming_indices.sort_by_key(|&i| output_patterns[i as usize].2);
    sorted_consuming_indices = sorted_consuming_indices.into_iter().map(|i| i + 1).collect(); // new indexing

    // maximum number of time in the scope 
    let maximum_delay = std::cmp::max(
        producing_matrix.iter().max().unwrap(), 
        consuming_matrix.iter().max().unwrap()
    ) + 1;

    let number_of_producers = mapping_output_channels.len() as i32;
    let number_of_consumers = mapping_input_channels.len() as i32;
    let maximum_reg_file_port_cap = *vec![
        number_of_producers,
        number_of_consumers,
        *total_communication_channel,
    ].iter().max().unwrap();

    // work on this statement if sharing a channel to different banks is permitted
    let statements = format!(r#"include "cumulative.mzn";
% problem to select memory banks and map channels to minimize cost 

% Parameters 
int: TOKEN_SIZE = {TOKEN_SIZE};
int: M = {PRODUCING_CHANNELS}; % number of proceducer channels to OB
int: N = {CONSUMING_CHANNELS}; % number of consumer channels from IB
int: MAX_K = {MAXIMUM_CHANNEL_WIDTH}; % communication channels between OB and IB
int: MAX_OB_SIZE = {MAXIMUM_OUTPUT_BUFFER_SIZE};
int: MAX_IB_SIZE = {MAXIMUM_INPUT_BUFFER_SIZE};
int: MAX_DELAY = {MAXIMUM_DELAY};

%%%% LIMITING TO FIND A 1-BANK SOLUTION BECAUSE OF CHANNELS CANNOT BE SHARED %%%%
int: MAX_OB_BANK = 1; % min(M,MAX_K);
int: MAX_IB_BANK = 1; % min(N,MAX_K);

% addresss patterns 
array [1..M,1..TOKEN_SIZE] of int: T0 = {PRODUCING_MATRIX}; % including fire time 
array [1..N,1..TOKEN_SIZE] of int: T3 = {CONSUMING_MATRIX}; % including fire time 
array [1..TOKEN_SIZE] of int: SORTED_T3 = {SORTED_OUTPUT_PATTERN_INDICES:?};
int: comm_delay = {WIRE_DELAY};

% time and channel patterns 
% memory characteristics 
enum MemType = {MEMORY_TYPES};
array [MemType] of int: port_capacity = {MEMORY_PORT_CAPACITY:?};
array [MemType] of int: memory_minimum_size = {MEMORY_MINIMUM_SIZE:?};
array [MemType] of int: memory_cost = {MEMORY_COST:?};
int: comm_width_cost = {COMMUNICATION_COST};

%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
% Decision variables and constraints

%%%%%%%%%%%%%%%%%%%
% Banking decision
array [1..MAX_OB_BANK] of var 0..1: OB_BANK_USED;
array [1..MAX_OB_BANK,1..M] of var 0..1: IN_OB_BANK;
array [1..MAX_OB_BANK,1..MAX_K] of var 0..1: OUT_OB_BANK;

array [1..MAX_IB_BANK] of var 0..1: IB_BANK_USED;
array [1..MAX_IB_BANK,1..MAX_K] of var 0..1: IN_IB_BANK;
array [1..MAX_IB_BANK,1..N] of var 0..1: OUT_IB_BANK;

constraint forall(i in 1..MAX_OB_BANK)(
    exists(j in 1..M)(IN_OB_BANK[i,j] = 1) <-> exists(j in 1..MAX_K)(OUT_OB_BANK[i,j] = 1)  
);
constraint forall(i in 1..MAX_OB_BANK)(
    if (exists(j in 1..M)(IN_OB_BANK[i,j] = 1)) then 
        OB_BANK_USED[i] = 1
    else 
        OB_BANK_USED[i] = 0
    endif
);

constraint forall(i in 1..MAX_IB_BANK)(
    exists(j in 1..MAX_K)(IN_IB_BANK[i,j] = 1) <-> exists(j in 1..N)(OUT_IB_BANK[i,j] = 1)  
);
constraint forall(i in 1..MAX_IB_BANK)(
    if (exists(j in 1..MAX_K)(IN_IB_BANK[i,j] = 1)) then 
        IB_BANK_USED[i] = 1
    else 
        IB_BANK_USED[i] = 0
    endif
);

% one input channel can connect to multiple buffers, 
% but output channel is strictly binded to one buffer 
constraint forall(j in 1..M)(
    sum(i in 1..MAX_OB_BANK)(bool2int(IN_OB_BANK[i,j] = 1)) >= 1
);
constraint forall(j in 1..MAX_K)(
    sum(i in 1..MAX_OB_BANK)(bool2int(OUT_OB_BANK[i,j] = 1)) <= 1
);
constraint forall(j in 1..MAX_K)(
    (sum(i in 1..MAX_OB_BANK)(bool2int(OUT_OB_BANK[i,j] = 1)) = 1) ->
        sum(i in 1..MAX_IB_BANK)(bool2int(IN_IB_BANK[i,j] = 1)) >= 1
);
constraint forall(j in 1..N)(
    sum(i in 1..MAX_IB_BANK)(bool2int(OUT_IB_BANK[i,j] = 1)) = 1
);

%%%%%%%%%%%%%%%%%%%
% memory detailed variables  
var 1..MAX_K: comm_cap;
array [1..MAX_OB_BANK] of var 0..MAX_OB_SIZE: ob_cap;
array [1..MAX_IB_BANK] of var 0..MAX_IB_SIZE: ib_cap;

array [1..MAX_OB_BANK] of var 0..M: ob_in_ports;
array [1..MAX_OB_BANK] of var 0..MAX_K: ob_out_ports;
array [1..MAX_IB_BANK] of var 0..MAX_K: ib_in_ports;
array [1..MAX_IB_BANK] of var 0..N: ib_out_ports;

array [1..MAX_OB_BANK] of var MemType: ob_type; 
array [1..MAX_IB_BANK] of var MemType: ib_type; 

array [1..MAX_K,1..TOKEN_SIZE] of var -1..MAX_DELAY: T1;
array [1..MAX_K,1..TOKEN_SIZE] of var -1..MAX_DELAY: T2;
array [1..MAX_K] of var bool: comm_used;

% value 0 means unused 
array [1..MAX_OB_BANK,1..TOKEN_SIZE] of var 0..MAX_DELAY: D01_INDEX;
array [1..MAX_OB_BANK,1..TOKEN_SIZE] of var 0..MAX_DELAY: D01_START;
array [1..MAX_OB_BANK,1..TOKEN_SIZE] of var 0..MAX_DELAY: D01_END;
array [1..MAX_OB_BANK,1..TOKEN_SIZE] of var 0..MAX_DELAY: D01;
array [1..MAX_IB_BANK,1..TOKEN_SIZE] of var 0..MAX_DELAY: D23_START;
array [1..MAX_IB_BANK,1..TOKEN_SIZE] of var 0..MAX_DELAY: D23_END;
array [1..MAX_IB_BANK,1..TOKEN_SIZE] of var 0..MAX_DELAY: D23;

%%%%%%%%%%%%%%%%%%%
% Constraints
% Each column has one non -1 value
constraint forall(j in 1..TOKEN_SIZE)(
    sum(i in 1..MAX_K)(bool2int(T1[i,j] != -1)) = 1
);
    
% Each row has unique non -1 value
constraint forall(i in 1..MAX_K, j1, j2 in 1..TOKEN_SIZE where j1 < j2)(
    (T1[i,j1] != -1 /\ T1[i,j2] != -1) -> T1[i,j1] != T1[i,j2]
);
 
% bind the variable to minimize the channel size 
constraint forall(i in 1..MAX_K)(
    comm_used[i] <-> exists(j in 1..TOKEN_SIZE)(T1[i,j] != -1)
);

constraint comm_cap = max([i | i in 1..MAX_K where comm_used[i] ] ++ [0]);

constraint forall(i in 1..MAX_K, j in 1..TOKEN_SIZE)(
    if T1[i,j] != -1 then
        T2[i,j] = T1[i,j] + comm_delay
    else
        T2[i,j] = -1
    endif
); 

% time interval in the buffers
% constraint that T1 > T0 and T3 > T2
% bind the banking to all T

constraint forall(b in 1..MAX_OB_BANK, j in 1..TOKEN_SIZE)(
    let {{
        % i1 is a valid input channel and i2 is an output channel
        var int: selected_i1 = sum(i1 in 1..M, i2 in 1..MAX_K)(
            i1 * bool2int(IN_OB_BANK[b,i1] = 1 /\ OUT_OB_BANK[b,i2] = 1 /\ T0[i1,j] != -1 /\ T1[i2,j] != -1)
        );
        var int: selected_i2 = sum(i1 in 1..M, i2 in 1..MAX_K)(
            i2 * bool2int(IN_OB_BANK[b,i1] = 1 /\ OUT_OB_BANK[b,i2] = 1 /\ T0[i1,j] != -1 /\ T1[i2,j] != -1)
        );
    }} in 
    if (selected_i1 != 0 /\ selected_i2 != 0) then
        D01[b,j] = T1[selected_i2, j] - T0[selected_i1, j] /\
        D01[b,j] >= 1 /\
        D01_START[b,j] = T0[selected_i1, j] /\
        D01_END[b,j] = T1[selected_i2, j] /\
        D01_INDEX[b,j] = SORTED_T3[j]
    else 
        D01[b,j] = 0 /\
        D01_START[b,j] = 0 /\
        D01_END[b,j] = 0 /\
        D01_INDEX[b,j] = 0
    endif
);


constraint forall(b in 1..MAX_IB_BANK, j in 1..TOKEN_SIZE)(
    let {{
        % i1 is a valid input channel and i2 is an output channel
        var int: selected_i1 = sum(i1 in 1..MAX_K, i2 in 1..N)(
            i1 * bool2int(IN_IB_BANK[b,i1] = 1 /\ OUT_IB_BANK[b,i2] = 1 /\ T2[i1,j] != -1 /\ T3[i2,j] != -1)
        );
        var int: selected_i2 = sum(i1 in 1..MAX_K, i2 in 1..N)(
            i2 * bool2int(IN_IB_BANK[b,i1] = 1 /\ OUT_IB_BANK[b,i2] = 1 /\ T2[i1,j] != -1 /\ T3[i2,j] != -1)
        );
    }} in 
    if (selected_i1 != 0 /\ selected_i2 != 0) then
        D23[b,j] = T3[selected_i2, j] - T2[selected_i1, j] /\
        D23[b,j] >= 1 /\
        D23_START[b,j] = T2[selected_i1, j] /\
        D23_END[b,j] = T3[selected_i2, j]
    else 
        D23[b,j] = 0 /\
        D23_START[b,j] = 0 /\
        D23_END[b,j] = 0
    endif
);


constraint forall(j in 1..TOKEN_SIZE)(sum(b in 1..MAX_OB_BANK)(bool2int(D01[b,j] != 0)) = 1);
constraint forall(j in 1..TOKEN_SIZE)(sum(b in 1..MAX_IB_BANK)(bool2int(D23[b,j] != 0)) = 1);

% apply NDF ordering for each OB bank to pressure the OB
constraint forall(b in 1..MAX_OB_BANK)(
    forall(j1 in 1..TOKEN_SIZE-1, j2 in j1+1..TOKEN_SIZE)(
        (D01[b,j1] != 0 /\ D01[b,j2] != 0) ->
            D01_END[b,j1] <= D01_END[b, j2]
    )
);


% constraints for limiting the buffer size 
constraint forall(b in 1..MAX_OB_BANK)(
   cumulative(
       [if D01[b,i] != 0 then D01_START[b,i] else 0 endif | i in 1..TOKEN_SIZE],
       [if D01[b,i] != 0 then D01[b,i] else 0 endif | i in 1..TOKEN_SIZE],
       [if D01[b,i] != 0 then 1 else 0 endif | i in 1..TOKEN_SIZE],
       ob_cap[b]
   )
);

constraint forall(b in 1..MAX_IB_BANK)(
   cumulative(
       [if D23[b,i] != 0 then D23_START[b,i] else 0 endif | i in 1..TOKEN_SIZE],
       [if D23[b,i] != 0 then D23[b,i] else 0 endif | i in 1..TOKEN_SIZE],
       [if D23[b,i] != 0 then 1 else 0 endif | i in 1..TOKEN_SIZE],
       ib_cap[b]
   )
);

%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
% Memory selection

% to be scalable 
constraint forall(b in 1..MAX_OB_BANK)(ob_in_ports[b] = sum(i in 1..M)(bool2int(IN_OB_BANK[b,i] = 1)));
constraint forall(b in 1..MAX_OB_BANK)(ob_out_ports[b] = sum(i in 1..MAX_K)(bool2int(OUT_OB_BANK[b,i] = 1)));
constraint forall(b in 1..MAX_IB_BANK)(ib_in_ports[b] = sum(i in 1..MAX_K)(bool2int(IN_IB_BANK[b,i] = 1)));
constraint forall(b in 1..MAX_IB_BANK)(ib_out_ports[b] = sum(i in 1..N)(bool2int(OUT_IB_BANK[b,i] = 1)));

constraint forall(b in 1..MAX_OB_BANK)(port_capacity[ob_type[b]] >= ob_in_ports[b]); 
constraint forall(b in 1..MAX_OB_BANK)(port_capacity[ob_type[b]] >= ob_out_ports[b]); 
constraint forall(b in 1..MAX_IB_BANK)(port_capacity[ib_type[b]] >= ib_in_ports[b]); 
constraint forall(b in 1..MAX_IB_BANK)(port_capacity[ib_type[b]] >= ib_out_ports[b]); 
constraint forall(b in 1..MAX_OB_BANK)(memory_minimum_size[ob_type[b]] <= ob_cap[b]);
constraint forall(b in 1..MAX_IB_BANK)(memory_minimum_size[ib_type[b]] <= ib_cap[b]);

% fifo's patterns need to be made sure
constraint forall(b in 1..MAX_OB_BANK)(
    ob_in_ports[b] = 1 /\ ob_out_ports[b] = 1 /\ 
    forall(j1 in 1..TOKEN_SIZE-1, j2 in j1+1..TOKEN_SIZE)(
        (D01[b,j1] != 0 /\ D01[b,j2] != 0) -> D01_START[b,j1] < D01_START[b,j2]
    ) /\
    forall(j1 in 1..TOKEN_SIZE-1, j2 in j1+1..TOKEN_SIZE)(
        (D01[b,j1] != 0 /\ D01[b,j2] != 0) -> D01_END[b,j1] < D01_END[b,j2]
    ) <-> ob_type[b] = fifo
);

constraint forall(b in 1..MAX_IB_BANK)(
    ib_in_ports[b] = 1 /\ ib_out_ports[b] = 1 /\ 
    forall(j1 in 1..TOKEN_SIZE-1, j2 in j1+1..TOKEN_SIZE)(
        (D23[b,j1] != 0 /\ D23[b,j2] != 0) -> D23_START[b,j1] < D23_START[b,j2]
    ) /\
    forall(j1 in 1..TOKEN_SIZE-1, j2 in j1+1..TOKEN_SIZE)(
        (D23[b,j1] != 0 /\ D23[b,j2] != 0) -> D23_END[b,j1] < D23_END[b,j2]
    ) <-> ib_type[b] = fifo
);

%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
% objective
% define costs of memory 
var int: cost_output_buffers = sum(b in 1..MAX_OB_BANK)(
    if OB_BANK_USED[b] = 1 then 
        memory_cost[ob_type[b]] * ob_cap[b] * (ob_in_ports[b] + ob_out_ports[b])
    else 
        0
    endif
); 
var int: cost_input_buffers = sum(b in 1..MAX_IB_BANK)( 
    if IB_BANK_USED[b] = 1 then 
        memory_cost[ib_type[b]] * ib_cap[b] * (ib_in_ports[b] + ib_out_ports[b])
    else 
        0
    endif
); 
var int: cost_wire = comm_width_cost * comm_cap;
solve minimize cost_output_buffers + cost_input_buffers + cost_wire;

% output 
output [
  "{{D01_START: \(D01_START), D01: \(D01), D23_START: \(D23_START) D23: \(D23), IN_OB_BANK: \(IN_OB_BANK), OUT_OB_BANK: \(OUT_OB_BANK), IN_IB_BANK: \(IN_IB_BANK), OUT_IB_BANK: \(OUT_IB_BANK), ob_type: \(ob_type), ib_type: \(ib_type), comm_cap: \(comm_cap), ob_cap: \(ob_cap), ib_cap: \(ib_cap), T0:\(T0), T1:\(T1), T2:\(T2), T3:\(T3)}}" 
];"#,
        TOKEN_SIZE = number_of_tokens,
        PRODUCING_CHANNELS = number_of_producers,
        CONSUMING_CHANNELS = number_of_consumers,
        MAXIMUM_CHANNEL_WIDTH = total_communication_channel,
        MAXIMUM_OUTPUT_BUFFER_SIZE = total_output_buffer,
        MAXIMUM_INPUT_BUFFER_SIZE = total_input_buffer,
        MAXIMUM_DELAY = maximum_delay,
        PRODUCING_MATRIX = format_matrix(&producing_matrix),
        CONSUMING_MATRIX = format_matrix(&consuming_matrix),
        SORTED_OUTPUT_PATTERN_INDICES = sorted_consuming_indices,
        WIRE_DELAY = routing_delay,
        MEMORY_TYPES = format!("{{ {} }}", vec!["fifo", "reg_file", "sram"].join(", ")),
        MEMORY_PORT_CAPACITY = vec![1, maximum_reg_file_port_cap, 2],
        MEMORY_MINIMUM_SIZE = vec![1, 1, 1024],
        MEMORY_COST = vec![2, 3, 1],
        COMMUNICATION_COST = 1,
    );

    /* solving the model */
    let mut solver = Solver::new(format!("solve_memory_{}_{}", &edge.id, id), module_dir.clone());
    solver.add(statements);
    let (status, solutions) = solver.solve("cp-sat", 120, "-p 16")?;
    match status.as_str() {
        "OPTIMAL_SOLUTION" | "FEASIBLE" => {}
        _ => return Ok(()),
    };
    let parsed_json_value: serde_json::Value = serde_json::from_str(&solutions[0])?;
 
    // formatting output 




    Ok(())
}
