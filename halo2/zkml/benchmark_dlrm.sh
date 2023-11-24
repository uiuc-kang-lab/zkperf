#!/bin/bash

BUILD_DIR=.
OUTPUT="dlrm_measurement.json"
EXECUTABLE="./target/release/bench_circuit"
MODEL="examples/dlrm/dlrm_model.msgpack"
INPUT="examples/dlrm/dlrm_input.msgpack"

if [ ! -f "$OUTPUT" ]; then
    touch "$BUILD_DIR"/"$OUTPUT"
    echo "{}" > "$OUTPUT"
fi

if [ ! -d "./params_kzg" ]; then
    echo "No params directory found. Creating the directory..."
    mkdir -p "./params_kzg"
fi

cargo build --release

echo "$(jq '. += {"Framework": "Halo2" }' "$BUILD_DIR"/"$OUTPUT")" > "$BUILD_DIR"/"$OUTPUT"
echo "$(jq '. += {"Circuit": "DLRM" }' "$BUILD_DIR"/"$OUTPUT")" > "$BUILD_DIR"/"$OUTPUT"
echo "$(jq '. += {"Backend": "Plonk" }' "$BUILD_DIR"/"$OUTPUT")" > "$BUILD_DIR"/"$OUTPUT"
echo "$(jq '. += {"Curve": "BN254" }' "$BUILD_DIR"/"$OUTPUT")" > "$BUILD_DIR"/"$OUTPUT"
echo "$(jq --arg tmp $(lscpu | grep "Model name:" | sed -e "s/^Model name:                      //" | sed -e "s/\s\+/./g") \
'. += {"Hardware": $tmp }' "$BUILD_DIR"/"$OUTPUT")" > "$BUILD_DIR"/"$OUTPUT"

echo "****Setup****"
$EXECUTABLE $MODEL $INPUT setup

echo "****GENERATING PROOF FOR SAMPLE INPUT****"
start=`date +%s%N`
touch /tmp/test
{ /usr/bin/time -v $EXECUTABLE $MODEL $INPUT prove > dlrm_witness.log; } 2> /tmp/test
echo "$(jq --arg tmp $(echo "scale=6; $(cat /tmp/test | grep "Maximum resident set size" | tr -d -c 0-9)/1024" | bc) '.+={"MemoryConsumption": $tmp }' "$BUILD_DIR"/"$OUTPUT")" > "$BUILD_DIR"/"$OUTPUT"
rm /tmp/test
end=`date +%s%N`
echo "$(jq --arg tmp $(stat -c %s "$BUILD_DIR"/proof) '.+={"ProofSize": $tmp}' "$BUILD_DIR"/"$OUTPUT")" > "$BUILD_DIR"/"$OUTPUT"
echo "$(jq --arg tmp $(echo "scale=6; $((end-start))/1000000000" | bc) '.+={"ProverTime": $tmp}' "$BUILD_DIR"/"$OUTPUT")" > "$BUILD_DIR"/"$OUTPUT"
echo "Proof Size: $(stat -c %s "$BUILD_DIR"/proof)bytes"
echo "DONE ($((end-start))ns)"

echo "****VERIFYING PROOF FOR SAMPLE INPUT****"
start=`date +%s%N`
$EXECUTABLE $MODEL $INPUT verify
end=`date +%s%N`
echo "$(jq --arg tmp $(echo "scale=6; $((end-start))/1000000" | bc) '.+={"VerifierTime": $tmp}' "$BUILD_DIR"/"$OUTPUT")" > "$BUILD_DIR"/"$OUTPUT"
echo "DONE ($((end-start))ns)"
