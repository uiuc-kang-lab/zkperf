package main

import (
	"fmt"
	"math/rand"
	"testing"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/backend"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/test"
)

func TestLT(t *testing.T) {
	assert := test.NewAssert(t)
	n := 10
	k := 50
	loookuptable := make([]frontend.Variable, n)
	lookuptableObj := make([]frontend.Variable, n)
	for i := 0; i < n; i++ {
		loookuptable[i] = frontend.Variable(i)
	}
	fmt.Println("n k: ", n, k)
	queries := make([]frontend.Variable, k)
	queriesObj := make([]frontend.Variable, k)
	for i := 0; i < k; i++ {
		buf := rand.Intn(n) // random number between 0 and n-1
		queries[i] = frontend.Variable(buf)
	}

	circuit := GnarkLookupCircuit{lookuptableObj, queriesObj}
	witness := GnarkLookupCircuit{loookuptable, queries}

	assert.ProverSucceeded(&circuit, &witness, test.WithCurves(ecc.BN254), test.WithBackends(backend.GROTH16))
}
