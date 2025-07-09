array[1..3] of var int: area_nodes;
array[1..3] of var int: energy_nodes;

var 1..2: selected_alimp_A;
array [1..2] of int: width_A = [2, 4];
array [1..2] of int: height_A = [2, 1];
array [1..2] of int: energy_A = [3, 2];
constraint width_A[selected_alimp_A] <= 100;
constraint height_A[selected_alimp_A] <= 100;
constraint width_A[selected_alimp_A] * height_A[selected_alimp_A] == area_nodes[1];
constraint energy_A[selected_alimp_A] == energy_nodes[1];

var 1..1: selected_alimp_B;
array [1..1] of int: width_B = [1];
array [1..1] of int: height_B = [1];
array [1..1] of int: energy_B = [1];
constraint width_B[selected_alimp_B] <= 100;
constraint height_B[selected_alimp_B] <= 100;
constraint width_B[selected_alimp_B] * height_B[selected_alimp_B] == area_nodes[2];
constraint energy_B[selected_alimp_B] == energy_nodes[2];

var 1..2: selected_alimp_C;
array [1..2] of int: width_C = [4, 5];
array [1..2] of int: height_C = [4, 3];
array [1..2] of int: energy_C = [10, 8];
constraint width_C[selected_alimp_C] <= 100;
constraint height_C[selected_alimp_C] <= 100;
constraint width_C[selected_alimp_C] * height_C[selected_alimp_C] == area_nodes[3];
constraint energy_C[selected_alimp_C] == energy_nodes[3];

% add global geometric and energy constraints
constraint sum(area_nodes) <= 10000;
constraint sum(energy_nodes) <= 100;


% add latency constraints
array[1..3] of var 0..100: latency_nodes;
array[1..3] of var 0..100: start_time_nodes;
array[1..3] of var 0..100: end_time_nodes;

array [1..2] of int: latency_A = [16, 16];
constraint latency_A[selected_alimp_A] == latency_nodes[1];
constraint start_time_nodes[1] == 0;
constraint end_time_nodes[1] <= 100;
constraint end_time_nodes[1] == start_time_nodes[1] + latency_nodes[1];
constraint latency_nodes[1] <= 70; % throughput contraint

array [1..1] of int: latency_B = [32];
constraint latency_B[selected_alimp_B] == latency_nodes[2];
constraint start_time_nodes[2] >= start_time_nodes[1] + (latency_nodes[1] div 2);
constraint end_time_nodes[2] <= 100;
constraint end_time_nodes[2] == start_time_nodes[2] + latency_nodes[2];
constraint latency_nodes[2] <= 70; % throughput contraint

array [1..2] of int: latency_C = [16, 16];
constraint latency_C[selected_alimp_C] == latency_nodes[3];
constraint start_time_nodes[3] >= start_time_nodes[2] + (latency_nodes[2] div 2);
constraint end_time_nodes[3] <= 100;
constraint end_time_nodes[3] == start_time_nodes[3] + latency_nodes[3];
constraint latency_nodes[3] <= 70; % throughput contraint



% add objective and model problem
var 0..1000000: objective;
constraint objective == (1 * sum(area_nodes)) + (1 * sum(energy_nodes));
solve minimize objective;
output ["{objective:\(objective)}"];
