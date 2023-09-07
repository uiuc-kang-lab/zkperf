package gnarkcircuits

import (
	"crypto/rand"
	"math/big"
	"testing"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark-crypto/ecc/secp256k1/ecdsa"
	"github.com/consensys/gnark/backend"
	"github.com/consensys/gnark/std/math/emulated"
	gnarkecdsa "github.com/consensys/gnark/std/signature/ecdsa"
	"github.com/consensys/gnark/test"
)

func TestECDSA(t *testing.T) {
	assert := test.NewAssert(t)
	secret_key, err := ecdsa.GenerateKey(rand.Reader)
	assert.NoError(err)

	m := []byte("UIUC Computer Science")

	signatureBytes, err := secret_key.Sign(m, nil)
	assert.NoError(err)

	var signature ecdsa.Signature // Reference: https://github.com/Consensys/gnark/blob/master/std/signature/ecdsa/ecdsa_test.go
	signature.SetBytes(signatureBytes)
	var r_bigInt, s_bigInt big.Int
	r_bigInt.SetBytes(signature.R[:32])
	s_bigInt.SetBytes(signature.S[:32])

	m_bigInt := ecdsa.HashToInt(m)

	// Objects for Instantiation
	var msg_object emulated.Element[emulated.Secp256k1Fr]
	var sig_object gnarkecdsa.Signature[emulated.Secp256k1Fr]
	var pk_object gnarkecdsa.PublicKey[emulated.Secp256k1Fp, emulated.Secp256k1Fr]

	circuit := &ECDSACircuit{
		Message:   msg_object,
		Signature: sig_object,
		PublicKey: pk_object,
	}

	witness := &ECDSACircuit{
		Message: emulated.ValueOf[emulated.Secp256k1Fr](m_bigInt),
		Signature: gnarkecdsa.Signature[emulated.Secp256k1Fr]{
			R: emulated.ValueOf[emulated.Secp256k1Fr](r_bigInt),
			S: emulated.ValueOf[emulated.Secp256k1Fr](s_bigInt),
		},
		PublicKey: gnarkecdsa.PublicKey[emulated.Secp256k1Fp, emulated.Secp256k1Fr]{
			X: emulated.ValueOf[emulated.Secp256k1Fp](secret_key.PublicKey.A.X),
			Y: emulated.ValueOf[emulated.Secp256k1Fp](secret_key.PublicKey.A.Y),
		},
	}

	assert.ProverSucceeded(circuit, witness, test.WithBackends(backend.PLONK), test.WithCurves(ecc.BN254))
}
