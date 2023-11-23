package main

import "github.com/consensys/gnark/frontend"

type DummyCircuit struct {
	In  []frontend.Variable // Size of MSM = len(In) + 1
	Out frontend.Variable
}

func (circuit *DummyCircuit) Define(api frontend.API) error {
	buf := frontend.Variable(1)
	for i := 0; i < len(circuit.In); i++ {
		buf = api.Mul(buf, circuit.In[i])
	}
	api.AssertIsEqual(buf, circuit.Out)
	api.Println("Circuit Finished")
	return nil
}

func DummyCircuitGen(n int) (DummyCircuit, DummyCircuit) {
	In := make([]frontend.Variable, n)
	InObj := make([]frontend.Variable, n)
	var OutObj frontend.Variable
	out := 1
	for i := 0; i < n; i++ {
		In[i] = frontend.Variable(i)
		out = out * i
	}
	circuit := DummyCircuit{
		In:  InObj,
		Out: OutObj,
	}
	witness := DummyCircuit{
		In:  In,
		Out: frontend.Variable(out)}

	return circuit, witness
}
