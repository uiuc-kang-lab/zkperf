#!/bin/bash

BUILD_DIR=.
OUTPUT="measurement.json"
EXECUTABLE="./target/release/merkle_tree"
CIRCUIT="mt_keccak"
if [ ! -f "$OUTPUT" ]; then
    touch "$BUILD_DIR"/"$OUTPUT"
    echo "{}" > "$OUTPUT"
fi

cargo build --release

echo "$(jq '. += {"Framework": "Halo2" }' "$BUILD_DIR"/"$OUTPUT")" > "$BUILD_DIR"/"$OUTPUT"
echo "$(jq '. += {"Circuit": "Merkle Tree 1024" }' "$BUILD_DIR"/"$OUTPUT")" > "$BUILD_DIR"/"$OUTPUT"
echo "$(jq '. += {"Backend": "Plonk" }' "$BUILD_DIR"/"$OUTPUT")" > "$BUILD_DIR"/"$OUTPUT"
echo "$(jq '. += {"Curve": "BN254" }' "$BUILD_DIR"/"$OUTPUT")" > "$BUILD_DIR"/"$OUTPUT"
echo "$(jq --arg tmp $(lscpu | grep "Model name:" | sed -e "s/^Model name:                      //" | sed -e "s/\s\+/./g") \
'. += {"Hardware": $tmp }' "$BUILD_DIR"/"$OUTPUT")" > "$BUILD_DIR"/"$OUTPUT"

echo "****Setup****"
$EXECUTABLE --name $CIRCUIT -k 12 mock
$EXECUTABLE --name $CIRCUIT -k 12 keygen

echo "****GENERATING PROOF FOR SAMPLE INPUT****"
start=`date +%s%N`
touch /tmp/test
{ /usr/bin/time -v $EXECUTABLE --name $CIRCUIT -k 12 prove > merkle.log; } 2> /tmp/test
echo "$(jq --arg tmp $(echo "scale=6; $(cat /tmp/test | grep "Maximum resident set size" | tr -d -c 0-9)/1024" | bc) '.+={"MemoryConsumption": $tmp }' "$BUILD_DIR"/"$OUTPUT")" > "$BUILD_DIR"/"$OUTPUT"
rm /tmp/test
end=`date +%s%N`
echo "$(jq --arg tmp $(stat -c %s "$BUILD_DIR"/data/"$CIRCUIT".snark) '.+={"ProofSize": $tmp}' "$BUILD_DIR"/"$OUTPUT")" > "$BUILD_DIR"/"$OUTPUT"
echo "$(jq --arg tmp $(echo "scale=6; $((end-start))/1000000000" | bc) '.+={"ProverTime": $tmp}' "$BUILD_DIR"/"$OUTPUT")" > "$BUILD_DIR"/"$OUTPUT"
echo "Proof Size: $(stat -c %s "$BUILD_DIR"/data/"$CIRCUIT".proof)bytes"
echo "DONE ($((end-start))ns)"

echo "****VERIFYING PROOF FOR SAMPLE INPUT****"
start=`date +%s%N`
$EXECUTABLE --name $CIRCUIT -k 12 verify
end=`date +%s%N`
echo "$(jq --arg tmp $(echo "scale=6; $((end-start))/1000000" | bc) '.+={"VerifierTime": $tmp}' "$BUILD_DIR"/"$OUTPUT")" > "$BUILD_DIR"/"$OUTPUT"
echo "DONE ($((end-start))ns)"
