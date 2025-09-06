
include "sha256/sha256.circom";
include "bitify.circom";

template Circuit() {
    signal input in;
    signal out[256];
    
    component n2b = Num2Bits(128);
    n2b.in <== in;
    
    component h = Sha256(128);
    for (var i = 0; i < 128; i++) {
        h.in[i] <== n2b.out[i];
    }

    for (var i = 0; i < 256; i++) {
        out[i] <== h.out[i];
    }
}

component main = Circuit();
