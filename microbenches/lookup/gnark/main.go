package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"os"
	"strconv"
)

type ResultFile struct {
	Framework     string
	Backend       string
	TableSize     int
	NbQueries     int
	NbConstraints int
	ProverTime    float64
	VerifierTime  float64
	ProofSize     int
}

func main() {
	var backend string
	backendHelp := "Choose a backend Plonk/Groth16"
	flag.StringVar(&backend, "backend", "None", backendHelp)
	flag.Parse()

	fmt.Println("backend:", backend)
	n := []int{16383, 32767, 65535} // (2^k-1 for k=14,15,16)
	k := []int{1000, 10000, 100000, 1000000, 10000000}
	for i := 0; i < len(n); i++ {
		for j := 0; j < len(k); j++ {
			fmt.Println("n,k:", n[i], k[j])
			numConstraints := 0
			var prover_time float64
			var verifier_time float64
			var proof_size int
			LookupSetup(backend, n[i], k[j])
			numConstraints, prover_time, verifier_time, proof_size = RunLookup(backend, n[i], k[j])
			fmt.Println("numConstraints:", numConstraints)
			fmt.Println("prover_time:", prover_time)
			fmt.Println("verifier_time:", verifier_time)
			fmt.Println("proof_size:", proof_size)
			outputPath := backend + "_lookup" + strconv.Itoa(n[i]) + strconv.Itoa(k[j]) + ".json"
			resultdata := ResultFile{
				Framework:     "gnark",
				Backend:       backend,
				TableSize:     n[i],
				NbQueries:     k[j],
				NbConstraints: numConstraints,
				ProverTime:    prover_time,
				VerifierTime:  verifier_time,
				ProofSize:     proof_size,
			}

			jsonfile, _ := json.MarshalIndent(resultdata, "", " ")
			_ = os.WriteFile(outputPath, jsonfile, 0644)
		}
	}
}
