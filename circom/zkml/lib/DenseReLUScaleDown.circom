pragma circom 2.0.0;

include "./circomlib-matrix/matMul.circom";
include "./ReLU.circom";
include "./ScaleDown.circom";

template DenseReLUScaleDown(nInputs,nOutputs, sd) {
    signal input in[nInputs];
    signal input weights[nInputs][nOutputs];
    signal input bias[nOutputs];
    signal output out[nOutputs];

    component dot[nOutputs];
    component relu[nOutputs];
    component scaledown[nOutputs];

    for (var i=0; i<nOutputs; i++) {
        dot[i] = matMul(1,nInputs,1);
        for (var j=0; j<nInputs; j++) {
            dot[i].a[0][j] <== in[j];
            dot[i].b[j][0] <== weights[j][i];
        }
        relu[i] = ReLU();
        relu[i].in <== dot[i].out[0][0] + bias[i];
        scaledown[i] = ScaleDown(sd);
        scaledown[i].in <== relu[i].out;
        out[i] <== scaledown[i].out;
    }
}