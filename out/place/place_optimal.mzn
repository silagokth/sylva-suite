% minimising the total area to get estimated max width and height
int: N = 3;
int: MAX_WIDTH = 99;
int: MAX_HEIGHT = 99;
array [1..N] of int: widths = [3, 6, 6];
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

% No overlap between any two nodes in both x and y directions
constraint forall(i, j in 1..N where i < j)(
  (x[i] + widths[i] <= x[j] \/ x[j] + widths[j] <= x[i]) /\
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


% ========= objective =========
solve minimize (max_x_position * max_y_position);


% ========= output =========
output ["{max_x_position:\(max_x_position), max_y_position:\(max_y_position)}"];
