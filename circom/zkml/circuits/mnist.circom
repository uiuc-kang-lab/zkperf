pragma circom 2.0.0;

include "../lib/Conv2D.circom";
include "../lib/Dense.circom";
include "../lib/ArgMax.circom";
include "../lib/ReLU.circom";
include "../lib/SumPooling2D.circom";
include "../lib/BatchNormalization2D.circom";
include "../lib/Flatten2D.circom";
include "../lib/ScaleDown.circom";
include "../lib/ReLU6.circom";

template mnist(sd) {
    signal input in[28][28][1];
    signal input conv2d_1_weights[3][3][1][8];
    signal input conv2d_1_bias[8];
    signal input bn_1_a[8];
    signal input bn_1_b[8];
    signal input conv2d_2_weights[3][3][8][16];
    signal input conv2d_2_bias[16];
    signal input bn_2_a[16];
    signal input bn_2_b[16];
    signal input conv2d_3_weights[3][3][16][16];
    signal input conv2d_3_bias[16];
    signal input bn_3_a[16];
    signal input bn_3_b[16];
    signal input conv2d_4_weights[1][1][16][4];
    signal input conv2d_4_bias[4];
    signal input bn_4_a[16];
    signal input bn_4_b[16];
    signal input conv2d_5_weights[3][3][4][64];
    signal input conv2d_5_bias[64];
    signal input conv2d_6_weights[1][1][64][10];
    signal input conv2d_6_bias[10];

    signal output out[10];

    component conv2d_1 = Conv2D(28,28,1,8,3,1);
    component relu_1[26][26][8];
    component sum2d_1 = SumPooling2D(26,26,8,2,2);
    component bn_1 = BatchNormalization2D(13,13,8);
    component sd_1[13][13][8];

    component conv2d_2 = Conv2D(13,13,8,16,3,1);
    component relu_2[11][11][16];
    component sum2d_2 = SumPooling2D(11,11,16,2,2);
    component bn_2 = BatchNormalization2D(5,5,16);
    component sd_2[5][5][16];

    component conv2d_3 = Conv2D(5,5,16,16,3,1);
    component relu_3[3][3][16];
    component bn_3 = BatchNormalization2D(3,3,16);
    component sd_3[3][3][16];

    component conv2d_4 = Conv2D(3,3,16,4,1,1);
    component relu_4[3][3][4];
    component bn_4 = BatchNormalization2D(3,3,4);
    component sd_4[3][3][4];

    component conv2d_5 = Conv2D(3,3,4,64,3,1);
    component sd_5[1][1][64];
    component relu_5[1][1][64];

    component conv2d_6 = Conv2D(1,1,64,10,1,1);

    // 1st Conv
    for (var i=0; i<28; i++) {
        for (var j=0; j<28; j++) {
            conv2d_1.in[i][j][0] <== in[i][j][0];
        }
    }    
    for (var m=0; m<8; m++) {
        for (var i=0; i<3; i++) {
            for (var j=0; j<3; j++) {
                conv2d_1.weights[i][j][0][m] <== conv2d_1_weights[i][j][0][m];
            }
        }
        conv2d_1.bias[m] <== conv2d_1_bias[m];
    }
    for (var m=0; m < 8; m++) {
        for (var i=0; i < 26; i++) {
            for (var j = 0; j < 26; j++) {
                relu_1[i][j][m] = ReLU6(2*sd);
                relu_1[i][j][m].in <== conv2d_1.out[i][j][m];
                sum2d_1.in[i][j][m] <== relu_1[i][j][m].out;
            }
        }
        bn_1.a[m] <== bn_1_a[m];
        bn_1.b[m] <== bn_1_b[m];
    }

    for (var m=0; m < 8; m++) {
        for (var i=0; i < 13; i++) {
            for (var j = 0; j < 13; j++) {
                bn_1.in[i][j][m] <== sum2d_1.out[i][j][m];
                sd_1[i][j][m] = ScaleDown(2*sd+2);
            }
        }
    }

    for (var m=0; m < 8; m++) {
        for (var i=0; i < 13; i++) {
            for (var j = 0; j < 13; j++) {
                sd_1[i][j][m].in <== bn_1.out[i][j][m];
            }
        }
    }
    

    // 2nd Conv
    for (var m=0; m < 8; m++) {
        for (var i=0; i<13; i++) {
            for (var j=0; j<13; j++) {
                conv2d_2.in[i][j][m] <== sd_1[i][j][m].out;
            }
        }
    }

    for (var n=0; n < 16; n++) {
        for (var m=0; m<8; m++) {
            for (var i=0; i<3; i++) {
                for (var j=0; j<3; j++) {
                    conv2d_2.weights[i][j][m][n] <== conv2d_2_weights[i][j][m][n];
                }
            }
        }
        conv2d_2.bias[n] <== conv2d_2_bias[n];
    }


    for (var m=0; m < 16; m++) {
        for (var i=0; i < 11; i++) {
            for (var j = 0; j < 11; j++) {
                relu_2[i][j][m] = ReLU6(2*sd);
                relu_2[i][j][m].in <== conv2d_2.out[i][j][m];
                sum2d_2.in[i][j][m] <== relu_2[i][j][m].out;
            }
        }
        bn_2.a[m] <== bn_2_a[m];
        bn_2.b[m] <== bn_2_b[m];
    }

    for (var m=0; m < 16; m++) {
        for (var i=0; i < 5; i++) {
            for (var j = 0; j < 5; j++) {
                bn_2.in[i][j][m] <== sum2d_2.out[i][j][m];
                sd_2[i][j][m] = ScaleDown(2*sd+2);
            }
        }
    }

    for (var m=0; m < 16; m++) {
        for (var i=0; i < 5; i++) {
            for (var j = 0; j < 5; j++) {
                sd_2[i][j][m].in <== bn_2.out[i][j][m];
            }
        }
    }

    // 3rd Conv
    for (var m=0; m < 16; m++) {
        for (var i=0; i<5; i++) {
            for (var j=0; j<5; j++) {
                conv2d_3.in[i][j][m] <== sd_2[i][j][m].out;
            }
        }
    }

    for (var n=0; n < 16; n++) {
        for (var m=0; m<16; m++) {
            for (var i=0; i<3; i++) {
                for (var j=0; j<3; j++) {
                    conv2d_3.weights[i][j][m][n] <== conv2d_3_weights[i][j][m][n];
                }
            }
        }
        conv2d_3.bias[n] <== conv2d_3_bias[n];
    }

    for (var m=0; m < 16; m++) {
        for (var i=0; i < 3; i++) {
            for (var j = 0; j < 3; j++) {
                relu_3[i][j][m] = ReLU6(2*sd);
                relu_3[i][j][m].in <== conv2d_3.out[i][j][m];
                bn_3.in[i][j][m] <== relu_3[i][j][m].out;
            }
        }
        bn_3.a[m] <== bn_3_a[m];
        bn_3.b[m] <== bn_3_b[m];
    }

    for (var m=0; m < 16; m++) {
        for (var i=0; i < 3; i++) {
            for (var j = 0; j < 3; j++) {
                sd_3[i][j][m] = ScaleDown(2*sd);
                sd_3[i][j][m].in <== bn_3.out[i][j][m];
            }
        }
    }
    

    // 4th Conv
    for (var m=0; m < 16; m++) {
        for (var i=0; i<3; i++) {
            for (var j=0; j<3; j++) {
                conv2d_4.in[i][j][m] <== sd_3[i][j][m].out;
            }
        }
    }

    for (var n=0; n < 4; n++) {
        for (var m=0; m<16; m++) {
            for (var i=0; i<1; i++) {
                for (var j=0; j<1; j++) {
                    conv2d_4.weights[i][j][m][n] <== conv2d_4_weights[i][j][m][n];
                }
            }
        }
        conv2d_4.bias[n] <== conv2d_4_bias[n];
    }

    for (var m=0; m < 4; m++) {
        for (var i=0; i < 3; i++) {
            for (var j = 0; j < 3; j++) {
                relu_4[i][j][m] = ReLU6(2*sd);
                relu_4[i][j][m].in <== conv2d_4.out[i][j][m];
                bn_4.in[i][j][m] <== relu_4[i][j][m].out;
            }
        }
        bn_4.a[m] <== bn_4_a[m];
        bn_4.b[m] <== bn_4_b[m];
    }
    
    for (var m=0; m < 4; m++) {
        for (var i=0; i < 3; i++) {
            for (var j = 0; j < 3; j++) {
                sd_4[i][j][m] = ScaleDown(2*sd);
                sd_4[i][j][m].in <== bn_4.out[i][j][m];
            }
        }
    }


    // 5th Conv
    for (var m=0; m < 4; m++) {
        for (var i=0; i<3; i++) {
            for (var j=0; j<3; j++) {
                conv2d_5.in[i][j][m] <== sd_4[i][j][m].out;
            }
        }
    }

    for (var n=0; n < 64; n++) {
        for (var m=0; m<4; m++) {
            for (var i=0; i<3; i++) {
                for (var j=0; j<3; j++) {
                    conv2d_5.weights[i][j][m][n] <== conv2d_5_weights[i][j][m][n];
                }
            }
        }
        conv2d_5.bias[n] <== conv2d_5_bias[n];
    }

    for (var m=0; m < 64; m++) {
        for (var i=0; i < 1; i++) {
            for (var j = 0; j < 1; j++) {
                relu_5[i][j][m] = ReLU6(2*sd);
                relu_5[i][j][m].in <== conv2d_5.out[i][j][m];
                sd_5[i][j][m] = ScaleDown(sd);
                sd_5[i][j][m].in <== relu_5[i][j][m].out;
            }
        }
    }

    // 6th Conv
    for (var m=0; m < 64; m++) {
        for (var i=0; i<1; i++) {
            for (var j=0; j<1; j++) {
                conv2d_6.in[i][j][m] <== sd_5[i][j][m].out;
            }
        }
    }

    for (var n=0; n < 10; n++) {
        for (var m=0; m<64; m++) {
            for (var i=0; i<1; i++) {
                for (var j=0; j<1; j++) {
                    conv2d_6.weights[i][j][m][n] <== conv2d_6_weights[i][j][m][n];
                }
            }
        }
        conv2d_6.bias[n] <== conv2d_6_bias[n];
    }

    for (var i=0; i < 10; i++) {
        out[i] <== conv2d_6.out[0][0][i];
    }
}

component main = mnist(9);
