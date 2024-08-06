package gnarkml

import (
	"fmt"
	"math"
	"math/big"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/std/rangecheck"
)

func Conv2D(api frontend.API, image [][][]frontend.Variable, kernel [][][][]frontend.Variable, bias []frontend.Variable, stride int, padding int, reluoff bool, Global_sf frontend.Variable, N int) [][][]frontend.Variable {
	/*
		Image: [H][W][C]
		Kernel: [N][H'][W'][C]
		Stride: int
		Padding: 0 == SAME padding, 1 == VALID padding
		Output: [(H+2P-H')/stride+1][(W+2P-W')/stride+1][N]
		DivLoookup: Yes
	*/
	var padlen int
	if padding == 0 {
		padlen = (len(kernel[0][0]) - 1) / 2
		paddedImage := make([][][]frontend.Variable, len(image)+2*padlen)
		for i := 0; i < len(image)+2*padlen; i++ {
			paddedImage[i] = make([][]frontend.Variable, len(image[0])+2*padlen)
			for j := 0; j < len(image[0])+2*padlen; j++ {
				paddedImage[i][j] = make([]frontend.Variable, len(image[0][0]))
				for k := 0; k < len(image[0][0]); k++ {
					if i < padlen || j < padlen || i >= len(image)+padlen || j >= len(image[0])+padlen {
						paddedImage[i][j][k] = frontend.Variable(0)
					} else {
						paddedImage[i][j][k] = image[i-padlen][j-padlen][k]
					}
				}
			}
		}
		image = paddedImage
	} else {
		padlen = 0
	}
	output_dim := make([]int, 3)
	output_dim[0] = (len(image)-len(kernel[0]))/stride + 1
	output_dim[1] = (len(image[0])-len(kernel[0][0]))/stride + 1
	output_dim[2] = len(kernel)
	fmt.Println("Output Dim", output_dim)
	output := make([][][]frontend.Variable, output_dim[0])
	for i := 0; i < output_dim[0]; i++ {
		output[i] = make([][]frontend.Variable, output_dim[1])
		for j := 0; j < output_dim[1]; j++ {
			output[i][j] = make([]frontend.Variable, output_dim[2])
			for k := 0; k < len(kernel); k++ {
				output[i][j][k] = frontend.Variable(0)
				for n := 0; n < len(kernel[0][0][0]); n++ {
					for m := 0; m < len(kernel[0]); m++ {
						for l := 0; l < len(kernel[0][0]); l++ {
							buf := api.Mul(image[i*stride+m][j*stride+l][n], kernel[k][m][l][n])
							output[i][j][k] = api.Add(output[i][j][k], buf)
						}
					}
				}
				// DownScale
				output[i][j][k] = DownScale(api, output[i][j][k], Global_sf, N)
				output[i][j][k] = api.Add(output[i][j][k], bias[k])
				if !reluoff {
					output[i][j][k] = RELU6_optimized(api, output[i][j][k])
				}
			}
		}
	}
	return output
}

func AvgPool2D(api frontend.API, Input [][][]frontend.Variable, params []int, Global_sf frontend.Variable, N int) [][][]frontend.Variable {
	/*
		Input: [H][W][C]
		Params: [Filter_height, Filter_width, Stride_height, Stride_width]
		Padding: Valid, Ignoring this for time being since we have only valid padding for model under considertation
		Output: [(H-Filter_height)/stride+1][(W-Filter_width)/stride+1][C]
	*/
	output_dim := make([]int, 3)
	output_dim[0] = (len(Input)-params[0])/params[2] + 1
	output_dim[1] = (len(Input[0])-params[1])/params[3] + 1
	output_dim[2] = len(Input[0][0])
	output := make([][][]frontend.Variable, output_dim[0])
	for i := 0; i < output_dim[0]; i++ {
		output[i] = make([][]frontend.Variable, output_dim[1])
		for j := 0; j < output_dim[1]; j++ {
			output[i][j] = make([]frontend.Variable, output_dim[2])
			for k := 0; k < output_dim[2]; k++ {
				output[i][j][k] = frontend.Variable(0)
				for m := 0; m < params[0]; m++ {
					for l := 0; l < params[1]; l++ {
						output[i][j][k] = api.Add(output[i][j][k], Input[i*params[2]+m][j*params[3]+l][k])
					}
				}
				df_int := params[0] * params[2]
				logdivfactor := int(math.Log2(float64(df_int)))
				DivFactor := frontend.Variable(df_int)
				// Division
				output[i][j][k] = DownScale(api, output[i][j][k], DivFactor, logdivfactor)
			}
		}
	}
	return output
}

func Add(api frontend.API, input [][][]frontend.Variable, bias []frontend.Variable, Global_sf frontend.Variable, N int) [][][]frontend.Variable {
	/*
		input: [H][W][C]
		bias: [C]
		Output: [H][W][C]
	*/
	output := make([][][]frontend.Variable, len(input))
	for i := 0; i < len(input); i++ {
		output[i] = make([][]frontend.Variable, len(input[0]))
		for j := 0; j < len(input[0]); j++ {
			output[i][j] = make([]frontend.Variable, len(input[0][0]))
			for k := 0; k < len(input[0][0]); k++ {
				output[i][j][k] = api.Add(input[i][j][k], bias[k])
			}
		}
	}
	return output
}

