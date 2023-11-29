package main

import (
	"encoding/json"
	"fmt"
	"os"
	"strconv"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/backend/groth16"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/frontend/cs/r1cs"
	"github.com/consensys/gnark/test"
)

type NumConstraints struct {
	Framework     string
	Size          int
	Func          string
	NbConstraints int
}

func main() {
	step := os.Args[1]
	arg := os.Args[2]
	path := os.Args[3]
	n, _ := strconv.Atoi(arg)
	if step == "circuit" {
		circuit, witness := DummyCircuitGen(n)
		test.IsSolved(&circuit, &witness, ecc.BN254.ScalarField())
		cs, _ := frontend.Compile(ecc.BN254.ScalarField(), r1cs.NewBuilder, &circuit)
		fmt.Println("Number of constraints: ", cs.GetNbConstraints())
		pK, vK, _ := groth16.Setup(cs)
		wtns, _ := frontend.NewWitness(&witness, ecc.BN254.ScalarField())
		pubwtns, _ := wtns.Public()
		proof, _ := groth16.Prove(cs, pK, wtns)
		err := groth16.Verify(proof, vK, pubwtns)
		if err != nil {
			panic(err)
		}
	} else if step == "msm" {
		DummyMSMAffine(n)
	} else if step == "fft" {
		DummyFFT(n)
	} else if step == "arithmetic" {
		DummyArithmetic(n)
	} else if step == "relu" {
		circuit, witness := DummyRELUCircuitGen(n)
		test.IsSolved(&circuit, &witness, ecc.BN254.ScalarField())
		cs, _ := frontend.Compile(ecc.BN254.ScalarField(), r1cs.NewBuilder, &circuit)
		fmt.Println("Number of constraints: ", cs.GetNbConstraints())
		pK, vK, _ := groth16.Setup(cs)
		wtns, _ := frontend.NewWitness(&witness, ecc.BN254.ScalarField())
		pubwtns, _ := wtns.Public()
		proof, _ := groth16.Prove(cs, pK, wtns)
		err := groth16.Verify(proof, vK, pubwtns)
		if err != nil {
			panic(err)
		}
		result := NumConstraints{
			Framework:     "gnark",
			Size:          n,
			Func:          "ReLU",
			NbConstraints: cs.GetNbConstraints(),
		}
		jsonfile, _ := json.MarshalIndent(result, "", " ")
		_ = os.WriteFile(path+strconv.Itoa(n)+"_constraint_stats.json", jsonfile, 0644)
	} else if step == "relu6" {
		circuit, witness := DummyRELU6CircuitGen(n)
		test.IsSolved(&circuit, &witness, ecc.BN254.ScalarField())
		cs, _ := frontend.Compile(ecc.BN254.ScalarField(), r1cs.NewBuilder, &circuit)
		fmt.Println("Number of constraints: ", cs.GetNbConstraints())
		pK, vK, _ := groth16.Setup(cs)
		wtns, _ := frontend.NewWitness(&witness, ecc.BN254.ScalarField())
		pubwtns, _ := wtns.Public()
		proof, _ := groth16.Prove(cs, pK, wtns)
		err := groth16.Verify(proof, vK, pubwtns)
		if err != nil {
			panic(err)
		}
	}

}
