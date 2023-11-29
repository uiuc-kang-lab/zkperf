package main

import (
	"math"
	"math/big"

	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/std/math/cmp"
)

type ReLUCircuit struct {
	In []frontend.Variable
}

type ReLU6Circuit struct {
	In []frontend.Variable
}

func (circuit *ReLUCircuit) Define(api frontend.API) error {
	output := make([]frontend.Variable, len(circuit.In))
	for i := 0; i < len(circuit.In); i++ {
		output[i] = RELU_optimized(api, circuit.In[i])
		// api.Println(output[i])
	}
	return nil
}

func (circuit *ReLU6Circuit) Define(api frontend.API) error {
	output := make([]frontend.Variable, len(circuit.In))
	for i := 0; i < len(circuit.In); i++ {
		output[i] = RELU6_optimized(api, circuit.In[i])
		api.Println(output[i])
	}
	return nil
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

func DummyRELUCircuitGen(n int) (ReLUCircuit, ReLUCircuit) {
	In := make([]frontend.Variable, n)
	InObj := make([]frontend.Variable, n)
	for i := -n / 2; i < n/2; i++ {
		In[i+n/2] = frontend.Variable(i)
	}
	circuit := ReLUCircuit{
		In: InObj,
	}
	witness := ReLUCircuit{
		In: In,
	}
	return circuit, witness
}

func DummyRELU6CircuitGen(n int) (ReLU6Circuit, ReLU6Circuit) {
	In := make([]frontend.Variable, n)
	InObj := make([]frontend.Variable, n)
	for i := -n / 2; i < n/2; i++ {
		In[i+n/2] = frontend.Variable(i)
	}
	circuit := ReLU6Circuit{
		In: InObj,
	}
	witness := ReLU6Circuit{
		In: In,
	}
	return circuit, witness
}
