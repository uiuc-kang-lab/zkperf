package gnarkml

import (
	"fmt"
	"testing"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/backend"
	"github.com/consensys/gnark/backend/plonk"
	"github.com/consensys/gnark/constraint/solver"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/frontend/cs/r1cs"
	"github.com/consensys/gnark/frontend/cs/scs"
	"github.com/consensys/gnark/test"
)

type FVCircuit struct {
	Input  [][]frontend.Variable
	Weight [][]frontend.Variable
}

func (circuit *FVCircuit) Define(api frontend.API) error {
	output := FullyConnectedWithFreivalds(api, circuit.Input, circuit.Weight)
	for i := 0; i < 8; i++ {
		for j := 0; j < 4; j++ {
			output[i][j] = api.Add(output[i][j], frontend.Variable(1))
		}
	}
	output = FullyConnectedWithFreivalds(api, output, circuit.Weight)
	return nil
}

func TestFVS(t *testing.T) {
	Input := make([][]frontend.Variable, 8)
	InputObj := make([][]frontend.Variable, 8)
	Weight := make([][]frontend.Variable, 4)
	WeightObj := make([][]frontend.Variable, 4)
	for i := 0; i < 8; i++ {
		Input[i] = make([]frontend.Variable, 4)
		InputObj[i] = make([]frontend.Variable, 4)
		for j := 0; j < 4; j++ {
			Input[i][j] = frontend.Variable(0)
		}
	}
	for i := 0; i < 4; i++ {
		Weight[i] = make([]frontend.Variable, 4)
		WeightObj[i] = make([]frontend.Variable, 4)
		for j := 0; j < 4; j++ {
			Weight[i][j] = frontend.Variable(0)
		}
	}
	circuit := FVCircuit{Input: InputObj, Weight: WeightObj}
	witness := FVCircuit{Input: Input, Weight: Weight}
	// test.IsSolved(&circuit, &witness, ecc.BN254.ScalarField())
	assert := test.NewAssert(t)
	fmt.Println("here")
	assert.ProverSucceeded(&circuit, &witness, test.WithBackends(backend.PLONK), test.WithCurves(ecc.BN254), test.WithSolverOpts(solver.WithHints(FCHint, DivHintNew)))
	cs, _ := frontend.Compile(ecc.BN254.ScalarField(), scs.NewBuilder, &circuit)
	fmt.Println("Number of constraints: ", cs.GetNbConstraints())
	srs, _ := test.NewKZGSRS(cs)
	pK, vK, _ := plonk.Setup(cs, srs)
	wtns, _ := frontend.NewWitness(&witness, ecc.BN254.ScalarField())
	pubwtns, _ := wtns.Public()
	proof, _ := plonk.Prove(cs, pK, wtns, backend.WithSolverOptions(solver.WithHints(FCHint, DivHintNew)))
	err := plonk.Verify(proof, vK, pubwtns)
	if err != nil {
		panic(err)
	}
}

func TestDLRM_2(t *testing.T) {
	fmt.Println("Running DLRM Test")
	circuit := DLRMCircuitGen("circuit")
	witness := DLRMCircuitGen("witness")
	test.IsSolved(&circuit, &witness, ecc.BN254.ScalarField())
	assert := test.NewAssert(t)
	assert.ProverSucceeded(&circuit, &witness, test.WithBackends(backend.GROTH16), test.WithCurves(ecc.BN254), test.WithSolverOpts(solver.WithHints(FCHint, DivHintNew)))
	cs, _ := frontend.Compile(ecc.BN254.ScalarField(), r1cs.NewBuilder, &circuit)
	fmt.Println("Number of constraints: ", cs.GetNbConstraints())
}
