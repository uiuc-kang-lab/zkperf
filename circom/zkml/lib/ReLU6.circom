pragma circom 2.0.0;

include "./util.circom";
include "./circomlib/comparators.circom";

// ReLU layer
template ReLU6 (sd) {
    signal input in;
    signal output out;
    signal aux;

    component isPositive = IsPositive();
    component lessThan = LessThanConst(252, 6<<sd);

    var constant = 6<<sd;

    isPositive.in <== in;
    lessThan.in <== in;
    
    aux <== (in - constant) * lessThan.out;
    out <== (aux + constant) * isPositive.out;
}

