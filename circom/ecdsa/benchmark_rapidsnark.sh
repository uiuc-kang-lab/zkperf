#!/bin/bash

PHASE1=pot21_final.ptau
BUILD_DIR=build/ecdsa
CIRCUIT_NAME=ecdsa_verify
INPUT=data/input.json
OUTPUT="measurement.json"
RAPIDSNARK_PROVER=~/rapidsnark/package/bin/prover
RAPIDSNARK_VERIFIER=~/rapidsnark/package/bin/verifier

if [ -f "$PHASE1" ]; then
    echo "Found Phase 1 ptau file"
else
    echo "No Phase 1 ptau file found. Downloading..."
    wget -O pot21_final.ptau https://hermez.s3-eu-west-1.amazonaws.com/powersOfTau28_hez_final_21.ptau
fi

if [ ! -d "$BUILD_DIR" ]; then
    echo "No build directory found. Creating build directory..."
    mkdir -p "$BUILD_DIR"
fi

if [ ! -f "$BUILD_DIR"/"$OUTPUT" ]; then
    touch "$BUILD_DIR"/"$OUTPUT"
    echo "{}" > "$BUILD_DIR"/"$OUTPUT"
fi

echo "$(jq '. += {"Framework": "circom_rapidsnark" }' "$BUILD_DIR"/"$OUTPUT")" > "$BUILD_DIR"/"$OUTPUT"
echo "$(jq '. += {"Circuit": "ECDSA" }' "$BUILD_DIR"/"$OUTPUT")" > "$BUILD_DIR"/"$OUTPUT"
echo "$(jq '. += {"Backend": "Groth16" }' "$BUILD_DIR"/"$OUTPUT")" > "$BUILD_DIR"/"$OUTPUT"
echo "$(jq '. += {"Curve": "BN254" }' "$BUILD_DIR"/"$OUTPUT")" > "$BUILD_DIR"/"$OUTPUT"
echo "$(jq --arg tmp $(lscpu | grep "Model name:" | sed -e "s/^Model name:                      //" | sed -e "s/\s\+/./g") \
'. += {"Hardware": $tmp }' "$BUILD_DIR"/"$OUTPUT")" > "$BUILD_DIR"/"$OUTPUT"

echo "****COMPILING CIRCUIT****"
start=`date +%s`
set -x
circom circuits/"$CIRCUIT_NAME".circom --r1cs --wasm --sym --c --wat --output "$BUILD_DIR"
{ set +x; } 2>/dev/null
end=`date +%s`
echo "DONE ($((end-start))s)"

echo "****GENERATING WITNESS FOR SAMPLE INPUT****"
start=`date +%s`
node "$BUILD_DIR"/"$CIRCUIT_NAME"_js/generate_witness.js "$BUILD_DIR"/"$CIRCUIT_NAME"_js/"$CIRCUIT_NAME".wasm $INPUT "$BUILD_DIR"/witness.wtns
end=`date +%s`
echo "DONE ($((end-start))s)"

echo "****GENERATING ZKEY 0****"
start=`date +%s`
npx snarkjs groth16 setup "$BUILD_DIR"/"$CIRCUIT_NAME".r1cs "$PHASE1" "$BUILD_DIR"/"$CIRCUIT_NAME"_0.zkey
end=`date +%s`
echo "DONE ($((end-start))s)"

echo "****GENERATING FINAL ZKEY****"
start=`date +%s`
npx snarkjs zkey beacon "$BUILD_DIR"/"$CIRCUIT_NAME"_0.zkey "$BUILD_DIR"/"$CIRCUIT_NAME".zkey 0102030405060708090a0b0c0d0e0f101112231415161718221a1b1c1d1e1f 10 -n="Final Beacon phase2"
end=`date +%s`
echo "DONE ($((end-start))s)"

echo "****VERIFYING FINAL ZKEY****"
start=`date +%s`
npx snarkjs zkey verify "$BUILD_DIR"/"$CIRCUIT_NAME".r1cs "$PHASE1" "$BUILD_DIR"/"$CIRCUIT_NAME".zkey
end=`date +%s`
echo "DONE ($((end-start))s)"

echo "** Exporting vkey"
start=`date +%s`
npx snarkjs zkey export verificationkey "$BUILD_DIR"/"$CIRCUIT_NAME".zkey "$BUILD_DIR"/vkey.json
end=`date +%s`
echo "DONE ($((end-start))s)"

echo "****GENERATING PROOF FOR SAMPLE INPUT****"
start=`date +%s%N`
touch /tmp/test
{ /usr/bin/time -v $RAPIDSNARK_PROVER "$BUILD_DIR"/"$CIRCUIT_NAME".zkey "$BUILD_DIR"/witness.wtns "$BUILD_DIR"/proof.json "$BUILD_DIR"/public.json; } 2> /tmp/test
echo "$(jq --arg tmp $(echo "scale=6; $(cat /tmp/test | grep "Maximum resident set size" | tr -d -c 0-9)/1024" | bc) '.+={"MemoryConsumption": $tmp }' "$BUILD_DIR"/"$OUTPUT")" > "$BUILD_DIR"/"$OUTPUT"
rm /tmp/test
end=`date +%s%N`
echo "$(jq --arg tmp $(stat -c %s "$BUILD_DIR"/proof.json) '.+={"ProofSize": $tmp}' "$BUILD_DIR"/"$OUTPUT")" > "$BUILD_DIR"/"$OUTPUT"
echo "$(jq --arg tmp $(echo "scale=6; $((end-start))/1000000000" | bc) '.+={"ProverTime": $tmp}' "$BUILD_DIR"/"$OUTPUT")" > "$BUILD_DIR"/"$OUTPUT"
echo "Proof Size: $(stat -c %s "$BUILD_DIR"/proof.json)bytes"
echo "Public Size: $(stat -c %s "$BUILD_DIR"/public.json)bytes"
echo "DONE ($((end-start))ns)"


echo "****VERIFYING PROOF FOR SAMPLE INPUT****"
start=`date +%s%N`
$RAPIDSNARK_VERIFIER "$BUILD_DIR"/vkey.json "$BUILD_DIR"/public.json "$BUILD_DIR"/proof.json
end=`date +%s%N`
echo "$(jq --arg tmp $(echo "scale=6; $((end-start))/1000000000" | bc) '.+={"VerifierTime": $tmp}' "$BUILD_DIR"/"$OUTPUT")" > "$BUILD_DIR"/"$OUTPUT"
echo "DONE ($((end-start))ns)"
