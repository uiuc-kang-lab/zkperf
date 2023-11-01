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

	hashFunc, _ := sha3.NewLegacyKeccak256(api) //Ref: https://github.com/Consensys/gnark/blob/36b0b58f02d0381774b24efba0a48032e5f794b4/std/hash/sha3/hashes.go#L60
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
