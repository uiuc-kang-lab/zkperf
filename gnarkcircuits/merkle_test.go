package gnarkcircuits

import (
	"crypto/rand"
	"fmt"
	"reflect"
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
	exp_len := []int{10, 100, 1000}
	for l := 0; l < len(exp_len); l++ {
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
		prover_start := time.Now()
		proof, _ := plonk.Prove(cs, pK, wtns)
		prover_time := time.Since(prover_start)
		proof_size := reflect.TypeOf(proof).Size()
		pubwtns, _ := wtns.Public()
		verifier_start := time.Now()
		err := plonk.Verify(proof, vK, pubwtns)
		verifier_time := time.Since(verifier_start)
		assert.Equal(err, nil)
		fmt.Println("Number of path elements: ", exp_len[l])
		fmt.Println("Number of constraints: ", cs.GetNbConstraints())
		fmt.Println("Prover time: ", prover_time)
		fmt.Println("Verifier time: ", verifier_time)
		fmt.Println("Proof size: ", proof_size, "B")
		var mem runtime.MemStats
		runtime.ReadMemStats(&mem)
		fmt.Println("Memory usage: ", mem.Sys/1024, "KB")
	}

	// assert.ProverSucceeded(circuit, witness, test.WithBackends(backend.PLONK), test.WithCurves(ecc.BN254))
}
