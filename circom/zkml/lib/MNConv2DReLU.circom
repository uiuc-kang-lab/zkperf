pragma circom 2.0.0;

include "./circomlib-matrix/matElemMul.circom";
include "./circomlib-matrix/matElemSum.circom";
include "./ReLU.circom";
include "./util.circom";

// Conv2D layer with valid padding and no dilation
template MN_Conv2D_ReLU (nRows, nCols, nChannels, nFilters, kernelSize, strides) {
    signal input in[nRows+2][nCols+2][nChannels];
    signal input weights[nFilters][kernelSize][kernelSize][nChannels];
    signal input bias[nFilters];
    signal output out[(nRows-kernelSize+2)\strides+1][(nCols-kernelSize+2)\strides+1][nFilters];

    component mul[(nRows-kernelSize+2)\strides+1][(nCols-kernelSize+2)\strides+1][nChannels][nFilters];
    component elemSum[(nRows-kernelSize+2)\strides+1][(nCols-kernelSize+2)\strides+1][nChannels][nFilters];
    component sum[(nRows-kernelSize+2)\strides+1][(nCols-kernelSize+2)\strides+1][nFilters];
    component relu[(nRows-kernelSize+2)\strides+1][(nCols-kernelSize+2)\strides+1][nFilters];

    for (var i=0; i<(nRows-kernelSize+2)\strides+1; i++) {
        for (var j=0; j<(nCols-kernelSize+2)\strides+1; j++) {
            for (var k=0; k<nChannels; k++) {
                for (var m=0; m<nFilters; m++) {
                    mul[i][j][k][m] = matElemMul(kernelSize,kernelSize);
                    for (var x=0; x<kernelSize; x++) {
                        for (var y=0; y<kernelSize; y++) {
                            mul[i][j][k][m].a[x][y] <== in[i*strides+x][j*strides+y][k];
                            mul[i][j][k][m].b[x][y] <== weights[m][x][y][k];
                        }
                    }
                    elemSum[i][j][k][m] = matElemSum(kernelSize,kernelSize);
                    for (var x=0; x<kernelSize; x++) {
                        for (var y=0; y<kernelSize; y++) {
                            elemSum[i][j][k][m].a[x][y] <== mul[i][j][k][m].out[x][y];
                        }
                    }
                }
            }
            for (var m=0; m<nFilters; m++) {
                sum[i][j][m] = Sum(nChannels);
                for (var k=0; k<nChannels; k++) {
                    sum[i][j][m].in[k] <== elemSum[i][j][k][m].out;
                }
            }
        }
    }

    // ReLU Layer
    for (var i=0; i<(nRows-kernelSize+2)\strides+1; i++) {
        for (var j=0; j<(nCols-kernelSize+2)\strides+1; j++) {
            for (var m=0; m<nFilters; m++) {
                relu[i][j][m] = ReLU();
                relu[i][j][m].in <== sum[i][j][m].out + bias[m];
                out[i][j][m] <== relu[i][j][m].out;
            }
        }
    }


}