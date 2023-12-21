pragma circom 2.0.0;

include "../poseidon.circom";

/*
    Hash1 = H(key | value)
 */

template SMTHash1() {
    signal input key;
    signal input value;
    signal output out;

    component h = Poseidon(2);
    h.inputs[0] <== key;
    h.inputs[1] <== value;
    out <== h.out;
}

/*
    This component is used to create the 2 nodes.

    Hash2 = H(Hl | Hr)
 */

template SMTHash2() {
    signal input L;
    signal input R;
    signal output out;

    component h = Poseidon(2);
    h.inputs[0] <== L;
    h.inputs[1] <== R;
    out <== h.out;
}
