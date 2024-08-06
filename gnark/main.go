package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"gnark-benchmark/merkle"
	"gnark-benchmark/merkleposeidon"
	gnarkml "gnark-benchmark/ml"
	"os"
	"time"
)

// ProverResultFile defines the structure for storing prover results.
type ProverResultFile struct {
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

func main() {
	circuit, backend, curve, step, outputPath := get_args() // Provide same output path for prover and verifier
	var prover_time float64
	var verifier_time time.Duration
	var proof_size int64
	prover_time = 0.0
	verifier_time = 0
	proof_size = 0
	if circuit == "Invalid" {
		panic("Please provide a valid circuit name")
	} else if circuit == "MerkleTree" && step == "Setup" {
		merkle.RunMerkleSetup(backend, curve)
	} else if circuit == "MerkleTree" && step == "Prover" {
		prover_time, proof_size = merkle.RunMerkleProver(backend, curve)
	} else if circuit == "MerkleTree" && step == "Verifier" {
		verifier_time = merkle.RunMerkleVerifier(backend, curve)
	} else if circuit == "PoseidonMerkle" && step == "Setup" {
		merkleposeidon.RunMerkleSetup(backend, curve)
	} else if circuit == "PoseidonMerkle" && step == "Prover" {
		prover_time, proof_size = merkleposeidon.RunMerkleProver(backend, curve)
	} else if circuit == "PoseidonMerkle" && step == "Verifier" {
		verifier_time = merkleposeidon.RunMerkleVerifier(backend, curve)
	} else if circuit == "ECDSAVerification" {
		fmt.Println("Run ecdsa/ecdsa.sh")
	} else if circuit == "MNIST" && step == "Setup" {
		gnarkml.RunMNISTSetup(backend, curve)
	} else if circuit == "MNIST" && step == "Prover" {
		prover_time, proof_size = gnarkml.RunMNISTProver(backend, curve)
	} else if circuit == "MNIST" && step == "Verifier" {
		verifier_time = gnarkml.RunMNISTVerifier(backend, curve)
	} else if circuit == "DLRM" && step == "Setup" {
		gnarkml.RunDLRMSetup(backend, curve)
	} else if circuit == "DLRM" && step == "Prover" {
		prover_time, proof_size = gnarkml.RunDLRMProver(backend, curve)
	} else if circuit == "DLRM" && step == "Verifier" {
		verifier_time = gnarkml.RunDLRMVerifier(backend, curve)
	} else if step == "Test" {
		merkle.TestMerkle()
		gnarkml.TestMNIST()
		gnarkml.TestDLRM()
	} else {
		panic("Please provide a valid circuit name")
	}

	fmt.Println("Prover time: ", prover_time, "S")
	fmt.Println("Verifier time: ", float64(verifier_time.Nanoseconds())/1e6, "ms")
	fmt.Println("Proof size: ", proof_size, "B")

	if step == "Prover" {

		resultdata := ProverResultFile{
			Framework:  "gnark",
			Circuit:    circuit,
			Backend:    backend,
			Curve:      curve,
			Hardware:   "CPU",
			ProverTime: prover_time,
			ProofSize:  proof_size,
		}

		jsonfile, _ := json.MarshalIndent(resultdata, "", " ")
		_ = os.WriteFile(outputPath+".json", jsonfile, 0644)
	} else if step == "Verifier" {
		var newResults VerifierResultFile
		newResults.VerifierTime = float64(time.Duration(verifier_time.Nanoseconds())) / 1e6 // update after running verifier
		jsonfile, _ := json.MarshalIndent(newResults, "", " ")
		_ = os.WriteFile(outputPath+"_verifier.json", jsonfile, 0644)
	}
}

func check_err(err error) {
	if err != nil {
		panic(err)
	}
}

func get_args() (string, string, string, string, string) {
	var circuit string
	circuitHelp := "Provide the name circuit, options are: MerkleTree, ECDSAVerification, MNIST, DLRM"
	var backend string
	backendHelp := "Choose a backend Plonk/Groth16"
	var curve string
	curveHelp := "Only BN254 for the time being"
	var outputPath string
	outputPathHelp := "Provide the path to store the output"
	var step string
	stepHelp := "Prover/Verifier"
	currentTime := time.Now()
	flag.StringVar(&step, "step", "Prover", stepHelp)
	flag.StringVar(&circuit, "circuit", "Invalid", circuitHelp)
	flag.StringVar(&backend, "backend", "Plonk", backendHelp)
	flag.StringVar(&curve, "curve", "BN254", curveHelp)
	flag.StringVar(&outputPath, "outputPath", circuit+"_"+fmt.Sprint(currentTime.UnixNano()), outputPathHelp)
	flag.Parse()
	return circuit, backend, curve, step, outputPath
}
