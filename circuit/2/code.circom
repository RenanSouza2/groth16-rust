template Circuit() {
    signal input a;
    signal input b;
    
    signal c;
    signal d;

    c <== a * b;
    d <== (a + b) * a;
}

component main = Circuit();
