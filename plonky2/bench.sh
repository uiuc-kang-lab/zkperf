#!/bin/bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

ECDSA_DIR=$SCRIPT_DIR/ecdsa
MERKLE_DIR=$SCRIPT_DIR/merkle_tree
ML_DIR=$SCRIPT_DIR/ml

ECDSA_OUTPUT="$SCRIPT_DIR"/ecdsa.json
MERKLE_OUTPUT="$SCRIPT_DIR"/merkle.json
MNIST_OUTPUT="$SCRIPT_DIR"/mnist.json
DLRM_OUTPUT="$SCRIPT_DIR"/dlrm.json

ECDSA_LOG="$SCRIPT_DIR"/ecdsa_log
MERKLE_LOG="$SCRIPT_DIR"/merkle_log
MNIST_LOG="$SCRIPT_DIR"/mnist_log
DLRM_LOG="$SCRIPT_DIR"/dlrm_log

ECDSA_BREAK_JSON="$SCRIPT_DIR"/ecdsa_breakdown.json
MERKLE_BREAK_JSON="$SCRIPT_DIR"/merkle_breakdown.json
MNIST_BREAK_JSON="$SCRIPT_DIR"/mnist_breakdown.json
DLRM_BREAK_JSON="$SCRIPT_DIR"/dlrm_breakdown.json

ECDSA_BREAK_CSV="$SCRIPT_DIR"/ecdsa_breakdown.csv
MERKLE_BREAK_CSV="$SCRIPT_DIR"/merkle_breakdown.csv
MNIST_BREAK_CSV="$SCRIPT_DIR"/mnist_breakdown.csv
DLRM_BREAK_CSV="$SCRIPT_DIR"/dlrm_breakdown.csv

cd $ECDSA_DIR
cargo build --release
cd ..
touch $ECDSA_LOG
#"$ECDSA_DIR"/target/release/standard build
{ RUST_LOG=debug /usr/bin/time -v "$ECDSA_DIR"/target/release/standard "$ECDSA_OUTPUT" 40; } 2> "$ECDSA_LOG"
echo "$(jq --arg tmp $(echo "scale=6; $(cat "$ECDSA_LOG" | grep "Maximum resident set size" | tr -d -c 0-9)/1024" | bc) '.+={"MemoryConsumption": $tmp }' "$ECDSA_OUTPUT")" > "$ECDSA_OUTPUT"

cd $MERKLE_DIR
cargo build --release
cd ..
#"$MERKLE_DIR"/target/release/standard
touch $MERKLE_LOG
{ RUST_LOG=debug /usr/bin/time -v "$MERKLE_DIR"/target/release/standard "$MERKLE_OUTPUT" 25; } 2> "$MERKLE_LOG"
echo "$(jq --arg tmp $(echo "scale=6; $(cat "$MERKLE_LOG" | grep "Maximum resident set size" | tr -d -c 0-9)/1024" | bc) '.+={"MemoryConsumption": $tmp }' "$MERKLE_OUTPUT")" > "$MERKLE_OUTPUT"

cd $ML_DIR
cargo build --release
cd ..

touch $MNIST_LOG
"$ML_DIR"/target/release/time_circuit mnist "$ML_DIR"/examples/mnist/model.msgpack "$ML_DIR"/examples/mnist/inp.msgpack build "$MNIST_OUTPUT"
{ RUST_LOG=debug /usr/bin/time -v "$ML_DIR"/target/release/time_circuit mnist "$ML_DIR"/examples/mnist/model.msgpack "$ML_DIR"/examples/mnist/inp.msgpack prove "$MNIST_OUTPUT"; } 2> $MNIST_LOG
echo "$(jq --arg tmp $(echo "scale=6; $(cat "$MNIST_LOG" | grep "Maximum resident set size" | tr -d -c 0-9)/1024" | bc) '.+={"MemoryConsumption": $tmp }' "$MNIST_OUTPUT")" > "$MNIST_OUTPUT"
echo "$(jq '. += {"Circuit": "MNIST" }' "$MNIST_OUTPUT")" > "$MNIST_OUTPUT"

touch $DLRM_LOG
"$ML_DIR"/target/release/time_circuit dlrm "$ML_DIR"/examples/dlrm/model.msgpack "$ML_DIR"/examples/dlrm/inp.msgpack build "$DLRM_OUTPUT"
{ RUST_LOG=debug /usr/bin/time -v "$ML_DIR"/target/release/time_circuit dlrm "$ML_DIR"/examples/dlrm/model.msgpack "$ML_DIR"/examples/dlrm/inp.msgpack prove "$DLRM_OUTPUT"; } 2> $DLRM_LOG
echo "$(jq --arg tmp $(echo "scale=6; $(cat "$DLRM_LOG" | grep "Maximum resident set size" | tr -d -c 0-9)/1024" | bc) '.+={"MemoryConsumption": $tmp }' "$DLRM_OUTPUT")" > "$DLRM_OUTPUT"
echo "$(jq '. += {"Circuit": "DLRM" }' "$DLRM_OUTPUT")" > "$DLRM_OUTPUT"

# get breakdowns
python3 "$SCRIPT_DIR"/scripts/breakdown.py ecdsa $ECDSA_LOG $ECDSA_BREAK_JSON $ECDSA_BREAK_CSV
python3 "$SCRIPT_DIR"/scripts/breakdown.py merkle $MERKLE_LOG $MERKLE_BREAK_JSON $MERKLE_BREAK_CSV
python3 "$SCRIPT_DIR"/scripts/breakdown.py mnist $MNIST_LOG $MNIST_BREAK_JSON $MNIST_BREAK_CSV
python3 "$SCRIPT_DIR"/scripts/breakdown.py dlrm $DLRM_LOG $DLRM_BREAK_JSON $DLRM_BREAK_CSV

declare -a json_bds=("$ECDSA_BREAK_JSON" "$MERKLE_BREAK_JSON" "$MNIST_BREAK_JSON" "$DLRM_BREAK_JSON")
collected_outp=$(jq -n '[]')
for outp in "${json_bds[@]}"; do
    outp_list=$(jq -n --argjson dict "$(cat $outp)" '[ $dict ]')
    collected_outp=$(jq -n --argjson outp_list "$outp_list" --argjson collected_outp "$collected_outp" '$outp_list + $collected_outp')
done

echo "$collected_outp" > "$SCRIPT_DIR"/breakdowns.json
rm -r "$SCRIPT_DIR"/breakdowns.csv
cat "$ECDSA_BREAK_CSV" "$MERKLE_BREAK_CSV" "$MNIST_BREAK_CSV" "$DLRM_BREAK_CSV" >> "$SCRIPT_DIR"/breakdowns.csv
