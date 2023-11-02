pragma circom 2.0.0;

include "./circomlib-matrix/matElemMul.circom";
include "./circomlib-matrix/matElemSum.circom";
include "./ReLU.circom";
include "./util.circom";

// DW Conv2D layer with valid padding
template Conv2D_DW_ReLU (nRows, nCols, nChannels, kernelSize, strides) {
    signal input in[nRows+2][nCols+2][nChannels];

    signal input weights[kernelSize][kernelSize][nChannels];
    signal input bias[nChannels];

    component mul[(nRows-kernelSize+2)\strides+1][(nCols-kernelSize+2)\strides+1][nChannels];
    component elemSum[(nRows-kernelSize+2)\strides+1][(nCols-kernelSize+2)\strides+1][nChannels];
    component relu[(nRows-kernelSize+2)\strides+1][(nCols-kernelSize+2)\strides+1][nChannels];

    signal output out[(nRows-kernelSize+2)\strides+1][(nCols-kernelSize+2)\strides+1][nChannels];

    // DW Conv
    for (var i=0; i<(nRows-kernelSize+2)\strides+1; i++) {
        for (var j=0; j<(nCols-kernelSize+2)\strides+1; j++) {
            for (var k=0; k<nChannels; k++) {
                mul[i][j][k] = matElemMul(kernelSize,kernelSize);
                for (var x=0; x<kernelSize; x++) {
                    for (var y=0; y<kernelSize; y++) {
                        mul[i][j][k].a[x][y] <== in[i*strides+x][j*strides+y][k];
                        mul[i][j][k].b[x][y] <== weights[x][y][k];
                    }
                }
                elemSum[i][j][k] = matElemSum(kernelSize,kernelSize);
                for (var x=0; x<kernelSize; x++) {
                    for (var y=0; y<kernelSize; y++) {
                        elemSum[i][j][k].a[x][y] <== mul[i][j][k].out[x][y];
                    }
                }
            }
        }
    }

    // ReLU Layer for DW Conv
    for (var i=0; i<(nRows-kernelSize+2)\strides+1; i++) {
        for (var j=0; j<(nCols-kernelSize+2)\strides+1; j++) {
            for (var k = 0; k < nChannels; k++) {
                relu[i][j][k] = ReLU();
                relu[i][j][k].in <== elemSum[i][j][k].out + bias[k];
            }
        }
    }
}