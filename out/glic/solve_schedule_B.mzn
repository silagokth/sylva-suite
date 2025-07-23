include "cumulative.mzn";

int: MAX_DELAY = 100;
int: EXECUTION_TIME = 32;
var 0..MAX_DELAY: fire_time;
var 0..MAX_DELAY: end_time;
constraint end_time = fire_time + EXECUTION_TIME;

% input and output buffers
var 0..2: IB_B_ab;
var 0..2: OB_A_ab;



% ========= main scheduling =========
int: TOKENSIZE_A_B = 16;
int: K_A_B = 1;
int: SOURCE_FIRE_TIME_A_B = 0;
int: WIRE_DELAY_A_B = 0;
int: TOTAL_DELAY_A_B = 2;
array [1..TOKENSIZE_A_B] of int: source_addr_time_A_B = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
array [1..TOKENSIZE_A_B] of int: target_addr_time_A_B = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
array [1..TOKENSIZE_A_B] of var 0..MAX_DELAY: T0_A_B;
array [1..TOKENSIZE_A_B] of var 0..MAX_DELAY: T1_A_B;
array [1..TOKENSIZE_A_B] of var 0..MAX_DELAY: T2_A_B;
array [1..TOKENSIZE_A_B] of var 0..MAX_DELAY: T3_A_B;
array [1..TOKENSIZE_A_B] of var 1..MAX_DELAY: D01_A_B;
array [1..TOKENSIZE_A_B] of var 1..MAX_DELAY: D23_A_B;

% scheduling constraints
constraint fire_time  >= SOURCE_FIRE_TIME_A_B + TOTAL_DELAY_A_B;
constraint forall(i in 1..TOKENSIZE_A_B)(
    T0_A_B[i] = SOURCE_FIRE_TIME_A_B + source_addr_time_A_B[i] /\
    T2_A_B[i] = T1_A_B[i] + WIRE_DELAY_A_B /\
    T3_A_B[i] = fire_time + target_addr_time_A_B[i] /\
    D01_A_B[i] = T1_A_B[i] - T0_A_B[i] /\
    D23_A_B[i] = T3_A_B[i] - T2_A_B[i]
);
% at any memory, buffers should not be overused
constraint cumulative(
    [T0_A_B[i] | i in 1..TOKENSIZE_A_B],
    [D01_A_B[i] | i in 1..TOKENSIZE_A_B],
    [1 | i in 1..TOKENSIZE_A_B],
    OB_A_ab,
);
constraint cumulative(
    [T2_A_B[i] | i in 1..TOKENSIZE_A_B],
    [D23_A_B[i] | i in 1..TOKENSIZE_A_B],
    [1 | i in 1..TOKENSIZE_A_B],
    IB_B_ab,
);
% channel bandwidth K
constraint cumulative(
    [T1_A_B[i] | i in 1..TOKENSIZE_A_B],
    [1 | i in 1..TOKENSIZE_A_B],
    [1 | i in 1..TOKENSIZE_A_B],
    K_A_B,
);



% ========= objective =========
var 2..32: BUFFER_SIZE;
constraint BUFFER_SIZE = sum([OB_A_ab]) + sum([IB_B_ab]);
solve minimize BUFFER_SIZE;


% ========= output =========
output ["{fire_time:\(fire_time), end_time:\(end_time), OB_A_ab:\(OB_A_ab), IB_B_ab:\(IB_B_ab), T0_A_B:\(T0_A_B), T1_A_B:\(T1_A_B), T2_A_B:\(T2_A_B), T3_A_B:\(T3_A_B)}"];
