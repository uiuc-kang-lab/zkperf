package merkleposeidon

import (
	"bytes"
	"fmt"
	"math/big"
	"os"
	"time"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/backend/groth16"
	"github.com/consensys/gnark/backend/plonk"
	"github.com/consensys/gnark/backend/witness"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/frontend/cs/r1cs"
	"github.com/consensys/gnark/frontend/cs/scs"
	"github.com/consensys/gnark/test"

	nonzkposeidon "github.com/iden3/go-iden3-crypto/poseidon"
)

func PoseidonMerkleCircuitGen(request string) PoseidonMerkleTree {
	path_int := []int64{1, 2, 3, 4, 5, 6, 7, 8, 9, 10}
	hashed_leaf, _ := nonzkposeidon.Hash([]*big.Int{big.NewInt(12)})
	path := make([]*big.Int, 10)
	path_x := make([]frontend.Variable, 10)
	// path_object := make([]frontend.Variable, 10)
	path[0], _ = nonzkposeidon.Hash([]*big.Int{big.NewInt(path_int[0])})
	path_x[0] = frontend.Variable(path[0])
	// path_object[0] = make([]uints.U8, len(path[0]))
	buf, _ := nonzkposeidon.Hash([]*big.Int{hashed_leaf, path[0]})

	for i := 1; i < 10; i++ {
		path[i], _ = nonzkposeidon.Hash([]*big.Int{big.NewInt(path_int[i])})
		// path_object[i] = make([]uints.U8, len(path[i]))
		// hashfunc := nonzksha3.NewLegacyKeccak256()
		// hashfunc.Write(Concat_byte(buf, hashElem(path_int[i])))
		path_x[i] = frontend.Variable(path[i])
		buf, _ = nonzkposeidon.Hash([]*big.Int{buf, path[i]})
	}
	digest := buf
	circuit := PoseidonMerkleTree{
		Leaf: frontend.Variable(hashed_leaf),
		Path: path_x,
		Root: frontend.Variable(digest),
	}
	witness := PoseidonMerkleTree{
		Leaf: frontend.Variable(hashed_leaf),
		Path: path_x,
		Root: frontend.Variable(digest),
	}
	// test.IsSolved(&circuit, &witness, ecc.BN254.ScalarField())
	var req PoseidonMerkleTree
	if request == "circuit" {
		req = circuit
	} else if request == "witness" {
		req = witness
	}
	return req
}

func GetPoseidonMerklePublicWitness() witness.Witness {
	witness := PoseidonMerkleCircuitGen("witness")
	wtns, _ := frontend.NewWitness(&witness, ecc.BN254.ScalarField())
	pub_wtns, _ := wtns.Public()
	return pub_wtns
}

