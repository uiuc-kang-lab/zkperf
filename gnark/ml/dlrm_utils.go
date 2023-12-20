package gnarkml

import (
	"bytes"
	"fmt"
	"os"
	"time"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/backend"
	"github.com/consensys/gnark/backend/groth16"
	"github.com/consensys/gnark/backend/plonk"
	"github.com/consensys/gnark/backend/witness"
	"github.com/consensys/gnark/constraint/solver"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/frontend/cs/r1cs"
	"github.com/consensys/gnark/frontend/cs/scs"
	"github.com/consensys/gnark/test"
)

func DLRMCircuitGen(request string) DLRMCircuit {
	DenseObj := make([][][]frontend.Variable, 26)
	Dense := make([][][]frontend.Variable, 26)
	Dense_int := make([][][]int, 26)
	for i := 0; i < 26; i++ {
		DenseObj[i] = make([][]frontend.Variable, 1)
		Dense[i] = make([][]frontend.Variable, 1)
		Dense_int[i] = make([][]int, 1)
		for j := 0; j < 1; j++ {
			DenseObj[i][j] = make([]frontend.Variable, 64)
			Dense[i][j] = make([]frontend.Variable, 64)
			Dense_int[i][j] = make([]int, 64)
			for k := 0; k < 64; k++ {
				Dense[i][j][k] = frontend.Variable(0)
				Dense_int[i][j][k] = 0
			}
		}
	}
	SparseObj := make([][]frontend.Variable, 1)
	Sparse := make([][]frontend.Variable, 1)
	Sparse_int := make([][]int, 1)
	for i := 0; i < 1; i++ {
		SparseObj[i] = make([]frontend.Variable, 13)
		Sparse[i] = make([]frontend.Variable, 13)
		Sparse_int[i] = make([]int, 13)
		for j := 0; j < 13; j++ {
			Sparse[i][j] = frontend.Variable(0)
			Sparse_int[i][j] = 0
		}
	}

	circuit := DLRMCircuit{
		DenseLayers: DenseObj,
		SparseLayer: SparseObj,
	}

	witness := DLRMCircuit{
		DenseLayers: Dense,
		SparseLayer: Sparse,
	}

	var req DLRMCircuit
	if request == "circuit" {
		req = circuit
	} else if request == "witness" {
		req = witness
	}
	return req
}

func GetDLRMPublicWitness() witness.Witness {
	witness := DLRMCircuitGen("witness")
	wtns, _ := frontend.NewWitness(&witness, ecc.BN254.ScalarField())
	pub_wtns, _ := wtns.Public()
	return pub_wtns
}

