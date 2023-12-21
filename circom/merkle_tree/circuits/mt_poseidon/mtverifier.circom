pragma circom 2.0.0;


include "../bitify.circom";
include "../comparators.circom";
include "../switcher.circom";
include "mtverifierlevel.circom";
include "mthash_poseidon.circom";

template MTVerifier(nLevels) {
    signal input root;
    signal input siblings[nLevels-1];
    signal input child;
    signal input index;
    signal output out;

    component bits = Num2Bits(nLevels-1);
    bits.in <== index;

    component levels[nLevels-1];
    for (var i = 0; i < nLevels-1; i++) {
        levels[i] = MTVerifierLevel();

        levels[i].sibling <== siblings[i];
        levels[i].lrbit <== bits.out[i];
        if (i == 0) {
            levels[i].child <== child;
        } else {
            levels[i].child <== levels[i-1].root;
        }
    }
    levels[nLevels-2].root === root;
}

component main = MTVerifier(11);
