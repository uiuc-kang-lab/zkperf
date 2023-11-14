#!/bin/bash

PHASE1=pot21_final.ptau
BUILD_DIR=build/mt_keccak
CIRCUIT_NAME=mtverifier
INPUT=data/mt_verifier_input.json

if [ -f "$PHASE1" ]; then
    echo "Found Phase 1 ptau file"
else
    echo "No Phase 1 ptau file found. Exiting..."
    exit 1
fi

if [ ! -d "$BUILD_DIR" ]; then
    echo "No build directory found. Creating build directory..."
    mkdir -p "$BUILD_DIR"
fi

circom circuits/"$CIRCUIT_NAME".circom --r1cs --wasm --sym --c --wat --output "$BUILD_DIR"
node "$BUILD_DIR"/"$CIRCUIT_NAME"_js/generate_witness.js "$BUILD_DIR"/"$CIRCUIT_NAME"_js/"$CIRCUIT_NAME".wasm $INPUT "$BUILD_DIR"/witness.wtns
node ../snarkjs/cli.js groth16 setup "$BUILD_DIR"/"$CIRCUIT_NAME".r1cs "$PHASE1" "$BUILD_DIR"/"$CIRCUIT_NAME"_0.zkey
node ../snarkjs/cli.js zkey beacon "$BUILD_DIR"/"$CIRCUIT_NAME"_0.zkey "$BUILD_DIR"/"$CIRCUIT_NAME".zkey 0102030405060708090a0b0c0d0e0f101112231415161718221a1b1c1d1e1f 10 -n="Final Beacon phase2"
node ../snarkjs/cli.js zkey verify "$BUILD_DIR"/"$CIRCUIT_NAME".r1cs "$PHASE1" "$BUILD_DIR"/"$CIRCUIT_NAME".zkey
node ../snarkjs/cli.js zkey export verificationkey "$BUILD_DIR"/"$CIRCUIT_NAME".zkey "$BUILD_DIR"/vkey.json
node ../snarkjs/cli.js -v groth16 prove "$BUILD_DIR"/"$CIRCUIT_NAME".zkey "$BUILD_DIR"/witness.wtns "$BUILD_DIR"/proof.json "$BUILD_DIR"/public.json > merkle_prover_breakdown.txt
node ../snarkjs/cli.js -v groth16 verify "$BUILD_DIR"/vkey.json "$BUILD_DIR"/public.json "$BUILD_DIR"/proof.json > merkle_verifier_breakdown.txt
python ../breakdown/breakdown.py merkle merkle_prover_breakdown.txt ../breakdown/merkle_prover_breakdown.json
python ../breakdown/breakdown.py merkle merkle_verifier_breakdown.txt ../breakdown/merkle_verifier_breakdown.json





