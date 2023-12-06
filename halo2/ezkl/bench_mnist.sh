#!/bin/bash

BUILD_DIR=.
OUTPUT="mnist_measurement.json"
EXECUTABLE="./target/release/ezkl"
NAME=model_truncate
EXAMPLE_PATH="examples/mnist"

if [ ! -f "$OUTPUT" ]; then
    touch "$BUILD_DIR"/"$OUTPUT"
    echo "{}" > "$OUTPUT"
fi

echo "$(jq '. += {"Framework": "Halo2_EZKL" }' "$BUILD_DIR"/"$OUTPUT")" > "$BUILD_DIR"/"$OUTPUT"
echo "$(jq '. += {"Circuit": "MNIST" }' "$BUILD_DIR"/"$OUTPUT")" > "$BUILD_DIR"/"$OUTPUT"
echo "$(jq '. += {"Backend": "Plonk" }' "$BUILD_DIR"/"$OUTPUT")" > "$BUILD_DIR"/"$OUTPUT"
echo "$(jq '. += {"Curve": "BN254" }' "$BUILD_DIR"/"$OUTPUT")" > "$BUILD_DIR"/"$OUTPUT"
echo "$(jq --arg tmp $(lscpu | grep "Model name:" | sed -e "s/^Model name:                      //" | sed -e "s/\s\+/./g") \
'. += {"Hardware": $tmp }' "$BUILD_DIR"/"$OUTPUT")" > "$BUILD_DIR"/"$OUTPUT"

echo "****Setup****"
$EXECUTABLE gen-settings -M "$EXAMPLE_PATH/$NAME.onnx"
$EXECUTABLE calibrate-settings -M "$EXAMPLE_PATH/$NAME.onnx" -D "$EXAMPLE_PATH/input.json" --target resources --scales 9
$EXECUTABLE get-srs -S settings.json
$EXECUTABLE compile-circuit -M "$EXAMPLE_PATH/$NAME.onnx" -S settings.json --compiled-circuit "$NAME.ezkl"
$EXECUTABLE setup -M "$NAME.ezkl" --srs-path=kzg.srs --vk-path=vk.key --pk-path=pk.key
$EXECUTABLE gen-witness -D "$EXAMPLE_PATH/input.json" -M "$NAME.ezkl"
$EXECUTABLE mock -M "$NAME.ezkl" --witness witness.json

echo "****GENERATING PROOF FOR SAMPLE INPUT****"
start=`date +%s%N`
touch /tmp/test
{ /usr/bin/time -v $EXECUTABLE prove -M $NAME.ezkl --witness witness.json --pk-path=pk.key --proof-path=model.proof --srs-path=kzg.srs; } 2> /tmp/test
echo "$(jq --arg tmp $(echo "scale=6; $(cat /tmp/test | grep "Maximum resident set size" | tr -d -c 0-9)/1024" | bc) '.+={"MemoryConsumption": $tmp }' "$BUILD_DIR"/"$OUTPUT")" > "$BUILD_DIR"/"$OUTPUT"
rm /tmp/test
end=`date +%s%N`
echo "$(jq --arg tmp $(stat -c %s "$BUILD_DIR"/model.proof) '.+={"ProofSize": $tmp}' "$BUILD_DIR"/"$OUTPUT")" > "$BUILD_DIR"/"$OUTPUT"
echo "$(jq --arg tmp $(echo "scale=6; $((end-start))/1000000000" | bc) '.+={"ProverTime": $tmp}' "$BUILD_DIR"/"$OUTPUT")" > "$BUILD_DIR"/"$OUTPUT"
echo "Proof Size: $(stat -c %s "$BUILD_DIR"/model.proof)bytes"
echo "DONE ($((end-start))ns)"

echo "****VERIFYING PROOF FOR SAMPLE INPUT****"
start=`date +%s%N`
$EXECUTABLE verify --proof-path=model.proof --settings-path=settings.json --vk-path=vk.key --srs-path=kzg.srs
end=`date +%s%N`
echo "$(jq --arg tmp $(echo "scale=6; $((end-start))/1000000" | bc) '.+={"VerifierTime": $tmp}' "$BUILD_DIR"/"$OUTPUT")" > "$BUILD_DIR"/"$OUTPUT"
echo "DONE ($((end-start))ns)"
