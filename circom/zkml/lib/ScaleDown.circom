pragma circom 2.0.0;

include "./circomlib/bitify.circom";

template ScaleDown(shift) {
    signal input in;
    signal output out;

    component numCheck1 = Num2Bits(shift);
    component numCheck2 = Num2Bits(254-shift);

    signal rem <-- in % (1 << shift);
    out <-- in >> shift;
    numCheck1.in <== rem;
    numCheck2.in <== out;

    in === out * (1 << shift) + rem;

}