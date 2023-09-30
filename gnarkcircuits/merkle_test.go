package gnarkcircuits

import (
	"bytes"
	"crypto/rand"
	"fmt"
	"math"
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

	exp_len := []int{2, int(math.Pow(2, 3)), int(math.Pow(2, 6)), int(math.Pow(2, 10))}
	for l := 0; l < len(exp_len); l++ {
		fmt.Println("Checkpoint1")
		results := make([]map[string]int, 5)
		mean_prover_time := 0
		mean_verifier_time := 0
		mean_proof_size := 0
		mean_memory_usage := 0
		for t := 0; t < 5; t++ {
			fmt.Println("Checkpoint2")
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
			fmt.Println("Checkpoint3")
			result := make(map[string]int) // initialize the map
			prover_start := time.Now()
			proof, _ := plonk.Prove(cs, pK, wtns)
			fmt.Println("Checkpoint4")
			prover_time := time.Since(prover_start)
			ser_proof := new(bytes.Buffer)
			proof.WriteTo(ser_proof)
			proof_size := ser_proof.Len()
			pubwtns, _ := wtns.Public()
			verifier_start := time.Now()
			err := plonk.Verify(proof, vK, pubwtns)
			verifier_time := time.Since(verifier_start)
			fmt.Println("Checkpoint5")
			assert.Equal(err, nil)
			result["NumElements"] = exp_len[l]
			result["NumConstraints"] = cs.GetNbConstraints()
			result["ProverTime"] = int(prover_time.Microseconds())
			result["VerifierTime"] = int(verifier_time.Microseconds())
			result["ProofSize"] = proof_size
			var mem runtime.MemStats
			runtime.ReadMemStats(&mem)
			result["MemoryUsage"] = int(mem.Sys / 1024)
			results[t] = result
			mean_prover_time += results[t]["ProverTime"]
			mean_verifier_time += results[t]["VerifierTime"]
			mean_proof_size += results[t]["ProofSize"]
			mean_memory_usage += results[t]["MemoryUsage"]
		}
		fmt.Println("Checkpoint6")
		fmt.Println("Number of path elements: ", exp_len[l])
		fmt.Println("Number of constraints: ", results[0]["NumConstraints"])
		fmt.Println("Prover time: ", mean_prover_time/5, "µs")
		fmt.Println("Verifier time: ", mean_verifier_time/5, "µs")
		fmt.Println("Proof size: ", mean_proof_size/5, "B")
		fmt.Println("Memory usage: ", mean_memory_usage/5, "KB")
	}
	// assert.ProverSucceeded(circuit, witness, test.WithBackends(backend.PLONK), test.WithCurves(ecc.BN254))
}
