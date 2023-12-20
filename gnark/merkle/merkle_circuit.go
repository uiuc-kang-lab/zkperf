package merkle

import (
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/std/hash/sha3"
	"github.com/consensys/gnark/std/math/uints"
	nonzksha3 "golang.org/x/crypto/sha3"
)

type MerkleTreeCircuit struct {
	Leaf []uints.U8
	Path [][]uints.U8
	Root []uints.U8
}

func (circuit *MerkleTreeCircuit) Define(api frontend.API) error {

	hashFunc, _ := sha3.NewLegacyKeccak256(api) //Ref: https://github.com/Consensys/gnark/blob/36b0b58f02d0381774b24efba0a48032e5f794b4/std/hash/sha3/hashes.go#L60
	hashFunc.Write(Concat(circuit.Leaf, circuit.Path[0]))
	buf := hashFunc.Sum()
	for i := 1; i < len(circuit.Path); i++ {
		hashFunc, _ := sha3.NewLegacyKeccak256(api)
		hashFunc.Write(Concat(buf, circuit.Path[i]))
		buf = hashFunc.Sum()
	}
	for i := 0; i < len(circuit.Root); i++ {
		api.AssertIsEqual(circuit.Root[i].Val, buf[i].Val)
	}
	return nil
}

func Concat(a []uints.U8, b []uints.U8) []uints.U8 {
	c := make([]uints.U8, len(a)+len(b))
	copy(c[:len(a)], a)
	copy(c[len(a):], b)
	return c
}
func hashElem(in int) []byte {
	elemhashFunc := nonzksha3.NewLegacyKeccak256()
	elemhashFunc.Write([]byte{byte(in)})
	return elemhashFunc.Sum(nil)
}

func Concat_byte(a []byte, b []byte) []byte {
	c := make([]byte, len(a)+len(b))
	copy(c[:len(a)], a)
	copy(c[len(a):], b)
	return c
}
