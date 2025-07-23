include "cumulative.mzn";

int: MAX_DELAY = 100;
int: EXECUTION_TIME = 16;
var 0..MAX_DELAY: fire_time;
var 0..MAX_DELAY: end_time;
constraint end_time = fire_time + EXECUTION_TIME;

% input and output buffers
var 0..2: IB_C_bc;
var 0..2: OB_B_bc;



% ========= main scheduling =========
int: TOKENSIZE_B_C = 16;
int: K_B_C = 1;
int: SOURCE_FIRE_TIME_B_C = 2;
int: WIRE_DELAY_B_C = 0;
int: TOTAL_DELAY_B_C = 18;
array [1..TOKENSIZE_B_C] of int: source_addr_time_B_C = [16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31];
array [1..TOKENSIZE_B_C] of int: target_addr_time_B_C = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
array [1..TOKENSIZE_B_C] of var 0..MAX_DELAY: T0_B_C;
array [1..TOKENSIZE_B_C] of var 0..MAX_DELAY: T1_B_C;
array [1..TOKENSIZE_B_C] of var 0..MAX_DELAY: T2_B_C;
array [1..TOKENSIZE_B_C] of var 0..MAX_DELAY: T3_B_C;
array [1..TOKENSIZE_B_C] of var 1..MAX_DELAY: D01_B_C;
array [1..TOKENSIZE_B_C] of var 1..MAX_DELAY: D23_B_C;

% scheduling constraints
constraint fire_time  >= SOURCE_FIRE_TIME_B_C + TOTAL_DELAY_B_C;
constraint forall(i in 1..TOKENSIZE_B_C)(
    T0_B_C[i] = SOURCE_FIRE_TIME_B_C + source_addr_time_B_C[i] /\
    T2_B_C[i] = T1_B_C[i] + WIRE_DELAY_B_C /\
    T3_B_C[i] = fire_time + target_addr_time_B_C[i] /\
    D01_B_C[i] = T1_B_C[i] - T0_B_C[i] /\
    D23_B_C[i] = T3_B_C[i] - T2_B_C[i]
);
% at any memory, buffers should not be overused
constraint cumulative(
    [T0_B_C[i] | i in 1..TOKENSIZE_B_C],
    [D01_B_C[i] | i in 1..TOKENSIZE_B_C],
    [1 | i in 1..TOKENSIZE_B_C],
    OB_B_bc,
);
constraint cumulative(
    [T2_B_C[i] | i in 1..TOKENSIZE_B_C],
    [D23_B_C[i] | i in 1..TOKENSIZE_B_C],
    [1 | i in 1..TOKENSIZE_B_C],
    IB_C_bc,
);
% channel bandwidth K
constraint cumulative(
    [T1_B_C[i] | i in 1..TOKENSIZE_B_C],
    [1 | i in 1..TOKENSIZE_B_C],
    [1 | i in 1..TOKENSIZE_B_C],
    K_B_C,
);



% ========= objective =========
var 2..32: BUFFER_SIZE;
constraint BUFFER_SIZE = sum([OB_B_bc]) + sum([IB_C_bc]);
solve minimize BUFFER_SIZE;


% ========= output =========
output ["{fire_time:\(fire_time), end_time:\(end_time), OB_B_bc:\(OB_B_bc), IB_C_bc:\(IB_C_bc), T0_B_C:\(T0_B_C), T1_B_C:\(T1_B_C), T2_B_C:\(T2_B_C), T3_B_C:\(T3_B_C)}"];
