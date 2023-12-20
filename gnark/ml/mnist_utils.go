package gnarkml

import (
	"bytes"
	"encoding/json"
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

func MNISTCircuitGen(request string) MNISTCircuit {
	img, err := os.ReadFile("ml/mnist_inp.json")
	check_err(err)
	var mnistImage []Tensor
	err = json.Unmarshal(img, &mnistImage)
	check_err(err)

	ImageObj := make([][][]frontend.Variable, 28)
	Image := make([][][]frontend.Variable, 28)
	index := 0
	for i := 0; i < 28; i++ {
		ImageObj[i] = make([][]frontend.Variable, 28)
		Image[i] = make([][]frontend.Variable, 28)
		for j := 0; j < 28; j++ {
			ImageObj[i][j] = make([]frontend.Variable, 1)
			Image[i][j] = make([]frontend.Variable, 1)
			Image[i][j][0] = frontend.Variable(mnistImage[0].Data[index])
			index += 1
		}
	}

	circuit := MNISTCircuit{
		Image: ImageObj,
	}

	witness := MNISTCircuit{
		Image: Image,
	}

	var req MNISTCircuit
	if request == "circuit" {
		req = circuit
	} else if request == "witness" {
		req = witness
	}
	return req
}

func GetMNISTPublicWitness() witness.Witness {
	witness := MNISTCircuitGen("witness")
	wtns, _ := frontend.NewWitness(&witness, ecc.BN254.ScalarField())
	pub_wtns, _ := wtns.Public()
	return pub_wtns
}

func RunMNISTSetup(proofsystem string, curve string) {
	circuit := MNISTCircuitGen("circuit")
	if proofsystem == "Groth16" {
		cs, _ := frontend.Compile(ecc.BN254.ScalarField(), r1cs.NewBuilder, &circuit)
		fmt.Println("Number of constraints: ", cs.GetNbConstraints())
		pK, vK, _ := groth16.Setup(cs)
		// serialize cs, pk and vk
		var ser_cs bytes.Buffer
		cs.WriteTo(&ser_cs)
		f, err := os.Create("mnist_groth16_cs")
		check_err(err)
		_, err = f.Write(ser_cs.Bytes())
		check_err(err)
		f.Close()

		var ser_pk bytes.Buffer
		pK.WriteTo(&ser_pk)
		f, err = os.Create("mnist_groth16_pk")
		check_err(err)
		_, err = f.Write(ser_pk.Bytes())
		check_err(err)
		f.Close()

		var ser_vk bytes.Buffer
		vK.WriteTo(&ser_vk)
		f, err = os.Create("mnist_groth16_vk")
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
		f, err := os.Create("mnist_plonk_cs")
		check_err(err)
		_, err = f.Write(ser_cs.Bytes())
		check_err(err)
		f.Close()

		var ser_pk bytes.Buffer
		pK.WriteTo(&ser_pk)
		f, err = os.Create("mnist_plonk_pk")
		check_err(err)
		_, err = f.Write(ser_pk.Bytes())
		check_err(err)
		f.Close()

		var ser_vk bytes.Buffer
		vK.WriteTo(&ser_vk)
		f, err = os.Create("mnist_plonk_vk")
		check_err(err)
		_, err = f.Write(ser_vk.Bytes())
		check_err(err)
		f.Close()
	}
}
func RunMNISTProver(proofsystem string, curve string) (float64, int64) {

	var prover_time time.Duration
	var proof_size int64
	witness := MNISTCircuitGen("witness")
	if proofsystem == "Groth16" {
		wtns, _ := frontend.NewWitness(&witness, ecc.BN254.ScalarField())
		// deserialize cs and pK
		cs := groth16.NewCS(ecc.BN254)
		f, err := os.ReadFile("mnist_groth16_cs")
		check_err(err)
		_, err = cs.ReadFrom(bytes.NewReader(f))
		check_err(err)

		pK := groth16.NewProvingKey(ecc.BN254)
		f, err = os.ReadFile("mnist_groth16_pk")
		check_err(err)
		_, err = pK.ReadFrom(bytes.NewReader(f))
		check_err(err)

		prover_start := time.Now()
		proof, _ := groth16.Prove(cs, pK, wtns, backend.WithSolverOptions(solver.WithHints(DivHint, DivHintNew)))
		prover_time = time.Since(prover_start)
		// serialize proof
		var ser_proof bytes.Buffer
		proof_size, _ = proof.WriteTo(&ser_proof)
		fdash, err := os.Create("mnist_groth16_proof")
		check_err(err)

		_, err = fdash.Write(ser_proof.Bytes())
		check_err(err)
		fdash.Close()
	} else if proofsystem == "Plonk" {
		wtns, _ := frontend.NewWitness(&witness, ecc.BN254.ScalarField())
		// deserialize cs and pK
		cs := plonk.NewCS(ecc.BN254)
		f, err := os.ReadFile("mnist_plonk_cs")
		check_err(err)
		_, err = cs.ReadFrom(bytes.NewReader(f))
		check_err(err)

		pK := plonk.NewProvingKey(ecc.BN254)
		f, err = os.ReadFile("mnist_plonk_pk")
		check_err(err)
		_, err = pK.ReadFrom(bytes.NewReader(f))
		check_err(err)

		prover_start := time.Now()
		proof, _ := plonk.Prove(cs, pK, wtns, backend.WithSolverOptions(solver.WithHints(DivHint, DivHintNew)))
		prover_time = time.Since(prover_start)
		// serialize proof
		var ser_proof bytes.Buffer
		proof_size, _ = proof.WriteTo(&ser_proof)
		fdash, err := os.Create("mnist_plonk_proof")
		check_err(err)

		_, err = fdash.Write(ser_proof.Bytes())
		check_err(err)
		fdash.Close()
	}

	return prover_time.Seconds(), proof_size
}

func RunMNISTVerifier(proofsystem string, curve string) time.Duration {
	pub_wtns := GetMNISTPublicWitness()
	var verifier_time time.Duration
	if proofsystem == "Groth16" {
		proof := groth16.NewProof(ecc.BN254)
		vk := groth16.NewVerifyingKey(ecc.BN254)
		f, err := os.ReadFile("mnist_groth16_proof")
		check_err(err)
		_, err = proof.ReadFrom(bytes.NewReader(f))
		check_err(err)

		f, err = os.ReadFile("mnist_groth16_vk")
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
		f, err := os.ReadFile("mnist_plonk_proof")
		check_err(err)
		_, err = proof.ReadFrom(bytes.NewReader(f))
		check_err(err)

		f, err = os.ReadFile("mnist_plonk_vk")
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

func TestMNIST() {
	fmt.Println("Running MNIST Test")
	circuit := MNISTCircuitGen("circuit")
	witness := MNISTCircuitGen("witness")
	test.IsSolved(&circuit, &witness, ecc.BN254.ScalarField())
}
