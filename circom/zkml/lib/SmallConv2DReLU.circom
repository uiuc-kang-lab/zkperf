pragma circom 2.0.0;

include "./circomlib-matrix/matElemMul.circom";
include "./circomlib-matrix/matElemSum.circom";
include "./ReLU.circom";
include "./util.circom";

// Conv2D layer with valid padding
template Small_Conv2D_BatchNorm_ReLU (nRows, nCols, nChannels, nFilters, kernelSize, strides) {
    signal input in[nRows][nCols][nChannels];

    signal input weights_dw[kernelSize][kernelSize][nChannels];
    signal input bias_dw[nChannels];

    signal input weights_p[nChannels][nFilters];
    signal input bias_p[nFilters];


    component mul_dw[(nRows-kernelSize)\strides+1][(nCols-kernelSize)\strides+1][nChannels];
    component elemSum_dw[(nRows-kernelSize)\strides+1][(nCols-kernelSize)\strides+1][nChannels];
    // component sum_dw[(nRows-kernelSize)\strides+1][(nCols-kernelSize)\strides+1][nChannels];
    component relu_dw[(nRows-kernelSize)\strides+1][(nCols-kernelSize)\strides+1][nChannels];

    component relu_p[(nRows-kernelSize)\strides+1][(nCols-kernelSize)\strides+1][nFilters];
    component sum_p[(nRows-kernelSize)\strides+1][(nCols-kernelSize)\strides+1][nFilters];


    signal output out[(nRows-kernelSize)\strides+1][(nCols-kernelSize)\strides+1][nFilters];

    // DW Conv
    for (var i=0; i<(nRows-kernelSize)\strides+1; i++) {
        for (var j=0; j<(nCols-kernelSize)\strides+1; j++) {
            for (var k=0; k<nChannels; k++) {
                mul_dw[i][j][k] = matElemMul(kernelSize,kernelSize);
                for (var x=0; x<kernelSize; x++) {
                    for (var y=0; y<kernelSize; y++) {
                        mul_dw[i][j][k].a[x][y] <== in[i*strides+x][j*strides+y][k];
                        mul_dw[i][j][k].b[x][y] <== weights_dw[x][y][k];
                    }
                }
                elemSum_dw[i][j][k] = matElemSum(kernelSize,kernelSize);
                for (var x=0; x<kernelSize; x++) {
                    for (var y=0; y<kernelSize; y++) {
                        elemSum_dw[i][j][k].a[x][y] <== mul_dw[i][j][k].out[x][y];
                    }
                }
            }
        }
    }

    // ReLU Layer for DW Conv
    for (var i=0; i<(nRows-kernelSize)\strides+1; i++) {
        for (var j=0; j<(nCols-kernelSize)\strides+1; j++) {
            for (var k = 0; k < nChannels; k++) {
                relu_dw[i][j][k] = ReLU();
                relu_dw[i][j][k].in <== elemSum_dw[i][j][k].out + bias_dw[k];
            }
        }
    }

    // PW Conv
    for (var i=0; i<(nRows-kernelSize)\strides+1; i++) {
        for (var j=0; j<(nCols-kernelSize)\strides+1; j++) {
            for (var m=0; m<nFilters; m++) {
                sum_p[i][j][m] = Sum(nChannels);
                for (var k=0; k<nChannels; k++) {
                    sum_p[i][j][m].in[k] <== relu_dw[i][j][k].out * weights_p[k][m];
                }
            }
        }
    }

    // ReLU Layer for PW Conv
    for (var i=0; i<(nRows-kernelSize)\strides+1; i++) {
        for (var j=0; j<(nCols-kernelSize)\strides+1; j++) {
            for (var m=0; m<nFilters; m++) {
                relu_p[i][j][m] = ReLU();
                relu_p[i][j][m].in <== sum_p[i][j][m].out + bias_p[m];
                out[i][j][m] <== relu_p[i][j][m].out;
            }
        }
    }


}