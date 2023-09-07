package gnarkcircuits

import (
	"crypto/rand"
	"testing"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/backend"
	"github.com/consensys/gnark/std/math/uints"
	"github.com/consensys/gnark/test"
	"golang.org/x/crypto/sha3"
)

func TestMerkle(t *testing.T) {
	assert := test.NewAssert(t)

	leaf := make([]byte, 1) // Reference: https://github.com/Consensys/gnark/tree/master/std/hash/sha3
	_, err := rand.Reader.Read(leaf)
	assert.NoError(err)

	buf := make([]byte, 1)
	_, err = rand.Reader.Read(buf)
	assert.NoError(err)

	path := make([][]uints.U8, 5)
	path_object := make([][]uints.U8, 5)

	hashFunc := sha3.New256()
	hashFunc.Write(leaf)

	for i := 0; i < 5; i++ {
		buf := make([]byte, 1)
		_, err := rand.Reader.Read(buf)
		assert.NoError(err)
		path[i] = uints.NewU8Array(buf)
		path_object[i] = make([]uints.U8, 1)
		hashFunc.Write(buf)
	}
	digest := hashFunc.Sum(nil)

	circuit := &MerkleTreeCircuit{
		Leaf: make([]uints.U8, len(leaf)),
		Path: path_object,
		Root: make([]uints.U8, len(digest)),
	}
	witness := &MerkleTreeCircuit{
		Leaf: uints.NewU8Array(leaf),
		Path: path,
		Root: uints.NewU8Array(digest),
	}
	assert.ProverSucceeded(circuit, witness, test.WithBackends(backend.PLONK), test.WithCurves(ecc.BN254))
}
