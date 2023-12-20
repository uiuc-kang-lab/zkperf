package gnarkml

import (
	"math"
	"math/big"

	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/std/math/cmp"
)

func RELU(api frontend.API, Input frontend.Variable, modulus big.Int) frontend.Variable {
	zk_zero := new(big.Int).Div(&modulus, big.NewInt(2))
	zero := frontend.Variable(0)
	six := frontend.Variable(6)
	buf := api.Cmp(zk_zero, Input)
	bitbuf := api.ToBinary(buf)
	output := api.Select(bitbuf[253], zero, Input)
	buf = api.Cmp(six, output)
	bitbuf = api.ToBinary(buf)
	output = api.Select(bitbuf[253], six, output)
	return output
}

func RELU_optimized(api frontend.API, Input frontend.Variable) frontend.Variable {
	input_binary := api.ToBinary(Input)
	t := input_binary[253]
	output := api.Select(t, frontend.Variable(0), Input)
	return output
}

func RELU6_optimized(api frontend.API, Input frontend.Variable) frontend.Variable {
	input_binary := api.ToBinary(Input)
	t := input_binary[253]
	output := api.Select(t, frontend.Variable(0), Input)
	newCmp := cmp.NewBoundedComparator(api, big.NewInt(int64(math.Pow(2, 27))), false)
	buf1 := newCmp.IsLess(output, frontend.Variable(6*512))
	output = api.Select(buf1, output, frontend.Variable(6*512))
	return output
}