func RunMerkleSetup(proofsystem string, curve string) {
	circuit := PoseidonMerkleCircuitGen("circuit")
	if proofsystem == "Groth16" {
		cs, _ := frontend.Compile(ecc.BN254.ScalarField(), r1cs.NewBuilder, &circuit)
		fmt.Println("Number of constraints: ", cs.GetNbConstraints())
		pK, vK, _ := groth16.Setup(cs)
		// serialize cs, pk and vk
		var ser_cs bytes.Buffer
		cs.WriteTo(&ser_cs)
		f, err := os.Create("poseidon_merkle_groth16_cs")
		check_err(err)
		_, err = f.Write(ser_cs.Bytes())
		check_err(err)
		f.Close()

		var ser_pk bytes.Buffer
		pK.WriteTo(&ser_pk)
		f, err = os.Create("poseidon_merkle_groth16_pk")
		check_err(err)
		_, err = f.Write(ser_pk.Bytes())
		check_err(err)
		f.Close()

		var ser_vk bytes.Buffer
		vK.WriteTo(&ser_vk)
		f, err = os.Create("poseidon_merkle_groth16_vk")
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
		f, err := os.Create("poseidon_merkle_plonk_cs")
		check_err(err)
		_, err = f.Write(ser_cs.Bytes())
		check_err(err)
		f.Close()

		var ser_pk bytes.Buffer
		pK.WriteTo(&ser_pk)
		f, err = os.Create("poseidon_merkle_plonk_pk")
		check_err(err)
		_, err = f.Write(ser_pk.Bytes())
		check_err(err)
		f.Close()

		var ser_vk bytes.Buffer
		vK.WriteTo(&ser_vk)
		f, err = os.Create("poseidon_merkle_plonk_vk")
		check_err(err)
		_, err = f.Write(ser_vk.Bytes())
		check_err(err)
		f.Close()
	}
}
func RunMerkleProver(proofsystem string, curve string) (float64, int64) {

	var prover_time time.Duration
	var proof_size int64
	witness := PoseidonMerkleCircuitGen("witness")
	if proofsystem == "Groth16" {
		wtns, _ := frontend.NewWitness(&witness, ecc.BN254.ScalarField())
		// deserialize cs and pK
		cs := groth16.NewCS(ecc.BN254)
		f, err := os.ReadFile("poseidon_merkle_groth16_cs")
		check_err(err)
		_, err = cs.ReadFrom(bytes.NewReader(f))
		check_err(err)

		pK := groth16.NewProvingKey(ecc.BN254)
		f, err = os.ReadFile("poseidon_merkle_groth16_pk")
		check_err(err)
		_, err = pK.ReadFrom(bytes.NewReader(f))
		check_err(err)

		prover_start := time.Now()
		proof, _ := groth16.Prove(cs, pK, wtns)
		prover_time = time.Since(prover_start)
		// serialize proof
		var ser_proof bytes.Buffer
		proof_size, _ = proof.WriteTo(&ser_proof)
		fdash, err := os.Create("poseidon_merkle_groth16_proof")
		check_err(err)

		_, err = fdash.Write(ser_proof.Bytes())
		check_err(err)
		fdash.Close()
	} else if proofsystem == "Plonk" {
		wtns, _ := frontend.NewWitness(&witness, ecc.BN254.ScalarField())
		// deserialize cs and pK
		cs := plonk.NewCS(ecc.BN254)
		f, err := os.ReadFile("poseidon_merkle_plonk_cs")
		check_err(err)
		_, err = cs.ReadFrom(bytes.NewReader(f))
		check_err(err)

		pK := plonk.NewProvingKey(ecc.BN254)
		f, err = os.ReadFile("poseidon_merkle_plonk_pk")
		check_err(err)
		_, err = pK.ReadFrom(bytes.NewReader(f))
		check_err(err)

		prover_start := time.Now()
		proof, _ := plonk.Prove(cs, pK, wtns)
		prover_time = time.Since(prover_start)
		// serialize proof
		var ser_proof bytes.Buffer
		proof_size, _ = proof.WriteTo(&ser_proof)
		fdash, err := os.Create("poseidon_merkle_plonk_proof")
		check_err(err)

		_, err = fdash.Write(ser_proof.Bytes())
		check_err(err)
		fdash.Close()
	}

	return prover_time.Seconds(), proof_size
}

func RunMerkleVerifier(proofsystem string, curve string) time.Duration {
	pub_wtns := GetPoseidonMerklePublicWitness()
	var verifier_time time.Duration
	if proofsystem == "Groth16" {
		proof := groth16.NewProof(ecc.BN254)
		vk := groth16.NewVerifyingKey(ecc.BN254)
		f, err := os.ReadFile("poseidon_merkle_groth16_proof")
		check_err(err)
		_, err = proof.ReadFrom(bytes.NewReader(f))
		check_err(err)

		f, err = os.ReadFile("poseidon_merkle_groth16_vk")
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
		f, err := os.ReadFile("poseidon_merkle_plonk_proof")
		check_err(err)
		_, err = proof.ReadFrom(bytes.NewReader(f))
		check_err(err)

		f, err = os.ReadFile("poseidon_merkle_plonk_vk")
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

func check_err(err error) {
	if err != nil {
		panic(err)
	}
}

func TestMerkle() {
	fmt.Println("Running Merkle Test")
	circuit := PoseidonMerkleCircuitGen("circuit")
	witness := PoseidonMerkleCircuitGen("witness")
	test.IsSolved(&circuit, &witness, ecc.BN254.ScalarField())
}
