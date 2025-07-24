int: N = 16;
int: WIDTH = 2;
int: MAX_DELAY = 58;
array [1..N] of int: src_patterns = [22, 30, 23, 18, 24, 29, 17, 31, 25, 16, 28, 21, 27, 20, 26, 19];
array [1..N] of int: dst_patterns = [6, 14, 7, 2, 8, 13, 1, 15, 9, 0, 12, 5, 11, 4, 10, 3];

var 2..MAX_DELAY: delay;
array [1..N] of var 0..MAX_DELAY: tr_read;

% ========= constraints =========
constraint forall(i in 1..N)(
  tr_read[i] > src_patterns[i] /\
  dst_patterns[i] + delay > tr_read[i] /\
  dst_patterns[i] + delay < MAX_DELAY
);
 % At any moment, there can be WIDTH transfers in the communication channel
constraint forall(x in min(tr_read)..max(tr_read))(
  count(tr_read, x) <= WIDTH
);


% ========= objective =========
solve minimize delay;


% ========= output =========
output ["{delay:\(delay)}"];
