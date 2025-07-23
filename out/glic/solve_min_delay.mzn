int: N = 16;
int: WIDTH = 2;
int: MAX_DELAY = 58;
array [1..N] of int: src_patterns = [18, 17, 28, 25, 30, 31, 29, 26, 22, 24, 23, 19, 16, 21, 27, 20];
array [1..N] of int: dst_patterns = [2, 1, 12, 9, 14, 15, 13, 10, 6, 8, 7, 3, 0, 5, 11, 4];

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
