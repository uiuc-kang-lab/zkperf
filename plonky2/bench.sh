#!/bin/bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
ML_DIR=$SCRIPT_DIR/ml

cd $ML_DIR
cargo build --release
cd ..

OUTPUT=$1
MNIST_OUTPUT="$SCRIPT_DIR"/mnist.json
DLRM_OUTPUT="$SCRIPT_DIR"/dlrm.json

declare -a json_outps=("$MNIST_OUTPUT" "$DLRM_OUTPUT")

for outp in "${json_outps[@]}"; do
    rm "$outp"
done

touch /tmp/test
{ /usr/bin/time -v "$ML_DIR"/target/release/time_circuit "$ML_DIR"/examples/mnist/model.msgpack "$ML_DIR"/examples/mnist/inp.msgpack "$MNIST_OUTPUT"; } 2> /tmp/test
cat "$MNIST_OUTPUT" | jq "{"MemoryConsumption": "$(cat /tmp/test | grep "Maximum resident set size" | tr -d -c 0-9)"} + ." > "$MNIST_OUTPUT"
echo "$(jq '. += {"Circuit": "MNIST" }' "$MNIST_OUTPUT")" > "$MNIST_OUTPUT"
echo "$(jq --arg tmp $(lscpu | grep "Model name:" | sed -e "s/^Model name:                      //" | sed -e "s/\s\+/./g") \
'. += {"Hardware": $tmp }' "$MNIST_OUTPUT")" > "$MNIST_OUTPUT"
rm /tmp/test

touch /tmp/test
{ /usr/bin/time -v "$ML_DIR"/target/release/time_circuit "$ML_DIR"/examples/dlrm/model.msgpack "$ML_DIR"/examples/dlrm/zero.msgpack "$DLRM_OUTPUT"; } 2> /tmp/test
cat "$DLRM_OUTPUT" | jq "{"MemoryConsumption": "$(cat /tmp/test | grep "Maximum resident set size" | tr -d -c 0-9)"} + ." > "$DLRM_OUTPUT"
echo "$(jq '. += {"Circuit": "DLRM" }' "$DLRM_OUTPUT")" > "$DLRM_OUTPUT"
echo "$(jq --arg tmp $(lscpu | grep "Model name:" | sed -e "s/^Model name:                      //" | sed -e "s/\s\+/./g") \
'. += {"Hardware": $tmp }' "$DLRM_OUTPUT")" > "$DLRM_OUTPUT"
rm /tmp/test

collected_outp=$(jq -n '[]')
for outp in "${json_outps[@]}"; do
    outp_list=$(jq -n --argjson dict "$(cat $outp)" '[ $dict ]')
    collected_outp=$(jq -n --argjson outp_list "$outp_list" --argjson collected_outp "$collected_outp" '$outp_list + $collected_outp')
done

echo "$collected_outp" > "$OUTPUT"