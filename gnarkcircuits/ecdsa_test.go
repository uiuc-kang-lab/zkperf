package gnarkcircuits

import (
	"crypto/rand"
	"fmt"
	"math/big"
	"reflect"
	"runtime"
	"testing"
	"time"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark-crypto/ecc/secp256k1/ecdsa"
	"github.com/consensys/gnark/backend/plonk"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/frontend/cs/scs"
	"github.com/consensys/gnark/std/math/emulated"
	gnarkecdsa "github.com/consensys/gnark/std/signature/ecdsa"
	"github.com/consensys/gnark/test"
)

func TestECDSA(t *testing.T) {
	assert := test.NewAssert(t)
	exp_len := []int{10, 10000, 1000000}
	for l := 0; l < len(exp_len); l++ {
		secret_key, e := ecdsa.GenerateKey(rand.Reader)
		assert.NoError(e)

		m := make([]byte, exp_len[l])
		fmt.Println("Message length: ", len(m))
		_, er := rand.Read(m)
		assert.NoError(er)

		signatureBytes, e := secret_key.Sign(m, nil)
		assert.NoError(e)

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

		ckt := ECDSACircuit{
			Message:   msg_object,
			Signature: sig_object,
			PublicKey: pk_object,
		}

		witness := ECDSACircuit{
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
		fmt.Println("Number of constraints: ", cs.GetNbConstraints())
		fmt.Println("Prover time: ", prover_time)
		fmt.Println("Verifier time: ", verifier_time)
		fmt.Println("Proof size: ", proof_size, "B")
		fmt.Println("Proof:", proof)
		var mem runtime.MemStats
		runtime.ReadMemStats(&mem)
		fmt.Println("Memory usage: ", mem.Sys/1024, "KB")
	}

	// assert.ProverSucceeded(circuit, witness, test.WithBackends(backend.PLONK), test.WithCurves(ecc.BN254))
}
