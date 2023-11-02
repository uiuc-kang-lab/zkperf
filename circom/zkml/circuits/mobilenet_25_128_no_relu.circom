pragma circom 2.0.0;

include "../lib/Conv2D_DW.circom";
include "../lib/MN_Conv2D.circom";
include "../lib/AveragePooling2D.circom";

template mobilenet_v1_25_128_no_relu() {
    signal input in[128][128][3];
    signal input conv2d_0_weights[16][3][3][3];
    signal input conv2d_0_bias[16];

    signal input conv2d_dw_1_weights[3][3][16];
    signal input conv2d_dw_1_bias[16];

    // signal input conv2d_2_weights[32][1][1][16];
    // signal input conv2d_2_bias[32];

    // signal input conv2d_dw_3_weights[3][3][32];
    // signal input conv2d_dw_3_bias[32];

    // signal input conv2d_4_weights[32][1][1][16];
    // signal input conv2d_4_bias[32];

    // signal input conv2d_dw_5_weights[3][3][64];
    // signal input conv2d_dw_5_bias[64];

    // signal input conv2d_6_weights[64][1][1][64];
    // signal input conv2d_6_bias[64];

    // signal input conv2d_dw_7_weights[3][3][64];
    // signal input conv2d_dw_7_bias[64];

    // signal input conv2d_8_weights[128][1][1][64];
    // signal input conv2d_8_bias[128];

    // signal input conv2d_dw_9_weights[3][3][128];
    // signal input conv2d_dw_9_bias[128];

    // signal input conv2d_10_weights[128][1][1][128];
    // signal input conv2d_10_bias[128];

    // signal input conv2d_dw_11_weights[3][3][128];
    // signal input conv2d_dw_11_bias[128];

    // signal input conv2d_12_weights[256][1][1][128];
    // signal input conv2d_12_bias[256];

    // signal input conv2d_dw_13_weights[3][3][256];
    // signal input conv2d_dw_13_bias[256];

    // signal input conv2d_14_weights[256][1][1][256];
    // signal input conv2d_14_bias[256];

    // signal input conv2d_dw_15_weights[3][3][256];
    // signal input conv2d_dw_15_bias[256];

    // signal input conv2d_16_weights[256][1][1][256];
    // signal input conv2d_16_bias[256];

    // signal input conv2d_dw_17_weights[3][3][256];
    // signal input conv2d_dw_17_bias[256];

    // signal input conv2d_18_weights[256][1][1][256];
    // signal input conv2d_18_bias[256];

    // signal input conv2d_dw_19_weights[3][3][256];
    // signal input conv2d_dw_19_bias[256];

    // signal input conv2d_20_weights[256][1][1][256];
    // signal input conv2d_20_bias[256];

    // signal input conv2d_dw_21_weights[3][3][256];
    // signal input conv2d_dw_21_bias[256];

    // signal input conv2d_22_weights[256][1][1][256];
    // signal input conv2d_22_bias[256];

    // signal input conv2d_dw_23_weights[3][3][256];
    // signal input conv2d_dw_23_bias[256];

    // signal input conv2d_24_weights[512][1][1][256];
    // signal input conv2d_24_bias[512];

    // signal input conv2d_dw_25_weights[3][3][512];
    // signal input conv2d_dw_25_bias[512];

    // signal input conv2d_26_weights[512][1][1][512];
    // signal input conv2d_26_bias[512];

    // signal input conv2d_28_weights[1001][1][1][512];
    // signal input conv2d_28_bias[1001];

    // signal output out[1001];
    
    component conv2d_0 = MN_Conv2D(128, 128, 3, 16, 3, 2);
    component conv2d_dw_1 = Conv2D_DW(64, 64, 16, 3, 1);
    // component conv2d_2 = MN_Conv2D_ReLU(64, 64, 16, 32, 1, 1);
    // component conv2d_dw_3 = Conv2D_DW_ReLU(64, 64, 32, 3, 2);
    // component conv2d_4 = MN_Conv2D_ReLU(32, 32, 32, 64, 1, 1);
    // component conv2d_dw_5 = Conv2D_DW_ReLU(32, 32, 64, 3, 1);
    // component conv2d_6 = MN_Conv2D_ReLU(32, 32, 64, 64, 1, 1);
    // component conv2d_dw_7 = Conv2D_DW_ReLU(32, 32, 64, 3, 2);
    // component conv2d_8 = MN_Conv2D_ReLU(16, 16, 64, 128, 1, 1);
    // component conv2d_dw_9 = Conv2D_DW_ReLU(16, 16, 128, 3, 1);
    // component conv2d_10 = MN_Conv2D_ReLU(16, 16, 128, 128, 1, 1);
    // component conv2d_dw_11 = Conv2D_DW_ReLU(16, 16, 128, 3, 2);
    // component conv2d_12 = MN_Conv2D_ReLU(8, 8, 128, 256, 1, 1);
    // component conv2d_dw_13 = Conv2D_DW_ReLU(8, 8, 256, 3, 1);
    // component conv2d_14 = MN_Conv2D_ReLU(8, 8, 256, 256, 1, 1);
    // component conv2d_dw_15 = Conv2D_DW_ReLU(8, 8, 256, 3, 1);
    // component conv2d_16 = MN_Conv2D_ReLU(8, 8, 256, 256, 1, 1);
    // component conv2d_dw_17 = Conv2D_DW_ReLU(8, 8, 256, 3, 1);
    // component conv2d_18 = MN_Conv2D_ReLU(8, 8, 256, 256, 1, 1);
    // component conv2d_dw_19 = Conv2D_DW_ReLU(8, 8, 256, 3, 1);
    // component conv2d_20 = MN_Conv2D_ReLU(8, 8, 256, 256, 1, 1);
    // component conv2d_dw_21 = Conv2D_DW_ReLU(8, 8, 256, 3, 1);
    // component conv2d_22 = MN_Conv2D_ReLU(8, 8, 256, 256, 1, 1);
    // component conv2d_dw_23 = Conv2D_DW_ReLU(8, 8, 256, 3, 2);
    // component conv2d_24 = MN_Conv2D_ReLU(4, 4, 256, 512, 1, 1);
    // component conv2d_dw_25 = Conv2D_DW_ReLU(4, 4, 512, 3, 1);
    // component conv2d_26 = MN_Conv2D_ReLU(4, 4, 512, 512, 1, 1);
    //TODO: average pool need scale configuration
    // component avg_pool = AveragePooling2D(4, 4, 512, 4, 2, 25);
    // component conv2d_28 = MN_Conv2D_ReLU(1, 1, 512, 1001, 1, 1);

    // conv2d_0
    for (var i=1; i<129; i++) {
        for (var j=1; j<129; j++) {
            for (var k=0; k < 3; k++) {
                conv2d_0.in[i][j][k] <== in[i-1][j-1][k];
            }
        }
    }
    for (var i = 1; i < 129; i++) {
        for (var k=0; k < 3; k++) {
            conv2d_0.in[0][i][k] <== in[0][i-1][k];
            conv2d_0.in[129][i][k] <== in[127][i-1][k];
            conv2d_0.in[i][0][k] <== in[i-1][0][k];
            conv2d_0.in[i][129][k] <== in[i-1][127][k];
        }
    }
    for (var k=0; k < 3; k++) {
        conv2d_0.in[0][0][k] <== in[0][0][k];
        conv2d_0.in[129][0][k] <== in[127][0][k];
        conv2d_0.in[0][129][k] <== in[0][127][k];
        conv2d_0.in[129][129][k] <== in[127][127][k];
    }

    for (var m=0; m<16; m++) {
        for (var i=0; i<3; i++) {
            for (var j=0; j<3; j++) {
                for (var k=0; k<3; k++) {
                    conv2d_0.weights[m][i][j][k] <== conv2d_0_weights[m][i][j][k];
                }
            }
        }
        conv2d_0.bias[m] <== conv2d_0_bias[m];
    }

    // conv2d_dw_1
    for (var i=1; i<65; i++) {
        for (var j=1; j<65; j++) {
            for (var k=0; k < 16; k++) {
                conv2d_dw_1.in[i][j][k] <== conv2d_0.out[i-1][j-1][k];
            }
        }
    }
    for (var i = 1; i < 65; i++) {
        for (var k=0; k < 16; k++) {
            conv2d_dw_1.in[0][i][k] <== conv2d_0.out[0][i-1][k];
            conv2d_dw_1.in[65][i][k] <== conv2d_0.out[63][i-1][k];
            conv2d_dw_1.in[i][0][k] <== conv2d_0.out[i-1][0][k];
            conv2d_dw_1.in[i][65][k] <== conv2d_0.out[i-1][63][k];
        }
    }
    for (var k=0; k < 16; k++) {
        conv2d_dw_1.in[0][0][k] <== conv2d_0.out[0][0][k];
        conv2d_dw_1.in[65][0][k] <== conv2d_0.out[63][0][k];
        conv2d_dw_1.in[0][65][k] <== conv2d_0.out[0][63][k];
        conv2d_dw_1.in[65][65][k] <== conv2d_0.out[63][63][k];
    }


    for (var k=0; k<16; k++) {
        for (var i=0; i<3; i++) {
            for (var j=0; j<3; j++) {
                conv2d_dw_1.weights[i][j][k] <== conv2d_dw_1_weights[i][j][k];
            }
        }
        conv2d_dw_1.bias[k] <== conv2d_dw_1_bias[k];
    }

    // // conv2d_2
    // for (var i=0; i<64; i++) {
    //     for (var j=0; j<64; j++) {
    //         for (var k=0; k < 16; k++) {
    //             conv2d_2.in[i][j][k] <== conv2d_dw_1.out[i][j][k];
    //         }
    //     }
    // }
    // for (var m=0; m<32; m++) {
    //     for (var i=0; i<1; i++) {
    //         for (var j=0; j<1; j++) {
    //             for (var k=0; k<16; k++) {
    //                 conv2d_2.weights[m][i][j][k] <== conv2d_2_weights[m][i][j][k];
    //             }
    //         }
    //     }
    //     conv2d_2.bias[m] <== conv2d_2_bias[m];
    // }

    // // conv2d_dw_3
    // for (var i=0; i<64; i++) {
    //     for (var j=0; j<64; j++) {
    //         for (var k=0; k < 32; k++) {
    //             conv2d_dw_3.in[i][j][k] <== conv2d_2.out[i][j][k];
    //         }
    //     }
    // }
    // for (var k=32; k<32; k++) {
    //     for (var i=0; i<3; i++) {
    //         for (var j=0; j<3; j++) {
    //             conv2d_dw_3.weights[i][j][k] <== conv2d_dw_3_weights[i][j][k];
    //         }
    //     }
    //     conv2d_dw_3.bias[k] <== conv2d_dw_3_bias[k];
    // }

    // // conv2d_4
    // for (var i=0; i<32; i++) {
    //     for (var j=0; j<32; j++) {
    //         for (var k=0; k < 32; k++) {
    //             conv2d_4.in[i][j][k] <== conv2d_dw_3.out[i][j][k];
    //         }
    //     }
    // }
    // for (var m=0; m<64; m++) {
    //     for (var i=0; i<1; i++) {
    //         for (var j=0; j<1; j++) {
    //             for (var k=0; k<32; k++) {
    //                 conv2d_4.weights[m][i][j][k] <== conv2d_4_weights[m][i][j][k];
    //             }
    //         }
    //     }
    //     conv2d_4.bias[m] <== conv2d_4_bias[m];
    // }

    // // conv2d_dw_5
    // for (var i=0; i<32; i++) {
    //     for (var j=0; j<32; j++) {
    //         for (var k=0; k < 64; k++) {
    //             conv2d_dw_5.in[i][j][k] <== conv2d_4.out[i][j][k];
    //         }
    //     }
    // }
    // for (var k=64; k<64; k++) {
    //     for (var i=0; i<3; i++) {
    //         for (var j=0; j<3; j++) {
    //             conv2d_dw_5.weights[i][j][k] <== conv2d_dw_5_weights[i][j][k];
    //         }
    //     }
    //     conv2d_dw_5.bias[k] <== conv2d_dw_5_bias[k];
    // }

    // // conv2d_6
    // for (var i=0; i<32; i++) {
    //     for (var j=0; j<32; j++) {
    //         for (var k=0; k < 64; k++) {
    //             conv2d_6.in[i][j][k] <== conv2d_dw_5.out[i][j][k];
    //         }
    //     }
    // }
    // for (var m=0; m<64; m++) {
    //     for (var i=0; i<1; i++) {
    //         for (var j=0; j<1; j++) {
    //             for (var k=0; k<64; k++) {
    //                 conv2d_6.weights[m][i][j][k] <== conv2d_6_weights[m][i][j][k];
    //             }
    //         }
    //     }
    //     conv2d_6.bias[m] <== conv2d_6_bias[m];
    // }

    // // conv2d_dw_7
    // for (var i=0; i<32; i++) {
    //     for (var j=0; j<32; j++) {
    //         for (var k=0; k < 64; k++) {
    //             conv2d_dw_7.in[i][j][k] <== conv2d_6.out[i][j][k];
    //         }
    //     }
    // }
    // for (var k=64; k<64; k++) {
    //     for (var i=0; i<3; i++) {
    //         for (var j=0; j<3; j++) {
    //             conv2d_dw_7.weights[i][j][k] <== conv2d_dw_7_weights[i][j][k];
    //         }
    //     }
    //     conv2d_dw_7.bias[k] <== conv2d_dw_7_bias[k];
    // }

    // // conv2d_8
    // for (var i=0; i<16; i++) {
    //     for (var j=0; j<16; j++) {
    //         for (var k=0; k < 64; k++) {
    //             conv2d_8.in[i][j][k] <== conv2d_dw_7.out[i][j][k];
    //         }
    //     }
    // }
    // for (var m=0; m<128; m++) {
    //     for (var i=0; i<1; i++) {
    //         for (var j=0; j<1; j++) {
    //             for (var k=0; k<64; k++) {
    //                 conv2d_8.weights[m][i][j][k] <== conv2d_8_weights[m][i][j][k];
    //             }
    //         }
    //     }
    //     conv2d_8.bias[m] <== conv2d_8_bias[m];
    // }

    // // conv2d_dw_9
    // for (var i=0; i<16; i++) {
    //     for (var j=0; j<16; j++) {
    //         for (var k=0; k < 128; k++) {
    //             conv2d_dw_9.in[i][j][k] <== conv2d_8.out[i][j][k];
    //         }
    //     }
    // }
    // for (var k=128; k<128; k++) {
    //     for (var i=0; i<3; i++) {
    //         for (var j=0; j<3; j++) {
    //             conv2d_dw_9.weights[i][j][k] <== conv2d_dw_9_weights[i][j][k];
    //         }
    //     }
    //     conv2d_dw_9.bias[k] <== conv2d_dw_9_bias[k];
    // }

    // // conv2d_10
    // for (var i=0; i<16; i++) {
    //     for (var j=0; j<16; j++) {
    //         for (var k=0; k < 128; k++) {
    //             conv2d_10.in[i][j][k] <== conv2d_dw_9.out[i][j][k];
    //         }
    //     }
    // }
    // for (var m=0; m<128; m++) {
    //     for (var i=0; i<1; i++) {
    //         for (var j=0; j<1; j++) {
    //             for (var k=0; k<128; k++) {
    //                 conv2d_10.weights[m][i][j][k] <== conv2d_10_weights[m][i][j][k];
    //             }
    //         }
    //     }
    //     conv2d_10.bias[m] <== conv2d_10_bias[m];
    // }

    // // conv2d_dw_11
    // for (var i=0; i<16; i++) {
    //     for (var j=0; j<16; j++) {
    //         for (var k=0; k < 128; k++) {
    //             conv2d_dw_11.in[i][j][k] <== conv2d_10.out[i][j][k];
    //         }
    //     }
    // }
    // for (var k=128; k<128; k++) {
    //     for (var i=0; i<3; i++) {
    //         for (var j=0; j<3; j++) {
    //             conv2d_dw_11.weights[i][j][k] <== conv2d_dw_11_weights[i][j][k];
    //         }
    //     }
    //     conv2d_dw_11.bias[k] <== conv2d_dw_11_bias[k];
    // }

    // // conv2d_12
    // for (var i=0; i<8; i++) {
    //     for (var j=0; j<8; j++) {
    //         for (var k=0; k < 128; k++) {
    //             conv2d_12.in[i][j][k] <== conv2d_dw_11.out[i][j][k];
    //         }
    //     }
    // }
    // for (var m=0; m<256; m++) {
    //     for (var i=0; i<1; i++) {
    //         for (var j=0; j<1; j++) {
    //             for (var k=0; k<128; k++) {
    //                 conv2d_12.weights[m][i][j][k] <== conv2d_12_weights[m][i][j][k];
    //             }
    //         }
    //     }
    //     conv2d_12.bias[m] <== conv2d_12_bias[m];
    // }

    // // conv2d_dw_13
    // for (var i=0; i<8; i++) {
    //     for (var j=0; j<8; j++) {
    //         for (var k=0; k < 256; k++) {
    //             conv2d_dw_13.in[i][j][k] <== conv2d_12.out[i][j][k];
    //         }
    //     }
    // }
    // for (var k=256; k<256; k++) {
    //     for (var i=0; i<3; i++) {
    //         for (var j=0; j<3; j++) {
    //             conv2d_dw_13.weights[i][j][k] <== conv2d_dw_13_weights[i][j][k];
    //         }
    //     }
    //     conv2d_dw_13.bias[k] <== conv2d_dw_13_bias[k];
    // }

    // // conv2d_14
    // for (var i=0; i<8; i++) {
    //     for (var j=0; j<8; j++) {
    //         for (var k=0; k < 256; k++) {
    //             conv2d_14.in[i][j][k] <== conv2d_dw_13.out[i][j][k];
    //         }
    //     }
    // }
    // for (var m=0; m<256; m++) {
    //     for (var i=0; i<1; i++) {
    //         for (var j=0; j<1; j++) {
    //             for (var k=0; k<256; k++) {
    //                 conv2d_14.weights[m][i][j][k] <== conv2d_14_weights[m][i][j][k];
    //             }
    //         }
    //     }
    //     conv2d_14.bias[m] <== conv2d_14_bias[m];
    // }

    // // conv2d_dw_15
    // for (var i=0; i<8; i++) {
    //     for (var j=0; j<8; j++) {
    //         for (var k=0; k < 256; k++) {
    //             conv2d_dw_15.in[i][j][k] <== conv2d_14.out[i][j][k];
    //         }
    //     }
    // }
    // for (var k=256; k<256; k++) {
    //     for (var i=0; i<3; i++) {
    //         for (var j=0; j<3; j++) {
    //             conv2d_dw_15.weights[i][j][k] <== conv2d_dw_15_weights[i][j][k];
    //         }
    //     }
    //     conv2d_dw_15.bias[k] <== conv2d_dw_15_bias[k];
    // }

    // // conv2d_16
    // for (var i=0; i<8; i++) {
    //     for (var j=0; j<8; j++) {
    //         for (var k=0; k < 256; k++) {
    //             conv2d_16.in[i][j][k] <== conv2d_dw_15.out[i][j][k];
    //         }
    //     }
    // }
    // for (var m=0; m<256; m++) {
    //     for (var i=0; i<1; i++) {
    //         for (var j=0; j<1; j++) {
    //             for (var k=0; k<256; k++) {
    //                 conv2d_16.weights[m][i][j][k] <== conv2d_16_weights[m][i][j][k];
    //             }
    //         }
    //     }
    //     conv2d_16.bias[m] <== conv2d_16_bias[m];
    // }

    // // conv2d_dw_17
    // for (var i=0; i<8; i++) {
    //     for (var j=0; j<8; j++) {
    //         for (var k=0; k < 256; k++) {
    //             conv2d_dw_17.in[i][j][k] <== conv2d_16.out[i][j][k];
    //         }
    //     }
    // }
    // for (var k=256; k<256; k++) {
    //     for (var i=0; i<3; i++) {
    //         for (var j=0; j<3; j++) {
    //             conv2d_dw_17.weights[i][j][k] <== conv2d_dw_17_weights[i][j][k];
    //         }
    //     }
    //     conv2d_dw_17.bias[k] <== conv2d_dw_17_bias[k];
    // }

    // // conv2d_18
    // for (var i=0; i<8; i++) {
    //     for (var j=0; j<8; j++) {
    //         for (var k=0; k < 256; k++) {
    //             conv2d_18.in[i][j][k] <== conv2d_dw_17.out[i][j][k];
    //         }
    //     }
    // }
    // for (var m=0; m<256; m++) {
    //     for (var i=0; i<1; i++) {
    //         for (var j=0; j<1; j++) {
    //             for (var k=0; k<256; k++) {
    //                 conv2d_18.weights[m][i][j][k] <== conv2d_18_weights[m][i][j][k];
    //             }
    //         }
    //     }
    //     conv2d_18.bias[m] <== conv2d_18_bias[m];
    // }

    // // conv2d_dw_19
    // for (var i=0; i<8; i++) {
    //     for (var j=0; j<8; j++) {
    //         for (var k=0; k < 256; k++) {
    //             conv2d_dw_19.in[i][j][k] <== conv2d_18.out[i][j][k];
    //         }
    //     }
    // }
    // for (var k=256; k<256; k++) {
    //     for (var i=0; i<3; i++) {
    //         for (var j=0; j<3; j++) {
    //             conv2d_dw_19.weights[i][j][k] <== conv2d_dw_19_weights[i][j][k];
    //         }
    //     }
    //     conv2d_dw_19.bias[k] <== conv2d_dw_19_bias[k];
    // }


    // // conv2d_20
    // for (var i=0; i<8; i++) {
    //     for (var j=0; j<8; j++) {
    //         for (var k=0; k < 256; k++) {
    //             conv2d_20.in[i][j][k] <== conv2d_dw_19.out[i][j][k];
    //         }
    //     }
    // }
    // for (var m=0; m<256; m++) {
    //     for (var i=0; i<1; i++) {
    //         for (var j=0; j<1; j++) {
    //             for (var k=0; k<256; k++) {
    //                 conv2d_20.weights[m][i][j][k] <== conv2d_20_weights[m][i][j][k];
    //             }
    //         }
    //     }
    //     conv2d_20.bias[m] <== conv2d_20_bias[m];
    // }

    // // conv2d_dw_21
    // for (var i=0; i<8; i++) {
    //     for (var j=0; j<8; j++) {
    //         for (var k=0; k < 256; k++) {
    //             conv2d_dw_21.in[i][j][k] <== conv2d_20.out[i][j][k];
    //         }
    //     }
    // }
    // for (var k=256; k<256; k++) {
    //     for (var i=0; i<3; i++) {
    //         for (var j=0; j<3; j++) {
    //             conv2d_dw_21.weights[i][j][k] <== conv2d_dw_21_weights[i][j][k];
    //         }
    //     }
    //     conv2d_dw_21.bias[k] <== conv2d_dw_21_bias[k];
    // }

    // // conv2d_22
    // for (var i=0; i<8; i++) {
    //     for (var j=0; j<8; j++) {
    //         for (var k=0; k < 256; k++) {
    //             conv2d_22.in[i][j][k] <== conv2d_dw_21.out[i][j][k];
    //         }
    //     }
    // }
    // for (var m=0; m<256; m++) {
    //     for (var i=0; i<1; i++) {
    //         for (var j=0; j<1; j++) {
    //             for (var k=0; k<256; k++) {
    //                 conv2d_22.weights[m][i][j][k] <== conv2d_22_weights[m][i][j][k];
    //             }
    //         }
    //     }
    //     conv2d_22.bias[m] <== conv2d_22_bias[m];
    // }

    // // conv2d_dw_23
    // for (var i=0; i<8; i++) {
    //     for (var j=0; j<8; j++) {
    //         for (var k=0; k < 256; k++) {
    //             conv2d_dw_23.in[i][j][k] <== conv2d_22.out[i][j][k];
    //         }
    //     }
    // }
    // for (var k=256; k<256; k++) {
    //     for (var i=0; i<3; i++) {
    //         for (var j=0; j<3; j++) {
    //             conv2d_dw_23.weights[i][j][k] <== conv2d_dw_23_weights[i][j][k];
    //         }
    //     }
    //     conv2d_dw_23.bias[k] <== conv2d_dw_23_bias[k];
    // }

    // // conv2d_24
    // for (var i=0; i<4; i++) {
    //     for (var j=0; j<4; j++) {
    //         for (var k=0; k < 256; k++) {
    //             conv2d_24.in[i][j][k] <== conv2d_dw_23.out[i][j][k];
    //         }
    //     }
    // }
    // for (var m=0; m<512; m++) {
    //     for (var i=0; i<1; i++) {
    //         for (var j=0; j<1; j++) {
    //             for (var k=0; k<256; k++) {
    //                 conv2d_24.weights[m][i][j][k] <== conv2d_24_weights[m][i][j][k];
    //             }
    //         }
    //     }
    //     conv2d_24.bias[m] <== conv2d_24_bias[m];
    // }

    // // conv2d_dw_25
    // for (var i=0; i<4; i++) {
    //     for (var j=0; j<4; j++) {
    //         for (var k=0; k < 512; k++) {
    //             conv2d_dw_25.in[i][j][k] <== conv2d_24.out[i][j][k];
    //         }
    //     }
    // }
    // for (var k=512; k<512; k++) {
    //     for (var i=0; i<3; i++) {
    //         for (var j=0; j<3; j++) {
    //             conv2d_dw_25.weights[i][j][k] <== conv2d_dw_25_weights[i][j][k];
    //         }
    //     }
    //     conv2d_dw_25.bias[k] <== conv2d_dw_25_bias[k];
    // }


    // // conv2d_26
    // for (var i=0; i<4; i++) {
    //     for (var j=0; j<4; j++) {
    //         for (var k=0; k < 512; k++) {
    //             conv2d_26.in[i][j][k] <== conv2d_dw_25.out[i][j][k];
    //         }
    //     }
    // }
    // for (var m=0; m<512; m++) {
    //     for (var i=0; i<1; i++) {
    //         for (var j=0; j<1; j++) {
    //             for (var k=0; k<512; k++) {
    //                 conv2d_26.weights[m][i][j][k] <== conv2d_26_weights[m][i][j][k];
    //             }
    //         }
    //     }
    //     conv2d_26.bias[m] <== conv2d_26_bias[m];
    // }

    // // Average Pool
    // for (var i=0; i<4; i++) {
    //     for (var j=0; j<4; j++) {
    //         for (var k=0; k < 512; k++) {
    //             avg_pool.in[i][j][k] <== conv2d_26.out[i][j][k];
    //         }
    //     }
    // }

    // // conv2d_28
    // for (var i=0; i<1; i++) {
    //     for (var j=0; j<1; j++) {
    //         for (var k=0; k < 512; k++) {
    //             conv2d_28.in[i][j][k] <== avg_pool.out[i][j][k];
    //         }
    //     }
    // }
    // for (var m=0; m<512; m++) {
    //     for (var i=0; i<1; i++) {
    //         for (var j=0; j<1; j++) {
    //             for (var k=0; k<512; k++) {
    //                 conv2d_28.weights[m][i][j][k] <== conv2d_28_weights[m][i][j][k];
    //             }
    //         }
    //     }
    //     conv2d_28.bias[m] <== conv2d_28_bias[m];
    // }

    // for (var i=0; i<1001; i++) {
    //     out[i] <== conv2d_28.out[1][1][i];
    // }
}

component main = mobilenet_v1_25_128_no_relu();