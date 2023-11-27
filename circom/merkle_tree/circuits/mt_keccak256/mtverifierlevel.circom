pragma circom 2.0.0;

template MTVerifierLevel() {

    signal input lrbit;
    signal output root[256];
    signal input child[256];
    signal input sibling[256];

    component proofHash = SMTHash2();
    component switcher[256];

    for (var i = 0; i < 256; i++) {
        switcher[i] =  Switcher();
        switcher[i].L <== child[i];
        switcher[i].R <== sibling[i];
        switcher[i].sel <== lrbit;
    }
    for (var i = 0; i < 256; i++) {
        proofHash.L[i] <== switcher[i].outL;
        proofHash.R[i] <== switcher[i].outR;
    }
    for (var i = 0; i < 256; i++) {
        root[i] <== proofHash.out[i];
    }

}
