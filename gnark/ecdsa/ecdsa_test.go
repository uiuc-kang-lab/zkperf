package ecdsa

import (
	"bytes"
	"crypto/rand"
	"encoding/json"
	"fmt"
	"math"
	"math/big"
	"os"
	"testing"
	"time"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark-crypto/ecc/secp256k1/ecdsa"
	"github.com/consensys/gnark/backend/groth16"
	"github.com/consensys/gnark/backend/plonk"
	"github.com/consensys/gnark/backend/witness"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/frontend/cs/r1cs"
	"github.com/consensys/gnark/frontend/cs/scs"
	"github.com/consensys/gnark/std/math/emulated"
	gnarkecdsa "github.com/consensys/gnark/std/signature/ecdsa"
	"github.com/consensys/gnark/test"
)

type ResultFile struct {
	Framework  string
	Circuit    string
	Backend    string
	Curve      string
	Hardware   string
	ProverTime float64
	ProofSize  int64
}

type VerifierResultFile struct {
	VerifierTime float64
}

func ECDSACircuitGen(request string) ECDSACircuit {
	secret_key, e := ecdsa.GenerateKey(rand.Reader)
	check_err(e)
	m := make([]byte, int(math.Pow(2, 10)))
	fmt.Println("Message length: ", len(m))
	_, er := rand.Read(m)
	check_err(er)
	signatureBytes, e := secret_key.Sign(m, nil)
	check_err(e)
	var signature ecdsa.Signature // Reference: https://github.com/Consensys/gnark/blob/master/std/signature/ecdsa/ecdsa_test.go
	signature.SetBytes(signatureBytes)
	var r_bigInt, s_bigInt big.Int
	r_bigInt.SetBytes(signature.R[:32])
	s_bigInt.SetBytes(signature.S[:32])

	m_bigInt := ecdsa.HashToInt(m)

	// Objects for Instantiation
	var msg_object emulated.Element[emulated.Secp256k1Fr]
	var sig_object gnarkecdsa.Signature[emulated.Secp256k1Fr]
	var pk_object gnarkecdsa.PublicKey[emulated.Secp256k1Fp, emulated.Secp256k1Fr]

	ckt := ECDSACircuit{
		Message:   msg_object,
		Signature: sig_object,
		PublicKey: pk_object,
	}

	witness := ECDSACircuit{
		Message: emulated.ValueOf[emulated.Secp256k1Fr](m_bigInt),
		Signature: gnarkecdsa.Signature[emulated.Secp256k1Fr]{
			R: emulated.ValueOf[emulated.Secp256k1Fr](r_bigInt),
			S: emulated.ValueOf[emulated.Secp256k1Fr](s_bigInt),
		},
		PublicKey: gnarkecdsa.PublicKey[emulated.Secp256k1Fp, emulated.Secp256k1Fr]{
			X: emulated.ValueOf[emulated.Secp256k1Fp](secret_key.PublicKey.A.X),
			Y: emulated.ValueOf[emulated.Secp256k1Fp](secret_key.PublicKey.A.Y),
		},
	}
	var req ECDSACircuit
	if request == "circuit" {
		req = ckt
	}
	if request == "witness" {
		req = witness
	}
	return req
}

func ECDSAGroth16Setup() {
	circuit := ECDSACircuitGen("circuit")
	cs, _ := frontend.Compile(ecc.BN254.ScalarField(), r1cs.NewBuilder, &circuit)
	fmt.Println("Number of constraints: ", cs.GetNbConstraints())
	pK, vK, _ := groth16.Setup(cs)
	// serialize cs, pk and vk
	var ser_cs bytes.Buffer
	cs.WriteTo(&ser_cs)
	f, err := os.Create("ecdsa_groth16_cs")
	check_err(err)
	_, err = f.Write(ser_cs.Bytes())
	check_err(err)
	f.Close()

	var ser_pk bytes.Buffer
	pK.WriteTo(&ser_pk)
	f, err = os.Create("ecdsa_groth16_pk")
	check_err(err)
	_, err = f.Write(ser_pk.Bytes())
	check_err(err)
	f.Close()

	var ser_vk bytes.Buffer
	vK.WriteTo(&ser_vk)
	f, err = os.Create("ecdsa_groth16_vk")
	check_err(err)
	_, err = f.Write(ser_vk.Bytes())
	check_err(err)
	f.Close()
}

