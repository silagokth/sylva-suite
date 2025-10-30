use sv_lib::solver::{Solver};
use std::collections::{HashMap};
use ndarray::Array2;
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
}

#[derive(Debug)]
pub struct MemoryBankInfo {
    // [bank id]
    pub ob_memory_types: Vec<String>,
    pub ib_memory_types: Vec<String>,
    pub ob_size: Vec<i32>,
    pub ib_size: Vec<i32>,
    // [bank id, channel]
    pub input_ob_channels: Array2<i32>, 
    pub output_ob_channels: Array2<i32>, 
    pub input_ib_channels: Array2<i32>,
    pub output_ib_channels: Array2<i32>,
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
    number: &mut i32, 
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

    // sorted column indices of consuming matrix 
    let mut sorted_consuming_indices: Vec<_> = (0..input_patterns.len()).map(|i| i as i32).collect();   
    sorted_consuming_indices.sort_by_key(|&i| input_patterns[i as usize].2);
    sorted_consuming_indices = sorted_consuming_indices.into_iter().map(|i| i + 1).collect(); 

    // maximum number of time in the scope 
    let maximum_delay = std::cmp::max(
        producing_matrix.iter().max().unwrap(), 
        consuming_matrix.iter().max().unwrap()
    ) + 1;

    let number_of_producers = mapping_output_channels.len() as i32;
    let number_of_consumers = mapping_input_channels.len() as i32;
    let number_of_communications = *vec![
        number_of_producers,
        number_of_consumers,
        constraints.channel_width_size,
    ].iter().min().unwrap();
    let maximum_reg_file_port_cap = *vec![
        number_of_producers,
        number_of_consumers,
        constraints.channel_width_size,
    ].iter().max().unwrap();

    let max_ob_bank = 1; // set to min(M,K)
    let max_ib_bank = 1; // set to min(N,K)

    fn next_power_of_two(x: i32) -> i32 {
        if x <= 1 {
            return 2;
        }
        ((x as u32).next_power_of_two()) as i32
    }

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

%%%% LIMITING TO FIND A 1-BANK SOLUTION BECAUSE CHANNELS CANNOT BE SHARED %%%%
int: MAX_OB_BANK = {MAX_OB_BANK}; % min(M,MAX_K);
int: MAX_IB_BANK = {MAX_IB_BANK}; % min(N,MAX_K);

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
        sum(i in 1..MAX_IB_BANK)(bool2int(IN_IB_BANK[i,j] = 1)) = 1
);
constraint forall(j in 1..N)(
    sum(i in 1..MAX_IB_BANK)(bool2int(OUT_IB_BANK[i,j] = 1)) = 1
);

% geometry constraints
% memory banks must be continuous 
constraint forall(b in 1..MAX_OB_BANK)(
    forall(j in 2..M)(
        (IN_OB_BANK[b,j] - IN_OB_BANK[b,j-1] < 0) -> % falling edge
        forall(k in j..M)(IN_OB_BANK[b,k] = 0)
    )
);

constraint forall(b in 1..MAX_IB_BANK)(
    forall(j in 2..N)(
        (OUT_IB_BANK[b,j] - OUT_IB_BANK[b,j-1] < 0) -> % falling edge
        forall(k in j..M)(OUT_IB_BANK[b,k] = 0)
    )
);

constraint forall(k in 2..MAX_K)(
    comm_used[k-1] >= comm_used[k]
);

%%%%%%%%%%%%%%%%%%%
% memory detailed variables  
var 1..MAX_K: comm_cap;
array [1..MAX_OB_BANK] of var 0..MAX_OB_SIZE: ob_cap;
array [1..MAX_IB_BANK] of var 0..MAX_IB_SIZE: ib_cap;
array [1..MAX_OB_BANK] of var 1..16: ob_cap_power;
array [1..MAX_IB_BANK] of var 1..16: ib_cap_power;

% force the buffer to be 2 ** N size
constraint forall(b in 1..MAX_OB_BANK)(ob_cap[b] = 2 ^ ob_cap_power[b]);
constraint forall(b in 1..MAX_IB_BANK)(ib_cap[b] = 2 ^ ib_cap_power[b]);

array [1..MAX_OB_BANK] of var 0..M: ob_in_ports;
array [1..MAX_OB_BANK] of var 0..MAX_K: ob_out_ports;
array [1..MAX_IB_BANK] of var 0..MAX_K: ib_in_ports;
array [1..MAX_IB_BANK] of var 0..N: ib_out_ports;

array [1..MAX_OB_BANK] of var MemType: ob_type; 
array [1..MAX_IB_BANK] of var MemType: ib_type; 

