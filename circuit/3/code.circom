include "node_modules/circomlib/circuits/comparators.circom";

template Circuit() {
  signal input a;
  signal input b;
  signal input out;

  component lt = LessThan(252);
  lt.in[0] <== a;
  lt.in[1] <== b;
  lt.out === out;
}

component main = Circuit();
