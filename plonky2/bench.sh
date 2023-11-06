#!/bin/bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

ECDSA_DIR=$SCRIPT_DIR/ecdsa
MERKLE_DIR=$SCRIPT_DIR/merkle_tree
ML_DIR=$SCRIPT_DIR/ml

# OUTPUT=$1
ECDSA_OUTPUT="$SCRIPT_DIR"/ecdsa.json
MERKLE_OUTPUT="$SCRIPT_DIR"/merkle.json
MNIST_OUTPUT="$SCRIPT_DIR"/mnist.json
DLRM_OUTPUT="$SCRIPT_DIR"/dlrm.json

cd $ECDSA_DIR
cargo build --release
cd ..
touch /tmp/test
"$ECDSA_DIR"/target/release/standard build
{ RUST_LOG=debug /usr/bin/time -v "$ECDSA_DIR"/target/release/standard; } 2> /tmp/test
echo "$(jq --arg tmp $(echo "scale=6; $(cat /tmp/test | grep "Maximum resident set size" | tr -d -c 0-9)/1024" | bc) '.+={"MemoryConsumption": $tmp }' "$ECDSA_OUTPUT")" > "$ECDSA_OUTPUT"
rm /tmp/test

cd $MERKLE_DIR
cargo test build_merkle -- --nocapture
touch /tmp/test
{ RUST_LOG=debug /usr/bin/time -v cargo test test_merkle -- --nocapture; } 2> /tmp/test
echo "$(jq --arg tmp $(echo "scale=6; $(cat /tmp/test | grep "Maximum resident set size" | tr -d -c 0-9)/1024" | bc) '.+={"MemoryConsumption": $tmp }' "$MERKLE_OUTPUT")" > "$MERKLE_OUTPUT"
rm /tmp/test

cd $ML_DIR
cargo build --release
cd ..

touch /tmp/test

"$ML_DIR"/target/release/time_circuit mnist "$ML_DIR"/examples/mnist/model.msgpack "$ML_DIR"/examples/mnist/inp.msgpack build "$MNIST_OUTPUT"
{ RUST_LOG=debug /usr/bin/time -v "$ML_DIR"/target/release/time_circuit mnist "$ML_DIR"/examples/mnist/model.msgpack "$ML_DIR"/examples/mnist/inp.msgpack prove "$MNIST_OUTPUT"; } 2> /tmp/test
echo "$(jq --arg tmp $(echo "scale=6; $(cat /tmp/test | grep "Maximum resident set size" | tr -d -c 0-9)/1024" | bc) '.+={"MemoryConsumption": $tmp }' "$MNIST_OUTPUT")" > "$MNIST_OUTPUT"
echo "$(jq '. += {"Circuit": "MNIST" }' "$MNIST_OUTPUT")" > "$MNIST_OUTPUT"
rm /tmp/test

touch /tmp/test
"$ML_DIR"/target/release/time_circuit dlrm "$ML_DIR"/examples/dlrm/model.msgpack "$ML_DIR"/examples/dlrm/inp.msgpack build "$DLRM_OUTPUT"
{ RUST_LOG=debug /usr/bin/time -v "$ML_DIR"/target/release/time_circuit dlrm "$ML_DIR"/examples/dlrm/model.msgpack "$ML_DIR"/examples/dlrm/inp.msgpack prove "$DLRM_OUTPUT"; } 2> /tmp/test
echo "$(jq --arg tmp $(echo "scale=6; $(cat /tmp/test | grep "Maximum resident set size" | tr -d -c 0-9)/1024" | bc) '.+={"MemoryConsumption": $tmp }' "$DLRM_OUTPUT")" > "$DLRM_OUTPUT"
echo "$(jq '. += {"Circuit": "DLRM" }' "$DLRM_OUTPUT")" > "$DLRM_OUTPUT"
rm /tmp/test

# declare -a json_outps=("$ECDSA_OUTPUT" "$MERKLE_OUTPUT" "$MNIST_OUTPUT" "$DLRM_OUTPUT")
# collected_outp=$(jq -n '[]')
# for outp in "${json_outps[@]}"; do
#     outp_list=$(jq -n --argjson dict "$(cat $outp)" '[ $dict ]')
#     collected_outp=$(jq -n --argjson outp_list "$outp_list" --argjson collected_outp "$collected_outp" '$outp_list + $collected_outp')
# done

# echo "$collected_outp" > "$OUTPUT"