func Add_Layers(api frontend.API, input1 [][][]frontend.Variable, input2 [][][]frontend.Variable) [][][]frontend.Variable {
	output := make([][][]frontend.Variable, len(input1))
	for i := 0; i < len(input1); i++ {
		output[i] = make([][]frontend.Variable, len(input1[0]))
		for j := 0; j < len(input1[0]); j++ {
			output[i][j] = make([]frontend.Variable, len(input1[0][0]))
			for k := 0; k < len(input1[0][0]); k++ {
				output[i][j][k] = api.Add(input1[i][j][k], input2[i][j][k])
			}
		}
	}
	return output
}

func FullyConnected(api frontend.API, input [][]frontend.Variable, weight [][]frontend.Variable, bias []frontend.Variable, params []int, Gloabl_sf frontend.Variable, N int, biasoff bool) [][]frontend.Variable {
	/*
		Input: [H][C]
		Weight: [N]][C]
		Bias: [N]
		Output: [H][N]
	*/
	weight = Transpose(api, weight)
	output := make([][]frontend.Variable, len(input))
	// fmt.Println("Output Dim:", len(input), len(weight[0]))
	for i := 0; i < len(input); i++ {
		output[i] = make([]frontend.Variable, len(weight[0]))
		for j := 0; j < len(weight[0]); j++ {
			output[i][j] = frontend.Variable(0)
			for k := 0; k < len(input[0]); k++ {
				buf := api.Mul(input[i][k], weight[k][j])
				output[i][j] = api.Add(buf, output[i][j])
			}
			output[i][j] = DownScale(api, output[i][j], Gloabl_sf, N) //Downscale

			if !biasoff {
				output[i][j] = api.Add(output[i][j], bias[j])
			}
			if params[0] == 1 {
				output[i][j] = RELU_optimized(api, output[i][j])
			}
		}
	}

	return output
}

func Mul(api frontend.API, input [][][]frontend.Variable, scale []frontend.Variable, Global_sf frontend.Variable, N int) [][][]frontend.Variable {
	/*
		input: [H][W][C]
		scale: [C]
		Output: [H][W][C]
	*/
	output := make([][][]frontend.Variable, len(input))
	for i := 0; i < len(input); i++ {
		output[i] = make([][]frontend.Variable, len(input[0]))
		for j := 0; j < len(input[0]); j++ {
			output[i][j] = make([]frontend.Variable, len(input[0][0]))
			for k := 0; k < len(input[0][0]); k++ {
				output[i][j][k] = api.Mul(input[i][j][k], scale[k])
				output[i][j][k] = DownScale(api, output[i][j][k], Global_sf, N)
			}
		}
	}
	return output
}
func DownScale(api frontend.API, Dividend frontend.Variable, ScaleFactor frontend.Variable, N int) frontend.Variable {
	divResult, _ := api.Compiler().NewHint(DivHintNew, 3, Dividend, ScaleFactor)
	quotient := divResult[0]
	remainder := divResult[1]
	// (2 * a + b) = (2 * b) * q + r
	lhs := api.Mul(frontend.Variable(2), Dividend)
	lhs = api.Add(lhs, ScaleFactor)
	rhs := api.Mul(ScaleFactor, quotient)
	rhs = api.Mul(rhs, frontend.Variable(2))
	rhs = api.Add(rhs, remainder)
	api.AssertIsEqual(lhs, rhs)
	// range of remainder
	r := rangecheck.New(api)
	r.Check(remainder, N+1) // 2b-r > 0
	return quotient
}

func DivHintNew(_ *big.Int, inputs []*big.Int, results []*big.Int) error {
	modulus := ecc.BN254.ScalarField()
	half_modulus := new(big.Int).Div(modulus, big.NewInt(2))
	if inputs[0].Cmp(half_modulus) == 1 { // Handling Negative Numbers
		inputs[0] = new(big.Int).Sub(modulus, inputs[0])
		inputs[0] = inputs[0].Neg(inputs[0])
	}
	// q = (2 * a + b) / (2 * b)
	numerator := new(big.Int).Mul(big.NewInt(2), inputs[0])
	numerator = new(big.Int).Add(numerator, inputs[1])
	denominator := new(big.Int).Mul(big.NewInt(2), inputs[1])
	results[0] = new(big.Int).Div(numerator, denominator)
	// r = (2 * a + b) % (2 * b)
	results[1] = new(big.Int).Mod(numerator, denominator)
	return nil
}

func DivHint(_ *big.Int, inputs []*big.Int, results []*big.Int) error {
	modulus := ecc.BN254.ScalarField()
	half_modulus := new(big.Int).Div(modulus, big.NewInt(2))
	if inputs[0].Cmp(half_modulus) == 1 { // Handling Negative Numbers
		inputs[0] = new(big.Int).Sub(modulus, inputs[0])
		inputs[0] = inputs[0].Neg(inputs[0])
	}
	results[0] = new(big.Int).Div(inputs[0], inputs[1])
	results[1] = new(big.Int).Mod(inputs[0], inputs[1])

	if results[1].Cmp(inputs[1].Div(inputs[1], big.NewInt(2))) == 1 {
		results[2] = big.NewInt(1)
	} else {
		results[2] = big.NewInt(0)
	}
	return nil
}
