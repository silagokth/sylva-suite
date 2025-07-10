% minimising the total area to get estimated max width and height
int: N = 3;
int: MAX_WIDTH = 9;
int: MAX_HEIGHT = 15;
array [1..N] of int: widths = [6, 3, 6];
array [1..N] of int: heights = [6, 6, 9];

% x and y coordinates
array [1..N] of var 0..MAX_WIDTH: x;
array [1..N] of var 0..MAX_HEIGHT: y;
var 0..MAX_WIDTH: max_x_position;
var 0..MAX_HEIGHT: max_y_position;

% ========= constraints =========
% each node must be fully inside the working area
constraint forall(i in 1..N)(
  x[i] + widths[i] <= MAX_WIDTH /\
  y[i] + heights[i] <= MAX_HEIGHT
);

% No overlap between any two nodes
constraint forall(i, j in 1..N where i < j)(
  (x[i] + widths[i] <= x[j] \/ x[j] + widths[j] <= x[i]) \/
  (y[i] + heights[i] <= y[j] \/ y[j] + heights[j] <= y[i])
);

% binding the maximum positions
constraint forall(i in 1..N)(
  x[i] + widths[i] <= max_x_position /\
  y[i] + heights[i] <= max_y_position
);

% ========= additional constraints =========
% first node should be in quadrant one (bottom-left)
constraint x[1] < ((MAX_WIDTH + 1) div 2);
constraint y[1] < ((MAX_HEIGHT + 1) div 2);

% add a restriction of the ration between max_x_position and max_y_position
% the ratio should be greater than 0.5 and less than 2
constraint max_x_position * 2 >= max_y_position;
constraint max_x_position <= max_y_position * 2;

% add Manhattan constraints for each pair of nodes
int: E = 2;
int: ROUTING_SIZE = 1;
array [1..E] of int: edge_sources = [1, 2];
array [1..E] of int: edge_targets = [2, 3];
array [1..E] of int: edge_source_ports = [0, 0];
array [1..E] of int: edge_target_ports = [0, 0];
array [1..E] of int: edge_conns = [16, 16];

array [1..E] of var 0..MAX_WIDTH: source_x;
array [1..E] of var 0..MAX_HEIGHT: source_y;
array [1..E] of var 0..MAX_WIDTH: target_x;
array [1..E] of var 0..MAX_HEIGHT: target_y;
constraint forall(i in 1..E)(
  source_x[i] = x[edge_sources[i]] + edge_source_ports[i] + ROUTING_SIZE/\
  source_y[i] = y[edge_sources[i]] + heights[edge_sources[i]] - 2*ROUTING_SIZE/\
  target_x[i] = x[edge_targets[i]] + edge_target_ports[i] + ROUTING_SIZE/\
  target_y[i] = y[edge_targets[i]] + ROUTING_SIZE
);
array [1..E] of var 0..MAX_WIDTH: dx;
array [1..E] of var 0..MAX_HEIGHT: dy;
array [1..E] of var 0..(MAX_WIDTH + MAX_HEIGHT): distance_matrix;
array [1..E] of var 0..((MAX_WIDTH + MAX_HEIGHT)*16): weighted_distance;
constraint forall(i in 1..E)(
  dx[i] = abs(source_x[i] - target_x[i]) /\
  dy[i] = abs(source_y[i] - target_y[i]) /\
  distance_matrix[i] = dx[i] + dy[i] /\
  weighted_distance[i] = edge_conns[i] * distance_matrix[i]
);


% ========= objective =========
solve minimize sum(weighted_distance);


% ========= output =========
output ["{x:\(x),y:\(y),max_x_position:\(max_x_position),max_y_position:\(max_y_position),distance_matrix:\(distance_matrix)}"];
