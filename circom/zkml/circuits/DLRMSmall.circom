pragma circom 2.0.0;

include "../lib/DenseScaleDown.circom";
include "../lib/DenseReLUScaleDown.circom";
include "../lib/Dense.circom";
include "./DLRMDot.circom";


template DLRMSmall(sd) {
    signal input dense_x[13];
    signal input embedding[26][64];
    signal output out;

    signal input bot_l_0_weight[512][13];
    signal input bot_l_0_bias[512];
    signal input bot_l_2_weight[256][512];
    signal input bot_l_2_bias[256];
    signal input bot_l_4_weight[64][256];
    signal input bot_l_4_bias[64];

    signal input top_l_0_weight[512][415];
    signal input top_l_0_bias[512];
    signal input top_l_2_weight[512][512];
    signal input top_l_2_bias[512];
    signal input top_l_4_weight[256][512];
    signal input top_l_4_bias[256];
    signal input top_l_6_weight[1][256];
    signal input top_l_6_bias[1];

    component bot_l_0 = DenseReLUScaleDown(13, 512, sd);
    component bot_l_2 = DenseReLUScaleDown(512, 256, sd);
    component bot_l_4 = DenseReLUScaleDown(256, 64, sd);
    component dot = DLRMDot(64, 26, sd);
    component top_l_0 = DenseReLUScaleDown(415, 512, sd);
    component top_l_2 = DenseReLUScaleDown(512, 512, sd);
    component top_l_4 = DenseReLUScaleDown(512, 256, sd);
    component top_l_6 = DenseScaleDown(256, 1, sd);
    component top_l_6 = Dense(256, 1);

    // Bot Layer
    for(var i = 0; i < 13; i++) {
        for (var j = 0; j < 512; j++) {
            bot_l_0.weights[i][j] <== bot_l_0_weight[j][i];
        }
        bot_l_0.in[i] <== dense_x[i];
    }
    for (var i = 0; i < 512; i++) {
        bot_l_0.bias[i] <== bot_l_0_bias[i];
    }
    for (var i = 0; i < 512; i++) {
        log(bot_l_0.out[i]);
    }

    for(var i = 0; i < 512; i++) {
        for (var j = 0; j < 256; j++) {
            bot_l_2.weights[i][j] <== bot_l_2_weight[j][i];
        }
        bot_l_2.in[i] <== bot_l_0.out[i];
    }
    for (var i = 0; i < 256; i++) {
        bot_l_2.bias[i] <== bot_l_2_bias[i];
    }
    for(var i = 0; i < 256; i++) {
        for (var j = 0; j < 64; j++) {
            bot_l_4.weights[i][j] <== bot_l_4_weight[j][i];
        }
        bot_l_4.in[i] <== bot_l_2.out[i];
    }
    for (var i = 0; i < 64; i++) {
        bot_l_4.bias[i] <== bot_l_4_bias[i];
    }

    // Dot Operation
    for (var i = 0; i < 64; i++) {
        dot.mix[0][i] <== bot_l_4.out[i];
    }
    for(var i = 1; i < 27; i++) {
        for (var j = 0; j < 64; j++) {
            dot.mix[i][j] <== embedding[i-1][j];
        }
    }

    // Top Layer
    for(var i = 0; i < 415; i++) {
        for (var j = 0; j < 512; j++) {
            top_l_0.weights[i][j] <== top_l_0_weight[j][i];
        }
        top_l_0.in[i] <== dot.out[i];
    }
    for (var i = 0; i < 512; i++) {
        top_l_0.bias[i] <== top_l_0_bias[i];
    }

    for(var i = 0; i < 512; i++) {
        for (var j = 0; j < 512; j++) {
            top_l_2.weights[i][j] <== top_l_2_weight[j][i];
        }
        top_l_2.in[i] <== top_l_0.out[i];
    }
    for (var i = 0; i < 512; i++) {
        top_l_2.bias[i] <== top_l_2_bias[i];
    }
    for(var i = 0; i < 512; i++) {
        for (var j = 0; j < 256; j++) {
            top_l_4.weights[i][j] <== top_l_4_weight[j][i];
        }
        top_l_4.in[i] <== top_l_2.out[i];
    }
    for (var i = 0; i < 256; i++) {
        top_l_4.bias[i] <== top_l_4_bias[i];
    }

    for(var i = 0; i < 256; i++) {
        for (var j = 0; j < 1; j++) {
            top_l_6.weights[i][j] <== top_l_6_weight[j][i];
        }
        top_l_6.in[i] <== top_l_4.out[i];
    }
    top_l_6.bias[0] <== top_l_6_bias[0];
    out <== top_l_6.out[0];
    log("The output is: ", out);
}

component main = DLRMSmall(10);