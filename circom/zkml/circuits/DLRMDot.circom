pragma circom 2.0.0;

include "../lib/Dense.circom";
include "../lib/ScaleDown.circom";
include "../lib/circomlib-matrix/matMul.circom";

template DLRMDot(M, L, sd) {
    // first row input should be dense_x
    /*
        [dense_x]
        [embedding]
    */
    signal input mix[L+1][M];
    signal output out[(L/2) * (L+1) + M];
    
    component z = matMul(L+1, M, L+1);
    component scaledown[(L+1)*(L+1)];

    for (var i = 0; i < M; i++) {
        z.a[0][i] <== mix[0][i];
        z.b[i][0] <== mix[0][i];
        out[i] <== mix[0][i];
    }
    for (var i = 1; i < L+1; i++) {
        for (var j = 0; j < M; j++) {
            z.a[i][j] <== mix[i][j];
            z.b[j][i] <== mix[i][j];
        }
    }

    var counter = 0;
    for (var i = 1; i < L+1; i++) {
        for (var j = 0; j < i; j++) {
            scaledown[counter] = ScaleDown(sd);
            scaledown[counter].in <== z.out[i][j];
            out[M + counter] <== scaledown[counter].out;
            counter += 1;
        }
    }

}