int: N = 3;
int: MAX_DELAY = 100;
array [1..N] of int: EXECUTION_TIME = [16, 32, 16];
array [1..N] of var 0..MAX_DELAY: fire_time;
array [1..N] of var 0..MAX_DELAY: end_time;

% ========= constraints for scheduling =========
% set end time 
constraint forall(i in 1..N)(
  end_time[i] = fire_time[i] + EXECUTION_TIME[i]
);
constraint 0 = min([fire_time[i] | i in 1..N]); % initial fire time is 0


% add variables and constraints for edge A_B
array [1..1] of int: K_OF_A_B = [1];
array [1..1] of int: MIN_DELAY_OF_A_B = [2];
var 1..1: selected_edge_A_B;
var int: k_A_B;
var int: min_delay_A_B;
constraint k_A_B = K_OF_A_B[selected_edge_A_B];
constraint min_delay_A_B = MIN_DELAY_OF_A_B[selected_edge_A_B];
constraint fire_time[1] + min_delay_A_B < fire_time[2];


% add variables and constraints for edge B_C
array [1..1] of int: K_OF_B_C = [1];
array [1..1] of int: MIN_DELAY_OF_B_C = [18];
var 1..1: selected_edge_B_C;
var int: k_B_C;
var int: min_delay_B_C;
constraint k_B_C = K_OF_B_C[selected_edge_B_C];
constraint min_delay_B_C = MIN_DELAY_OF_B_C[selected_edge_B_C];
constraint fire_time[2] + min_delay_B_C < fire_time[3];


% ========= objective =========
var int: sum_of_k;
var int: sum_of_min_delay;
constraint sum_of_k = sum([k_A_B, k_B_C]);
constraint sum_of_min_delay = sum([min_delay_A_B, min_delay_B_C]);
solve minimize (sum_of_min_delay + 2 * sum_of_k);


% ========= output =========
output ["{end_time:\(end_time), k_A_B:\(k_A_B), min_delay_A_B:\(min_delay_A_B), k_B_C:\(k_B_C), min_delay_B_C:\(min_delay_B_C)}"];
