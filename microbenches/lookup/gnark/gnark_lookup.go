package main

import (
	"bytes"
	"math/rand"
	"os"
	"time"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/backend/groth16"
	"github.com/consensys/gnark/backend/plonk"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/frontend/cs/r1cs"
	"github.com/consensys/gnark/frontend/cs/scs"
	"github.com/consensys/gnark/std/lookup/logderivlookup"
	"github.com/consensys/gnark/test"
)

type GnarkLookupCircuit struct {
	LookupTable []frontend.Variable
	Queries     []frontend.Variable
}

func (circuit *GnarkLookupCircuit) Define(api frontend.API) error {
	t := logderivlookup.New(api)
	for i := range circuit.LookupTable {
		t.Insert(circuit.LookupTable[i])
	}

	for i := range circuit.Queries {
		t.Lookup(circuit.Queries[i])
		// api.AssertIsEqual(buf[0], circuit.Queries[i])
	}
	return nil
}

func LookupSetup(proofsystem string, n int, k int) {
	circuit := LookupCircuitGen(n, k, "circuit")
	if proofsystem == "Groth16" {
		cs, _ := frontend.Compile(ecc.BN254.ScalarField(), r1cs.NewBuilder, &circuit)
		pK, vK, _ := groth16.Setup(cs)
		// serialize cs, pk and vk
		var ser_cs bytes.Buffer
		cs.WriteTo(&ser_cs)
		f, err := os.Create("lookup_groth16_cs")
		check_err(err)
		_, err = f.Write(ser_cs.Bytes())
		check_err(err)
		f.Close()

		var ser_pk bytes.Buffer
		pK.WriteTo(&ser_pk)
		f, err = os.Create("lookup_groth16_pk")
		check_err(err)
		_, err = f.Write(ser_pk.Bytes())
		check_err(err)
		f.Close()

		var ser_vk bytes.Buffer
		vK.WriteTo(&ser_vk)
		f, err = os.Create("lookup_groth16_vk")
		check_err(err)
		_, err = f.Write(ser_vk.Bytes())
		check_err(err)
		f.Close()
	} else if proofsystem == "Plonk" {
		cs, _ := frontend.Compile(ecc.BN254.ScalarField(), scs.NewBuilder, &circuit)
		srs, _ := test.NewKZGSRS(cs)
		pK, vK, _ := plonk.Setup(cs, srs)
		// serialize cs, pk and vk
		var ser_cs bytes.Buffer
		cs.WriteTo(&ser_cs)
		f, err := os.Create("lookup_plonk_cs")
		check_err(err)
		_, err = f.Write(ser_cs.Bytes())
		check_err(err)
		f.Close()

		var ser_pk bytes.Buffer
		pK.WriteTo(&ser_pk)
		f, err = os.Create("lookup_plonk_pk")
		check_err(err)
		_, err = f.Write(ser_pk.Bytes())
		check_err(err)
		f.Close()

		var ser_vk bytes.Buffer
		vK.WriteTo(&ser_vk)
		f, err = os.Create("lookup_plonk_vk")
		check_err(err)
		_, err = f.Write(ser_vk.Bytes())
		check_err(err)
		f.Close()
	}
}

func RunLookup(proofsystem string, n int, k int) (int, float64, float64, int) { // n: table size, k: number of queries
	witness := LookupCircuitGen(n, k, "witness")
	wtns, _ := frontend.NewWitness(&witness, ecc.BN254.ScalarField())
	var prover_time time.Duration
	var verifier_time time.Duration
	var proof_size int
	var NbConstraints int
	if proofsystem == "Groth16" {
		cs := groth16.NewCS(ecc.BN254)
		f, err := os.ReadFile("lookup_groth16_cs")
		check_err(err)
		_, err = cs.ReadFrom(bytes.NewReader(f))
		check_err(err)

		pK := groth16.NewProvingKey(ecc.BN254)
		f, err = os.ReadFile("lookup_groth16_pk")
		check_err(err)
		_, err = pK.ReadFrom(bytes.NewReader(f))
		check_err(err)

		vK := groth16.NewVerifyingKey(ecc.BN254)
		f, err = os.ReadFile("lookup_groth16_vk")
		check_err(err)
		_, err = vK.ReadFrom(bytes.NewReader(f))
		check_err(err)

		prover_start := time.Now()
		proof, _ := groth16.Prove(cs, pK, wtns)
		prover_time = time.Since(prover_start)
		ser_proof := new(bytes.Buffer)
		proof.WriteTo(ser_proof)
		proof_size = ser_proof.Len()
		pub_wtns, _ := wtns.Public()
		verifier_start := time.Now()
		err = groth16.Verify(proof, vK, pub_wtns)
		if err != nil {
			panic(err)
		}
		verifier_time = time.Since(verifier_start)
		NbConstraints = cs.GetNbConstraints()
	} else if proofsystem == "Plonk" {
		cs := plonk.NewCS(ecc.BN254)
		f, err := os.ReadFile("lookup_plonk_cs")
		check_err(err)
		_, err = cs.ReadFrom(bytes.NewReader(f))
		check_err(err)

		pK := plonk.NewProvingKey(ecc.BN254)
		f, err = os.ReadFile("lookup_plonk_pk")
		check_err(err)
		_, err = pK.ReadFrom(bytes.NewReader(f))
		check_err(err)

		vK := plonk.NewVerifyingKey(ecc.BN254)
		f, err = os.ReadFile("lookup_plonk_vk")
		check_err(err)
		_, err = vK.ReadFrom(bytes.NewReader(f))
		check_err(err)

		prover_start := time.Now()
		proof, _ := plonk.Prove(cs, pK, wtns)
		prover_time = time.Since(prover_start)
		ser_proof := new(bytes.Buffer)
		proof.WriteTo(ser_proof)
		proof_size = ser_proof.Len()
		pub_wtns, _ := wtns.Public()
		verifier_start := time.Now()
		err = plonk.Verify(proof, vK, pub_wtns)
		if err != nil {
			panic(err)
		}
		verifier_time = time.Since(verifier_start)
		NbConstraints = cs.GetNbConstraints()
	}
	return NbConstraints, prover_time.Seconds(), float64(time.Duration(verifier_time.Nanoseconds())) / 1e6, proof_size
}

func LookupCircuitGen(n int, k int, request string) GnarkLookupCircuit {
	loookuptable := make([]frontend.Variable, n)
	lookuptableObj := make([]frontend.Variable, n)
	for i := 0; i < n; i++ {
		loookuptable[i] = frontend.Variable(i)
	}
	queries := make([]frontend.Variable, k)
	queriesObj := make([]frontend.Variable, k)
	for i := 0; i < k; i++ {
		buf := rand.Intn(n) // random number between 0 and n-1
		queries[i] = frontend.Variable(buf)
	}

	circuit := GnarkLookupCircuit{lookuptableObj, queriesObj}
	witness := GnarkLookupCircuit{loookuptable, queries}
	// test.IsSolved(&circuit, &witness, ecc.BN254.ScalarField())
	var req GnarkLookupCircuit
	if request == "circuit" {
		req = circuit
	}
	if request == "witness" {
		req = witness
	}
	return req
}

func check_err(err error) {
	if err != nil {
		panic(err)
	}
}
