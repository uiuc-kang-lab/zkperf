package merkleposeidon

import (
	"github.com/consensys/gnark/frontend"
	poseidonhash "github.com/vocdoni/gnark-crypto-primitives/poseidon"
)

type PoseidonMerkleTree struct {
	Leaf frontend.Variable
	Path []frontend.Variable
	Root frontend.Variable
}

func (circuit *PoseidonMerkleTree) Define(api frontend.API) error {

	hashFunc := poseidonhash.NewPoseidon(api) //Ref: https://github.com/vocdoni/gnark-crypto-primitives/blob/main/poseidon/poseidon.go
	hashFunc.Write(circuit.Leaf, circuit.Path[0])
	buf := hashFunc.Sum()
	for i := 1; i < len(circuit.Path); i++ {
		hashFunc := poseidonhash.NewPoseidon(api)
		hashFunc.Write(buf, circuit.Path[i])
		buf = hashFunc.Sum()
	}
	api.AssertIsEqual(circuit.Root, buf)
	return nil
}