func ECDSAPlonkSetup() {
	circuit := ECDSACircuitGen("circuit")
	cs, _ := frontend.Compile(ecc.BN254.ScalarField(), scs.NewBuilder, &circuit)
	fmt.Println("Number of constraints: ", cs.GetNbConstraints())
	srs, _ := test.NewKZGSRS(cs)
	pK, vK, _ := plonk.Setup(cs, srs)
	// serialize cs, pk and vk
	var ser_cs bytes.Buffer
	cs.WriteTo(&ser_cs)
	f, err := os.Create("ecdsa_plonk_cs")
	check_err(err)
	_, err = f.Write(ser_cs.Bytes())
	check_err(err)
	f.Close()

	var ser_pk bytes.Buffer
	pK.WriteTo(&ser_pk)
	f, err = os.Create("ecdsa_plonk_pk")
	check_err(err)
	_, err = f.Write(ser_pk.Bytes())
	check_err(err)
	f.Close()

	var ser_vk bytes.Buffer
	vK.WriteTo(&ser_vk)
	f, err = os.Create("ecdsa_plonk_vk")
	check_err(err)
	_, err = f.Write(ser_vk.Bytes())
	check_err(err)
	f.Close()
}
func GetECDSAPublicWitness() witness.Witness {
	witness := ECDSACircuitGen("witness")
	wtns, _ := frontend.NewWitness(&witness, ecc.BN254.ScalarField())
	pub_wtns, _ := wtns.Public()
	return pub_wtns
}
func TestSetup(t *testing.T) {
	ECDSAGroth16Setup()
	ECDSAPlonkSetup()
}

func TestECDSAGroth16BN254(t *testing.T) {
	witness := ECDSACircuitGen("witness")
	wtns, _ := frontend.NewWitness(&witness, ecc.BN254.ScalarField())
	// deserialize cs and pK
	cs := groth16.NewCS(ecc.BN254)
	f, err := os.ReadFile("ecdsa_groth16_cs")
	check_err(err)
	_, err = cs.ReadFrom(bytes.NewReader(f))
	check_err(err)

	pK := groth16.NewProvingKey(ecc.BN254)
	f, err = os.ReadFile("ecdsa_groth16_pk")
	check_err(err)
	_, err = pK.ReadFrom(bytes.NewReader(f))
	check_err(err)

	prover_start := time.Now()
	proof, _ := groth16.Prove(cs, pK, wtns)
	prover_time := time.Since(prover_start).Seconds()
	ser_proof := new(bytes.Buffer)
	proof.WriteTo(ser_proof)
	proof_size := ser_proof.Len()
	fdash, err := os.Create("ecdsa_groth16_proof")
	check_err(err)
	_, err = fdash.Write(ser_proof.Bytes())
	check_err(err)
	fdash.Close()

	fmt.Println("Prover time: ", prover_time)
	fmt.Println("Proof size: ", proof_size, "B")

	resultdata := ResultFile{
		Framework:  "gnark",
		Circuit:    "ECDSAVerification",
		Backend:    "Groth16",
		Curve:      "BN254",
		Hardware:   "CPU",
		ProverTime: prover_time,
		ProofSize:  int64(proof_size),
	}

	jsonfile, _ := json.MarshalIndent(resultdata, "", " ")
	_ = os.WriteFile("ecdsa_groth16.json", jsonfile, 0644)
}

