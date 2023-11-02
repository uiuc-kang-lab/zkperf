pragma circom 2.0.0;

include "../lib/Conv2D_BatchNorm_ReLU.circom";
include "../lib/Small_Conv2D_BatchNorm_ReLU.circom";

template tiny_mobilenet() {
    signal input in[224][224][3];
    signal conv2d_1_weights[3][3][3][32];
    signal conv2d_1_bias[32];
    signal conv2d_1_bn_a[32];
    signal conv2d_1_bn_b[32];

    signal conv2d_2_dw_weights[3][3][32];
    signal conv2d_2_dw_bias[32];
    signal conv2d_2_p_weights[32][64];
    signal conv2d_2_p_bias[64];
    signal conv2d_2_dw_bn_a[32];
    signal conv2d_2_dw_bn_b[32];
    signal conv2d_2_p_bn_a[64];
    signal conv2d_2_p_bn_b[64];
    
    component conv2d_1 = Conv2D_BatchNorm_ReLU(224, 224, 3, 32, 3, 2);
    component conv2d_2 = Small_Conv2D_BatchNorm_ReLU(112, 112, 32, 64, 3, 1);

    signal output out[112][112][64];

    for (var i=0; i<224; i++) {
        for (var j=0; j<224; j++) {
            for (var k = 0; k < 3; k++) {
                conv2d_1.in[i][j][k] <== in[i][j][k];
            }
        }
    }

    for (var m=0; m<32; m++) {
        for (var i=0; i<3; i++) {
            for (var j=0; j<3; j++) {
                for (var k = 0; k < 3; k++) {
                    conv2d_1.weights[i][j][k][m] <== conv2d_1_weights[i][j][k][m];
                }
            }
        }
        conv2d_1.bias[m] <== conv2d_1_bias[m];
        conv2d_1.bn_a[m] <== conv2d_1_bn_a[m];
        conv2d_1.bn_b[m] <== conv2d_1_bn_b[m];
    }
    
    // TODO: Add Quantization
    for (var i=0; i<112; i++) {
        for (var j=0; j<112; j++) {
            for (var k = 0; k < 32; k++) {
                conv2d_2.in[i][j][k] <== conv2d_1.out[i][j][k];
            }
        }
    }

    for (var k = 0; k < 32; k++) {
        for(var i = 0; i < 3; i++) {
            for(var j = 0; j < 3; j++) {
                conv2d_2.weights_dw[i][j][k] <== conv2d_2_dw_weights[i][j][k];
            }
        }
        conv2d_2.bias_dw[k] <== conv2d_2_dw_bias[k];
        conv2d_2.bn_dw_a[k] <== conv2d_2_dw_bn_a[k];
        conv2d_2.bn_dw_b[k] <== conv2d_2_dw_bn_b[k];
    }

    for (var m = 0; m < 64; m++) {
        for (var k = 0; k < 32; k++) {
            conv2d_2.weights_p[k][m] <== conv2d_2_p_weights[k][m];
        }
        conv2d_2.bias_p[m] <== conv2d_2_p_bias[m];
        conv2d_2.bn_p_a[m] <== conv2d_2_p_bn_a[m];
        conv2d_2.bn_p_b[m] <== conv2d_2_p_bn_b[m];
    }


    for (var i = 0; i < 112; i++) {
        for (var j = 0; j < 112; j++) {
            for (var k = 0; k < 64; k++) {
                out[i][j][k] <== conv2d_2.out[i][j][k];
            }
        }
    }
}

component main = tiny_mobilenet();