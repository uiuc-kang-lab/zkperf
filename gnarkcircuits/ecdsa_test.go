package gnarkcircuits

import (
	"bytes"
	"crypto/rand"
	"fmt"
	"math"
	"math/big"
	"runtime"
	"testing"
	"time"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark-crypto/ecc/secp256k1/ecdsa"
	"github.com/consensys/gnark/backend/groth16"
	"github.com/consensys/gnark/backend/plonk"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/frontend/cs/r1cs"
	"github.com/consensys/gnark/frontend/cs/scs"
	"github.com/consensys/gnark/std/math/emulated"
	gnarkecdsa "github.com/consensys/gnark/std/signature/ecdsa"
	"github.com/consensys/gnark/test"
)

func TestECDSAPlonkBN254(t *testing.T) {
	assert := test.NewAssert(t)
	NumConstraints := 0
	mean_prover_time := 0
	mean_verifier_time := 0
	mean_proof_size := 0
	mean_mem := 0
	for l := 0; l < 1; l++ {
		secret_key, e := ecdsa.GenerateKey(rand.Reader)
		assert.NoError(e)

		m := make([]byte, int(math.Pow(2, 10)))
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
		prover_time := time.Since(prover_start).Microseconds()
		ser_proof := new(bytes.Buffer)
		proof.WriteTo(ser_proof)
		proof_size := ser_proof.Len()
		pubwtns, _ := wtns.Public()
		verifier_start := time.Now()
		err := plonk.Verify(proof, vK, pubwtns)
		verifier_time := time.Since(verifier_start).Microseconds()
		assert.Equal(err, nil)
		NumConstraints = cs.GetNbConstraints()
		var mem runtime.MemStats
		runtime.ReadMemStats(&mem)
		memUsage := int(mem.Sys / 1024)
		mean_prover_time += int(prover_time)
		mean_verifier_time += int(verifier_time)
		mean_proof_size += proof_size
		mean_mem += memUsage
	}
	fmt.Println("Number of constraints: ", NumConstraints)
	fmt.Println("Prover time: ", mean_prover_time, "µs")
	fmt.Println("Verifier time: ", mean_verifier_time, "µs")
	fmt.Println("Proof size: ", mean_proof_size, "B")
	fmt.Println("Memory usage: ", mean_mem, "KB")

	// assert.ProverSucceeded(circuit, witness, test.WithBackends(backend.PLONK), test.WithCurves(ecc.BN254))
}

func TestECDSAGroth16BN254(t *testing.T) {
	assert := test.NewAssert(t)
	NumConstraints := 0
	mean_prover_time := 0
	mean_verifier_time := 0
	mean_proof_size := 0
	mean_mem := 0
	for l := 0; l < 1; l++ {
		secret_key, e := ecdsa.GenerateKey(rand.Reader)
		assert.NoError(e)

		m := make([]byte, int(math.Pow(2, 10)))
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
		cs, _ := frontend.Compile(ecc.BN254.ScalarField(), r1cs.NewBuilder, &ckt)
		pK, vK, _ := groth16.Setup(cs)
		prover_start := time.Now()
		proof, _ := groth16.Prove(cs, pK, wtns)
		prover_time := time.Since(prover_start).Microseconds()
		ser_proof := new(bytes.Buffer)
		proof.WriteTo(ser_proof)
		proof_size := ser_proof.Len()
		pubwtns, _ := wtns.Public()
		verifier_start := time.Now()
		err := groth16.Verify(proof, vK, pubwtns)
		verifier_time := time.Since(verifier_start).Microseconds()
		assert.Equal(err, nil)
		NumConstraints = cs.GetNbConstraints()
		var mem runtime.MemStats
		runtime.ReadMemStats(&mem)
		memUsage := int(mem.Sys / 1024)
		mean_prover_time += int(prover_time)
		mean_verifier_time += int(verifier_time)
		mean_proof_size += proof_size
		mean_mem += memUsage
	}
	fmt.Println("Number of constraints: ", NumConstraints)
	fmt.Println("Prover time: ", mean_prover_time, "µs")
	fmt.Println("Verifier time: ", mean_verifier_time, "µs")
	fmt.Println("Proof size: ", mean_proof_size, "B")
	fmt.Println("Memory usage: ", mean_mem, "KB")

	// assert.ProverSucceeded(circuit, witness, test.WithBackends(backend.PLONK), test.WithCurves(ecc.BN254))
}

