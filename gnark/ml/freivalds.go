package gnarkml

import (
	"fmt"
	"math/big"

	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/std/multicommit"
)

// Try Freivalds
func FullyConnectedWithFreivalds(api frontend.API, input [][]frontend.Variable, weight [][]frontend.Variable) [][]frontend.Variable {
	/*
		Input: [H][C]
		Weight: [N][C]
		Bias: [N]
		Output: [H][N]
	*/
	weight = Transpose(api, weight)
	output := make([][]frontend.Variable, len(input))
	outputcopy := make([][]frontend.Variable, len(input))
	mergelength := len(input)*len(input[0]) + len(weight)*len(weight[0]) + len(input)*len(weight[0])
	tocommit := make([]frontend.Variable, mergelength)
	// Outside Circuitclea
	fmt.Println("check point 1")
	for i := 0; i < len(input); i++ {
		output[i] = make([]frontend.Variable, len(weight[0]))
		outputcopy[i] = make([]frontend.Variable, len(weight[0]))
		for j := 0; j < len(weight[0]); j++ {
			output[i][j] = frontend.Variable(0)
			outputcopy[i][j] = frontend.Variable(0)
			for k := 0; k < len(input[0]); k++ {
				hintOP, _ := api.Compiler().NewHint(FCHint, 1, input[i][k], weight[k][j], output[i][j])
				output[i][j] = hintOP[0]
				outputcopy[i][j] = hintOP[0]
				// output[i][j] = DownScale(api, output[i][j], Gloabl_sf, N) //Downscale
			}
		}
	}
	//Merge Arrays
	t := 0
	for i := 0; i < len(input); i++ {
		for j := 0; j < len(input[0]); j++ {
			tocommit[t] = input[i][j]
			t++
		}
	}

	for i := 0; i < len(weight); i++ {
		for j := 0; j < len(weight[0]); j++ {
			tocommit[t] = weight[i][j]
			t++
		}
	}
	for i := 0; i < len(outputcopy); i++ {
		for j := 0; j < len(outputcopy[0]); j++ {
			tocommit[t] = outputcopy[i][j]
			t++
		}
	}
	// Create Randomess
	multicommit.WithCommitment(api, func(api frontend.API, commitment frontend.Variable) error {
		r := commitment
		fmt.Println(r)
		rlen := len(outputcopy[0])
		rVec := make([]frontend.Variable, len(outputcopy[0]))
		for i := 0; i < rlen; i++ {
			// rVec[i] = power(api, r, i)
			rVec[i] = r
		}

		// Freivalds
		// LHS: Ax(BxR); RHS: CxR
		lhs_dash := make([][]frontend.Variable, len(weight))
		for i := 0; i < len(weight); i++ {
			lhs_dash[i] = make([]frontend.Variable, 1)
			for j := 0; j < 1; j++ {
				lhs_dash[i][j] = frontend.Variable(0)
				for k := 0; k < len(weight[0]); k++ {
					buf := api.Mul(weight[i][k], rVec[k])
					lhs_dash[i][j] = api.Add(lhs_dash[i][j], buf)
				}
			}
		}

		lhs := make([][]frontend.Variable, len(input))
		for i := 0; i < len(input); i++ {
			lhs[i] = make([]frontend.Variable, 1)
			for j := 0; j < 1; j++ {
				lhs[i][j] = frontend.Variable(0)
				for k := 0; k < len(input[0]); k++ {
					api.Println("inp[i][k]:", input[i][k])
					buf := api.Mul(input[i][k], lhs_dash[k][j])
					lhs[i][j] = api.Add(lhs[i][j], buf)
				}

			}
		}

		rhs := make([][]frontend.Variable, len(outputcopy))
		for i := 0; i < len(outputcopy); i++ {
			rhs[i] = make([]frontend.Variable, 1)
			for j := 0; j < 1; j++ {
				rhs[i][j] = frontend.Variable(0)
				for k := 0; k < len(outputcopy[0]); k++ {
					buf := api.Mul(outputcopy[i][k], rVec[k])
					// api.Println("outputcopy[i][k]:", outputcopy[i][k])
					// api.Println("rVec[k]:", rVec[k])
					// api.Println("buf:", buf)
					rhs[i][j] = api.Add(rhs[i][j], buf)
				}
			}
		}
		// Constraints
		for i := 0; i < len(outputcopy); i++ {
			api.AssertIsEqual(lhs[i][0], rhs[i][0])
		}
		return nil
	}, tocommit[:]...)

	return output
}

func PostFV(api frontend.API, output [][]frontend.Variable, bias []frontend.Variable, params []int, Gloabl_sf frontend.Variable, N int, biasoff bool) [][]frontend.Variable {
	for i := 0; i < len(output); i++ {
		for j := 0; j < len(output[0]); j++ {

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

func FCHint(_ *big.Int, inputs []*big.Int, results []*big.Int) error {
	// inputs = [a,b,running_sum]
	buf := new(big.Int).Mul(inputs[0], inputs[1])
	results[0] = new(big.Int).Add(buf, inputs[2])
	return nil
}

func BiasHint(_ *big.Int, inputs []*big.Int, results []*big.Int) error {
	results[0] = new(big.Int).Add(inputs[0], inputs[1])
	return nil
}

func power(api frontend.API, r frontend.Variable, d int) frontend.Variable {
	rpower := frontend.Variable(1)
	for i := 0; i < d; i++ {
		buf := api.Mul(r, rpower)
		rpower = buf
	}
	return rpower
}

func print_r(api frontend.API, com frontend.Variable) {
	api.Println(com)
}
