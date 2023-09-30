package gnarkcircuits

import (
	"bytes"
	"crypto/rand"
	"encoding/csv"
	"fmt"
	"log"
	"os"
	"runtime"
	"testing"
	"time"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/backend/plonk"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/frontend/cs/scs"
	"github.com/consensys/gnark/std/math/uints"
	"github.com/consensys/gnark/test"
	"golang.org/x/crypto/sha3"
)

func TestMerkle(t *testing.T) {
	assert := test.NewAssert(t)
	exp_len := []int{2, 2 ^ 3, 2 ^ 6, 2 ^ 9, 2 ^ 12}
	for l := 0; l < len(exp_len); l++ {
		for t := 0; t < 5; t++ {
			results := make([]map[string]int, 5)

			leaf := make([]byte, 1) // Reference: https://github.com/Consensys/gnark/tree/master/std/hash/sha3
			_, e := rand.Reader.Read(leaf)
			assert.NoError(e)

			path := make([][]uints.U8, exp_len[l])
			path_object := make([][]uints.U8, exp_len[l])

			hashFunc := sha3.New256()
			hashFunc.Write(leaf)

			for i := 0; i < exp_len[l]; i++ {
				buf := make([]byte, 1)
				_, err := rand.Reader.Read(buf)
				assert.NoError(err)
				path[i] = uints.NewU8Array(buf)
				path_object[i] = make([]uints.U8, 1)
				hashFunc.Write(buf)
			}
			digest := hashFunc.Sum(nil)

			ckt := MerkleTreeCircuit{
				Leaf: make([]uints.U8, len(leaf)),
				Path: path_object,
				Root: make([]uints.U8, len(digest)),
			}
			witness := MerkleTreeCircuit{
				Leaf: uints.NewU8Array(leaf),
				Path: path,
				Root: uints.NewU8Array(digest),
			}
			wtns, _ := frontend.NewWitness(&witness, ecc.BN254.ScalarField())
			cs, _ := frontend.Compile(ecc.BN254.ScalarField(), scs.NewBuilder, &ckt)
			srs, _ := test.NewKZGSRS(cs)
			pK, vK, _ := plonk.Setup(cs, srs)
			result := make(map[string]int) // initialize the map
			prover_start := time.Now()
			proof, _ := plonk.Prove(cs, pK, wtns)
			prover_time := time.Since(prover_start)
			ser_proof := new(bytes.Buffer)
			proof.WriteTo(ser_proof)
			proof_size := ser_proof.Len()
			pubwtns, _ := wtns.Public()
			verifier_start := time.Now()
			err := plonk.Verify(proof, vK, pubwtns)
			verifier_time := time.Since(verifier_start)
			assert.Equal(err, nil)
			result["NumElements"] = exp_len[l]
			result[""] = cs.GetNbConstraints()
			result["ProverTime"] = int(prover_time.Microseconds())
			result["VerifierTime"] = int(verifier_time.Microseconds())
			result["ProofSize"] = proof_size
			var mem runtime.MemStats
			runtime.ReadMemStats(&mem)
			result["MemoryUsage"] = int(mem.Sys / 1024)
			results[t] = result
			mean_prover_time := 0
			mean_verifier_time := 0
			mean_proof_size := 0
			mean_memory_usage := 0
			for i := 0; i < 5; i++ {
				mean_prover_time += results[i]["ProverTime"]
				mean_verifier_time += results[i]["VerifierTime"]
				mean_proof_size += results[i]["ProofSize"]
				mean_memory_usage += results[i]["MemoryUsage"]
			}

			file, err := os.Create("results.csv")
			if err != nil {
				log.Fatal(err)
			}
			defer file.Close()

			w := csv.NewWriter(file)
			for _, recordset := range results {
				for key, value := range recordset {
					err := w.Write([]string{fmt.Sprintf("%v", key), fmt.Sprintf("%v", value)})
					if err != nil {
						panic(err)
					}
				}
			}

			w.Flush()
			fmt.Println("Number of path elements: ", exp_len[l])
			fmt.Println("Number of constraints: ", results[0]["NumConstraints"])
			fmt.Println("Prover time: ", mean_prover_time, "µs")
			fmt.Println("Verifier time: ", mean_verifier_time, "µs")
			fmt.Println("Proof size: ", mean_proof_size, "B")
			fmt.Println("Memory usage: ", mean_memory_usage, "KB")
		}
	}
	// assert.ProverSucceeded(circuit, witness, test.WithBackends(backend.PLONK), test.WithCurves(ecc.BN254))
}
