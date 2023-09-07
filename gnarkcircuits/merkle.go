package gnarkcircuits

import (
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/std/hash/sha3"
	"github.com/consensys/gnark/std/math/uints"
)

type MerkleTreeCircuit struct {
	Leaf []uints.U8
	Path [][]uints.U8
	Root []uints.U8
}

func (circuit *MerkleTreeCircuit) Define(api frontend.API) error {

	hashFunc, _ := sha3.New256(api)
	hashFunc.Write(circuit.Leaf)
	for i := 0; i < len(circuit.Path); i++ {
		hashFunc.Write(circuit.Path[i])
	}

	digest := hashFunc.Sum()

	for i := range digest {
		api.AssertIsEqual(digest[i].Val, circuit.Root[i].Val)
	}
	return nil
}