func TestECDSAPlonkBLS(t *testing.T) {
	assert := test.NewAssert(t)
	NumConstraints := 0
	mean_prover_time := 0
	mean_verifier_time := 0
	mean_proof_size := 0
	mean_mem := 0
	for l := 0; l < 1; l++ {
		secret_key, e := ecdsa.GenerateKey(rand.Reader)
		assert.NoError(e)

		m := make([]byte, int(math.Pow(2, 10)))
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
		wtns, _ := frontend.NewWitness(&witness, ecc.BLS12_381.ScalarField())
		cs, _ := frontend.Compile(ecc.BLS12_381.ScalarField(), scs.NewBuilder, &ckt)
		srs, _ := test.NewKZGSRS(cs)
		pK, vK, _ := plonk.Setup(cs, srs)
		prover_start := time.Now()
		proof, _ := plonk.Prove(cs, pK, wtns)
		prover_time := time.Since(prover_start).Microseconds()
		ser_proof := new(bytes.Buffer)
		proof.WriteTo(ser_proof)
		proof_size := ser_proof.Len()
		pubwtns, _ := wtns.Public()
		verifier_start := time.Now()
		err := plonk.Verify(proof, vK, pubwtns)
		verifier_time := time.Since(verifier_start).Microseconds()
		assert.Equal(err, nil)
		NumConstraints = cs.GetNbConstraints()
		var mem runtime.MemStats
		runtime.ReadMemStats(&mem)
		memUsage := int(mem.Sys / 1024)
		mean_prover_time += int(prover_time)
		mean_verifier_time += int(verifier_time)
		mean_proof_size += proof_size
		mean_mem += memUsage
	}
	fmt.Println("Number of constraints: ", NumConstraints)
	fmt.Println("Prover time: ", mean_prover_time, "µs")
	fmt.Println("Verifier time: ", mean_verifier_time, "µs")
	fmt.Println("Proof size: ", mean_proof_size, "B")
	fmt.Println("Memory usage: ", mean_mem, "KB")

	// assert.ProverSucceeded(circuit, witness, test.WithBackends(backend.PLONK), test.WithCurves(ecc.BN254))
}

func TestECDSAGroth16BLS(t *testing.T) {
	assert := test.NewAssert(t)
	NumConstraints := 0
	mean_prover_time := 0
	mean_verifier_time := 0
	mean_proof_size := 0
	mean_mem := 0
	for l := 0; l < 1; l++ {
		secret_key, e := ecdsa.GenerateKey(rand.Reader)
		assert.NoError(e)

		m := make([]byte, int(math.Pow(2, 10)))
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
		wtns, _ := frontend.NewWitness(&witness, ecc.BLS12_381.ScalarField())
		cs, _ := frontend.Compile(ecc.BLS12_381.ScalarField(), r1cs.NewBuilder, &ckt)
		pK, vK, _ := groth16.Setup(cs)
		prover_start := time.Now()
		proof, _ := groth16.Prove(cs, pK, wtns)
		prover_time := time.Since(prover_start).Microseconds()
		ser_proof := new(bytes.Buffer)
		proof.WriteTo(ser_proof)
		proof_size := ser_proof.Len()
		pubwtns, _ := wtns.Public()
		verifier_start := time.Now()
		err := groth16.Verify(proof, vK, pubwtns)
		verifier_time := time.Since(verifier_start).Microseconds()
		assert.Equal(err, nil)
		NumConstraints = cs.GetNbConstraints()
		var mem runtime.MemStats
		runtime.ReadMemStats(&mem)
		memUsage := int(mem.Sys / 1024)
		mean_prover_time += int(prover_time)
		mean_verifier_time += int(verifier_time)
		mean_proof_size += proof_size
		mean_mem += memUsage
	}
	fmt.Println("Number of constraints: ", NumConstraints)
	fmt.Println("Prover time: ", mean_prover_time, "µs")
	fmt.Println("Verifier time: ", mean_verifier_time, "µs")
	fmt.Println("Proof size: ", mean_proof_size, "B")
	fmt.Println("Memory usage: ", mean_mem, "KB")

	// assert.ProverSucceeded(circuit, witness, test.WithBackends(backend.PLONK), test.WithCurves(ecc.BN254))
}
