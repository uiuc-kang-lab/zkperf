package gnarkml

import (
	"github.com/consensys/gnark/frontend"
)

// Call Tranpose vs Transpose3D based on the params of the transpose layer

func Transpose(api frontend.API, input [][]frontend.Variable) [][]frontend.Variable {
	/*
		Input: [H][C]
		Output: [C][H]
	*/
	output := make([][]frontend.Variable, len(input[0]))
	for i := 0; i < len(input[0]); i++ {
		output[i] = make([]frontend.Variable, len(input))
		for j := 0; j < len(input); j++ {
			output[i][j] = input[j][i]
		}
	}
	return output
}

func Transpose3D_1(api frontend.API, input [][][]frontend.Variable, opshape []int) [][][]frontend.Variable {
	// [0,1,2] -> [0,2,1]
	output := make([][][]frontend.Variable, opshape[0])
	for i := 0; i < opshape[0]; i++ {
		output[i] = make([][]frontend.Variable, opshape[1])
		for j := 0; j < opshape[1]; j++ {
			output[i][j] = make([]frontend.Variable, opshape[2])
			for k := 0; k < opshape[2]; k++ {
				output[i][j][k] = input[i][k][j]
			}
		}
	}
	return output
}

func Transpose3D_2(api frontend.API, input [][][]frontend.Variable, opshape []int) [][][]frontend.Variable {
	// [0,1,2] -> [1,2,0]
	output := make([][][]frontend.Variable, opshape[0])
	for i := 0; i < opshape[0]; i++ {
		output[i] = make([][]frontend.Variable, opshape[1])
		for j := 0; j < opshape[1]; j++ {
			output[i][j] = make([]frontend.Variable, opshape[2])
			for k := 0; k < opshape[2]; k++ {
				output[i][j][k] = input[k][j][i]
			}
		}
	}

	return output
}

func Concatenate(api frontend.API, input1 [][]frontend.Variable, input2 [][]frontend.Variable) [][]frontend.Variable {
	/*
		Input1: [H][C1]
		Input2: [H][C2]
		Output: [H][C1+C2]
	*/
	output := make([][]frontend.Variable, len(input1))
	for i := 0; i < len(input1); i++ {
		output[i] = make([]frontend.Variable, len(input1[0])+len(input2[0]))
		for j := 0; j < len(input1[0]); j++ {
			output[i][j] = input1[i][j]
		}
		for j := 0; j < len(input2[0]); j++ {
			output[i][j+len(input1[0])] = input2[i][j]
		}
	}
	return output
}

func Reshape1Dto2D(api frontend.API, input [][]frontend.Variable, outputshape []int) [][]frontend.Variable {
	output := make([][]frontend.Variable, outputshape[0])
	index := 0
	for i := 0; i < outputshape[0]; i++ {
		output[i] = make([]frontend.Variable, outputshape[1])
		for j := 0; j < outputshape[1]; j++ {
			output[i][j] = input[0][index]
			index += 1
		}
	}
	return output
}

func Reshape2Dto1D(api frontend.API, input [][]frontend.Variable, outputshape []int) [][]frontend.Variable {
	output := make([][]frontend.Variable, outputshape[0])
	index := 0
	for i := 0; i < len(input); i++ {
		for j := 0; j < len(input[0]); j++ {
			output[index] = make([]frontend.Variable, outputshape[1])
			output[index][0] = input[i][j]
			index += 1
		}
	}
	return output
}

// Call Reshape3Dto2D vs Reshape2Dto3D based on the params of the reshape layer

func Reshape3Dto2D(api frontend.API, input [][][]frontend.Variable, outputshape []int) [][]frontend.Variable {
	/*
		Input: [C][H][W]
		Output: [C*H][W]
	*/
	output := make([][]frontend.Variable, len(input)*len(input[0]))
	for i := 0; i < len(input); i++ {
		for j := 0; j < len(input[0]); j++ {
			output[i*len(input[0])+j] = make([]frontend.Variable, len(input[0][0]))
			for k := 0; k < len(input[0][0]); k++ {
				output[i*len(input[0])+j][k] = input[i][j][k]
			}
		}
	}
	return output
}

func Reshape2Dto3D(api frontend.API, input [][]frontend.Variable, outputshape []int) [][][]frontend.Variable {
	/*
		Input: [C][K]
		Output: [C][H][W]
		K = H*W
		H = outputshape[1]
		W = outputshape[2]
	*/
	output := make([][][]frontend.Variable, len(input))
	for i := 0; i < len(input); i++ {
		output[i] = make([][]frontend.Variable, outputshape[1])
		for j := 0; j < outputshape[1]; j++ {
			output[i][j] = make([]frontend.Variable, outputshape[2])
			for k := 0; k < outputshape[2]; k++ {
				output[i][j][k] = input[i][j*outputshape[2]+k]
			}
		}
	}
	return output
}

func Reshape(api frontend.Variable, input [][]frontend.Variable, shape []int) [][]frontend.Variable {
	output := make([][]frontend.Variable, shape[0])
	for i := 0; i < shape[0]; i++ {
		output[i] = make([]frontend.Variable, shape[1])
		for j := 0; j < shape[1]; j++ {
			output[i][j] = input[j][i]
		}
	}
	return output
}
