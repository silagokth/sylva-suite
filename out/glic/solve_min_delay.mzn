int: N = 16;
int: WIDTH = 2;
int: MAX_DELAY = 58;
array [1..N] of int: src_patterns = [21, 30, 25, 31, 23, 29, 26, 22, 24, 28, 19, 27, 16, 18, 17, 20];
array [1..N] of int: dst_patterns = [5, 14, 9, 15, 7, 13, 10, 6, 8, 12, 3, 11, 0, 2, 1, 4];

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
