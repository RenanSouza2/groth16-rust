template Circuit() {
    signal input a;
    signal input b;
    signal input c;

    c === a * b;
}

component main = Circuit();
