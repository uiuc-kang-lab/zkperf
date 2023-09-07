package gnarkcircuits

import (
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/std/algebra/emulated/sw_emulated"
	"github.com/consensys/gnark/std/math/emulated"
	gnarkecdsa "github.com/consensys/gnark/std/signature/ecdsa"
)

// emulated.Secp256k1Fp: Base Field
// emulated.Secp256k1Fr: Scalar Field

type ECDSACircuit struct {
	Message   emulated.Element[emulated.Secp256k1Fr]
	Signature gnarkecdsa.Signature[emulated.Secp256k1Fr]
	PublicKey gnarkecdsa.PublicKey[emulated.Secp256k1Fp, emulated.Secp256k1Fr]
}

func (circuit *ECDSACircuit) Define(api frontend.API) error {

	circuit.PublicKey.Verify(api, sw_emulated.GetSecp256k1Params(), &circuit.Message, &circuit.Signature)
	return nil
}