func RunDLRMSetup(proofsystem string, curve string) {
	circuit := DLRMCircuitGen("circuit")
	if proofsystem == "Groth16" {
		cs, _ := frontend.Compile(ecc.BN254.ScalarField(), r1cs.NewBuilder, &circuit)
		fmt.Println("Number of constraints: ", cs.GetNbConstraints())
		pK, vK, _ := groth16.Setup(cs)
		// serialize cs, pk and vk
		var ser_cs bytes.Buffer
		cs.WriteTo(&ser_cs)
		f, err := os.Create("dlrm_groth16_cs")
		check_err(err)
		_, err = f.Write(ser_cs.Bytes())
		check_err(err)
		f.Close()

		var ser_pk bytes.Buffer
		pK.WriteTo(&ser_pk)
		f, err = os.Create("dlrm_groth16_pk")
		check_err(err)
		_, err = f.Write(ser_pk.Bytes())
		check_err(err)
		f.Close()

		var ser_vk bytes.Buffer
		vK.WriteTo(&ser_vk)
		f, err = os.Create("dlrm_groth16_vk")
		check_err(err)
		_, err = f.Write(ser_vk.Bytes())
		check_err(err)
		f.Close()
	} else if proofsystem == "Plonk" {
		cs, _ := frontend.Compile(ecc.BN254.ScalarField(), scs.NewBuilder, &circuit)
		fmt.Println("Number of constraints: ", cs.GetNbConstraints())
		srs, _ := test.NewKZGSRS(cs)
		pK, vK, _ := plonk.Setup(cs, srs)
		// serialize cs, pk and vk
		var ser_cs bytes.Buffer
		cs.WriteTo(&ser_cs)
		f, err := os.Create("dlrm_plonk_cs")
		check_err(err)
		_, err = f.Write(ser_cs.Bytes())
		check_err(err)
		f.Close()

		var ser_pk bytes.Buffer
		pK.WriteTo(&ser_pk)
		f, err = os.Create("dlrm_plonk_pk")
		check_err(err)
		_, err = f.Write(ser_pk.Bytes())
		check_err(err)
		f.Close()

		var ser_vk bytes.Buffer
		vK.WriteTo(&ser_vk)
		f, err = os.Create("dlrm_plonk_vk")
		check_err(err)
		_, err = f.Write(ser_vk.Bytes())
		check_err(err)
		f.Close()
	}
}
func RunDLRMProver(proofsystem string, curve string) (float64, int64) {
	var prover_time time.Duration
	var proof_size int64
	witness := DLRMCircuitGen("witness")
	if proofsystem == "Groth16" {
		wtns, _ := frontend.NewWitness(&witness, ecc.BN254.ScalarField())
		// deserialize cs and pK
		cs := groth16.NewCS(ecc.BN254)
		f, err := os.ReadFile("dlrm_groth16_cs")
		check_err(err)
		_, err = cs.ReadFrom(bytes.NewReader(f))
		check_err(err)

		pK := groth16.NewProvingKey(ecc.BN254)
		f, err = os.ReadFile("dlrm_groth16_pk")
		check_err(err)
		_, err = pK.ReadFrom(bytes.NewReader(f))
		check_err(err)

		prover_start := time.Now()
		proof, _ := groth16.Prove(cs, pK, wtns, backend.WithSolverOptions(solver.WithHints(DivHint, DivHintNew)))
		prover_time = time.Since(prover_start)
		// serialize proof
		var ser_proof bytes.Buffer
		proof_size, _ = proof.WriteTo(&ser_proof)
		fdash, err := os.Create("dlrm_groth16_proof")
		check_err(err)

		_, err = fdash.Write(ser_proof.Bytes())
		check_err(err)
		fdash.Close()
	} else if proofsystem == "Plonk" {
		wtns, _ := frontend.NewWitness(&witness, ecc.BN254.ScalarField())
		// deserialize cs and pK
		cs := plonk.NewCS(ecc.BN254)
		f, err := os.ReadFile("dlrm_plonk_cs")
		check_err(err)
		_, err = cs.ReadFrom(bytes.NewReader(f))
		check_err(err)

		pK := plonk.NewProvingKey(ecc.BN254)
		f, err = os.ReadFile("dlrm_plonk_pk")
		check_err(err)
		_, err = pK.ReadFrom(bytes.NewReader(f))
		check_err(err)

		prover_start := time.Now()
		proof, _ := plonk.Prove(cs, pK, wtns, backend.WithSolverOptions(solver.WithHints(DivHint, DivHintNew)))
		prover_time = time.Since(prover_start)
		// serialize proof
		var ser_proof bytes.Buffer
		proof_size, _ = proof.WriteTo(&ser_proof)
		fdash, err := os.Create("dlrm_plonk_proof")
		check_err(err)

		_, err = fdash.Write(ser_proof.Bytes())
		check_err(err)
		fdash.Close()
	}

	return prover_time.Seconds(), proof_size
}

func RunDLRMVerifier(proofsystem string, curve string) time.Duration {
	pub_wtns := GetDLRMPublicWitness()
	var verifier_time time.Duration
	if proofsystem == "Groth16" {
		proof := groth16.NewProof(ecc.BN254)
		vk := groth16.NewVerifyingKey(ecc.BN254)
		f, err := os.ReadFile("dlrm_groth16_proof")
		check_err(err)
		_, err = proof.ReadFrom(bytes.NewReader(f))
		check_err(err)

		f, err = os.ReadFile("dlrm_groth16_vk")
		check_err(err)
		_, err = vk.ReadFrom(bytes.NewReader(f))
		check_err(err)

		verifier_start := time.Now()
		err = groth16.Verify(proof, vk, pub_wtns)
		check_err(err)
		verifier_time = time.Since(verifier_start)
	} else if proofsystem == "Plonk" {
		proof := plonk.NewProof(ecc.BN254)
		vk := plonk.NewVerifyingKey(ecc.BN254)
		f, err := os.ReadFile("dlrm_plonk_proof")
		check_err(err)
		_, err = proof.ReadFrom(bytes.NewReader(f))
		check_err(err)

		f, err = os.ReadFile("dlrm_plonk_vk")
		check_err(err)
		_, err = vk.ReadFrom(bytes.NewReader(f))
		check_err(err)

		verifier_start := time.Now()
		err = plonk.Verify(proof, vk, pub_wtns)
		check_err(err)
		verifier_time = time.Since(verifier_start)
	}
	return verifier_time
}

func TestDLRM() {
	fmt.Println("Running DLRM Test")
	circuit := DLRMCircuitGen("circuit")
	witness := DLRMCircuitGen("witness")
	test.IsSolved(&circuit, &witness, ecc.BN254.ScalarField())
}