func TestECDSAGroth16Verifier(t *testing.T) {
	assert := test.NewAssert(t)
	pub_wtns := GetECDSAPublicWitness()
	proof := groth16.NewProof(ecc.BN254)
	vk := groth16.NewVerifyingKey(ecc.BN254)
	f, err := os.ReadFile("ecdsa_groth16_proof")
	check_err(err)
	_, err = proof.ReadFrom(bytes.NewReader(f))
	check_err(err)

	f, err = os.ReadFile("ecdsa_groth16_vk")
	check_err(err)
	_, err = vk.ReadFrom(bytes.NewReader(f))
	check_err(err)

	verifier_start := time.Now()
	err = groth16.Verify(proof, vk, pub_wtns)
	verifier_time := time.Since(verifier_start).Nanoseconds()
	assert.Equal(err, nil)
	fmt.Println("Verifier time: ", float64(verifier_time)/1e6)
	resultdata := VerifierResultFile{
		VerifierTime: float64(verifier_time) / 1e6,
	}

	jsonfile, _ := json.MarshalIndent(resultdata, "", " ")
	_ = os.WriteFile("ecdsa_groth16_verifier.json", jsonfile, 0644)

}

func TestECDSAPlonkBN254(t *testing.T) {
	witness := ECDSACircuitGen("witness")
	wtns, _ := frontend.NewWitness(&witness, ecc.BN254.ScalarField())
	// deserialize cs and pK
	cs := plonk.NewCS(ecc.BN254)
	f, err := os.ReadFile("ecdsa_plonk_cs")
	check_err(err)
	_, err = cs.ReadFrom(bytes.NewReader(f))
	check_err(err)

	pK := plonk.NewProvingKey(ecc.BN254)
	f, err = os.ReadFile("ecdsa_plonk_pk")
	check_err(err)
	_, err = pK.ReadFrom(bytes.NewReader(f))
	check_err(err)

	prover_start := time.Now()
	proof, _ := plonk.Prove(cs, pK, wtns)
	prover_time := time.Since(prover_start).Seconds()
	ser_proof := new(bytes.Buffer)
	proof.WriteTo(ser_proof)
	proof_size := ser_proof.Len()
	fdash, err := os.Create("ecdsa_plonk_proof")
	check_err(err)
	_, err = fdash.Write(ser_proof.Bytes())
	check_err(err)
	fdash.Close()

	fmt.Println("Prover time: ", prover_time)
	fmt.Println("Proof size: ", proof_size, "B")

	resultdata := ResultFile{
		Framework:  "gnark",
		Circuit:    "ECDSAVerification",
		Backend:    "Plonk",
		Curve:      "BN254",
		Hardware:   "CPU",
		ProverTime: prover_time,
		ProofSize:  int64(proof_size),
	}

	jsonfile, _ := json.MarshalIndent(resultdata, "", " ")
	_ = os.WriteFile("ecdsa_plonk.json", jsonfile, 0644)
}

func TestECDSAPlonkVerifier(t *testing.T) {
	assert := test.NewAssert(t)
	pub_wtns := GetECDSAPublicWitness()
	proof := plonk.NewProof(ecc.BN254)
	vk := plonk.NewVerifyingKey(ecc.BN254)
	f, err := os.ReadFile("ecdsa_plonk_proof")
	check_err(err)
	_, err = proof.ReadFrom(bytes.NewReader(f))
	check_err(err)

	f, err = os.ReadFile("ecdsa_plonk_vk")
	check_err(err)
	_, err = vk.ReadFrom(bytes.NewReader(f))
	check_err(err)

	verifier_start := time.Now()
	err = plonk.Verify(proof, vk, pub_wtns)
	verifier_time := time.Since(verifier_start).Nanoseconds()
	assert.Equal(err, nil)
	fmt.Println("Verifier time: ", float64(verifier_time)/1e6)
	resultdata := VerifierResultFile{
		VerifierTime: float64(verifier_time) / 1e6,
	}

	jsonfile, _ := json.MarshalIndent(resultdata, "", " ")
	_ = os.WriteFile("ecdsa_plonk_verifier.json", jsonfile, 0644)

}

func check_err(e error) {
	if e != nil {
		panic(e)
	}
}

func TestMockECDSA(t *testing.T) {
	circuit := ECDSACircuitGen("circuit")
	witness := ECDSACircuitGen("witness")
	test.IsSolved(&circuit, &witness, ecc.BN254.ScalarField())
}
