pragma circom 2.0.0;

include "mthash_poseidon.circom";

template MTVerifierLevel() {

    signal input lrbit;
    signal output root;
    signal input child;
    signal input sibling;

    component proofHash = SMTHash2();
    component switcher = Switcher();

    switcher.L <== child;
    switcher.R <== sibling;
    switcher.sel <== lrbit;

    proofHash.L <== switcher.outL;
    proofHash.R <== switcher.outR;
    root <== proofHash.out;
}