array [1..MAX_K,1..TOKEN_SIZE] of var -1..MAX_DELAY: T1;
array [1..MAX_K,1..TOKEN_SIZE] of var -1..MAX_DELAY: T2;
array [1..MAX_K] of var bool: comm_used; % update this for banking

% value 0 means unused 
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

constraint comm_cap = sum(i in 1..MAX_K)(bool2int(comm_used[i]));

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
        D01_END[b,j] = T1[selected_i2, j] 
    else 
        D01[b,j] = 0 /\
        D01_START[b,j] = 0 /\
        D01_END[b,j] = 0 
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
    forall(k1 in 1..TOKEN_SIZE-1, k2 in k1+1..TOKEN_SIZE)(
        let {{
            int: j1 = SORTED_T3[k1],
            int: j2 = SORTED_T3[k2]
        }} in
        (D01[b,j1] != 0 /\ D01[b,j2] != 0) ->
            D01_END[b,j1] <= D01_END[b,j2]
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
constraint forall(b in 1..MAX_OB_BANK, j in 1..TOKEN_SIZE-1) (
    if ob_type[b] = fifo then
        if D01[b,j] != 0 then
            % Look ahead for the first used token k > j
            forall(k in j+1..TOKEN_SIZE) (
                if D01[b,k] != 0 then
                    % Once the next used token k is found, enforce the order
                    % And ensure no token l between j and k is used
                    if forall(l in j+1..k-1) (D01[b,l] = 0) then
                        D01_START[b,j] < D01_START[b,k]
                    endif
                endif
            )
        endif
    endif
);

constraint forall(b in 1..MAX_IB_BANK, j in 1..TOKEN_SIZE-1) (
    if ib_type[b] = fifo then
        if D23[b,j] != 0 then
            % Look ahead for the first used token k > j
            forall(k in j+1..TOKEN_SIZE) (
                if D23[b,k] != 0 then
                    % Once the next used token k is found, enforce the order
                    % And ensure no token l between j and k is used
                    if forall(l in j+1..k-1) (D23[b,l] = 0) then
                        D23_START[b,j] < D23_START[b,k]
                    endif
                endif
            )
        endif
    endif
);

%constraint forall(b in 1..MAX_OB_BANK)(
%    ob_in_ports[b] = 1 /\ ob_out_ports[b] = 1 /\ 
%    forall(j1 in 1..TOKEN_SIZE-1, j2 in j1+1..TOKEN_SIZE)(
%        (D01[b,j1] != 0 /\ D01[b,j2] != 0) -> D01_START[b,j1] < D01_START[b,j2]
%    ) /\
%    forall(j1 in 1..TOKEN_SIZE-1, j2 in j1+1..TOKEN_SIZE)(
%        (D01[b,j1] != 0 /\ D01[b,j2] != 0) -> D01_END[b,j1] < D01_END[b,j2]
%    ) <-> ob_type[b] = fifo
%);
%
%constraint forall(b in 1..MAX_IB_BANK)(
%    ib_in_ports[b] = 1 /\ ib_out_ports[b] = 1 /\ 
%    forall(j1 in 1..TOKEN_SIZE-1, j2 in j1+1..TOKEN_SIZE)(
%        (D23[b,j1] != 0 /\ D23[b,j2] != 0) -> D23_START[b,j1] < D23_START[b,j2]
%    ) /\
%    forall(j1 in 1..TOKEN_SIZE-1, j2 in j1+1..TOKEN_SIZE)(
%        (D23[b,j1] != 0 /\ D23[b,j2] != 0) -> D23_END[b,j1] < D23_END[b,j2]
%    ) <-> ib_type[b] = fifo
%);

constraint forall(b in 1..MAX_OB_BANK)(
    (OB_BANK_USED[b] = 0) -> ob_type[b] = none
);

constraint forall(b in 1..MAX_IB_BANK)(
    (IB_BANK_USED[b] = 0) -> ib_type[b] = none
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
  "{{IN_OB_BANK:\(IN_OB_BANK), OUT_OB_BANK:\(OUT_OB_BANK), IN_IB_BANK:\(IN_IB_BANK), OUT_IB_BANK:\(OUT_IB_BANK), " ++
  "ob_type:[" ++ join(", ", [ "\"\(s)\"" | s in ob_type ]) ++ "], " ++ 
  "ib_type:[" ++ join(", ", [ "\"\(s)\"" | s in ib_type ]) ++ "], " ++ 
  "comm_cap:\(comm_cap), ob_cap:\(ob_cap), ib_cap:\(ib_cap), T0:\(T0), T1:\(T1), T2:\(T2), T3:\(T3)}}" 
];"#,
        TOKEN_SIZE = number_of_tokens,
        PRODUCING_CHANNELS = number_of_producers,
        CONSUMING_CHANNELS = number_of_consumers,
        MAXIMUM_CHANNEL_WIDTH = number_of_communications,
        MAXIMUM_OUTPUT_BUFFER_SIZE = next_power_of_two(constraints.output_buffer_size),
        MAXIMUM_INPUT_BUFFER_SIZE = next_power_of_two(constraints.input_buffer_size),
        MAXIMUM_DELAY = maximum_delay,
        MAX_OB_BANK = max_ob_bank,
        MAX_IB_BANK = max_ib_bank,
        PRODUCING_MATRIX = format_matrix(&producing_matrix),
        CONSUMING_MATRIX = format_matrix(&consuming_matrix),
        SORTED_OUTPUT_PATTERN_INDICES = sorted_consuming_indices,
        WIRE_DELAY = constraints.routing_delay,
        MEMORY_TYPES = format!("{{ {} }}", vec!["none", "fifo", "reg_file", "sram"].join(", ")),
        MEMORY_PORT_CAPACITY = vec![0, 1, maximum_reg_file_port_cap, 2],
        MEMORY_MINIMUM_SIZE = vec![0, 1, 1, 1024],
        MEMORY_COST = vec![100, 2, 3, 1],
        COMMUNICATION_COST = 30,
    );

    /* solving the model */
    let mut solver = Solver::new(format!("solve_memory_{}_{}", constraints.edge_id, number), module_dir.clone());
    solver.add(statements);
    *number = *number + 1;
    let (status, solutions) = solver.solve("cp-sat", 180, "-p 16")?;
    match status.as_str() {
        "OPTIMAL_SOLUTION" | "FEASIBLE" => {}
        _ => return Err(format!("fail to optimize the memory banking under the constraint programming").into()),
    };
    let parsed_json_value: serde_json::Value = serde_json::from_str(&solutions[0])?;

    // define an empty struct to be returned
    let mut ret = MemoryBankInfo {
        ob_memory_types: vec![],
        ib_memory_types: vec![],
        ob_size: vec![],
        ib_size: vec![],
        input_ob_channels: Array2::zeros((0, 0)),
        output_ob_channels: Array2::zeros((0, 0)),
        input_ib_channels: Array2::zeros((0, 0)),
        output_ib_channels: Array2::zeros((0, 0)),
        t0: Array2::zeros((0, 0)),
        t1: Array2::zeros((0, 0)),
        t2: Array2::zeros((0, 0)),
        t3: Array2::zeros((0, 0)),
    };

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

    let parse_vec_str_helper = |name: &str| -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let json_array = parsed_json_value
            .get(name)
            .and_then(|v| v.as_array())
            .ok_or_else(|| format!("{} not found or not an array", name))?;
         
        Ok(json_array
            .iter()
            .map(|v| v.as_str().unwrap_or("").to_string())
            .collect()
        )
    };

    ret.ob_memory_types = parse_vec_str_helper("ob_type")?; 
    ret.ib_memory_types = parse_vec_str_helper("ib_type")?;
    ret.ob_size = parse_vec_int_helper("ob_cap")?;
    ret.ib_size = parse_vec_int_helper("ib_cap")?;

    let mut tmp_array: Vec<i32> = vec![];

    tmp_array = parse_vec_int_helper("IN_OB_BANK")?;
    ret.input_ob_channels = Array2::from_shape_vec((max_ob_bank as usize, number_of_producers as usize), tmp_array)?;

    tmp_array = parse_vec_int_helper("OUT_OB_BANK")?;
    ret.output_ob_channels = Array2::from_shape_vec((max_ob_bank as usize, number_of_communications as usize), tmp_array)?;

    tmp_array = parse_vec_int_helper("IN_IB_BANK")?;
    ret.input_ib_channels = Array2::from_shape_vec((max_ib_bank as usize, number_of_communications as usize), tmp_array)?;

    tmp_array = parse_vec_int_helper("OUT_IB_BANK")?;
    ret.output_ib_channels = Array2::from_shape_vec((max_ib_bank as usize, number_of_consumers as usize), tmp_array)?;

    tmp_array = parse_vec_int_helper("T0")?;
    ret.t0 = Array2::from_shape_vec((number_of_producers as usize, number_of_tokens as usize), tmp_array)?;

    tmp_array = parse_vec_int_helper("T1")?;
    ret.t1 = Array2::from_shape_vec((number_of_communications as usize, number_of_tokens as usize), tmp_array)?;

    tmp_array = parse_vec_int_helper("T2")?;
    ret.t2 = Array2::from_shape_vec((number_of_communications as usize, number_of_tokens as usize), tmp_array)?;

    tmp_array = parse_vec_int_helper("T3")?;
    ret.t3 = Array2::from_shape_vec((number_of_consumers as usize, number_of_tokens as usize), tmp_array)?;

    Ok(ret)
}
