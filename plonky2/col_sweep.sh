#!/bin/bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

ECDSA_DIR=$SCRIPT_DIR/ecdsa
MERKLE_DIR=$SCRIPT_DIR/merkle_tree
ML_DIR=$SCRIPT_DIR/ml
SWEEP_DIR=$SCRIPT_DIR/logs/column_sweep

cd $ECDSA_DIR
cargo build --release
cd ..

declare -a cols=(25 40 50 60 70 80 90 100 110 120 135)
mkdir -p logs/column_sweep
for col in "${cols[@]}"; do
    touch "$SWEEP_DIR"/"$col"_ecdsa_log
    #"$ECDSA_DIR"/target/release/standard build $col
    { RUST_LOG=debug /usr/bin/time -v "$ECDSA_DIR"/target/release/standard "$SWEEP_DIR"/"$col"_ecdsa.json $col; } 2> "$SWEEP_DIR"/"$col"_ecdsa_log
    echo "$(jq --arg tmp $(echo "scale=6; $(cat "$SWEEP_DIR"/"$col"_ecdsa_log | grep "Maximum resident set size" | tr -d -c 0-9)/1024" | bc) '.+={"MemoryConsumption": $tmp }' "$SWEEP_DIR"/"$col"_ecdsa.json)" > "$SWEEP_DIR"/"$col"_ecdsa.json
done

cd $MERKLE_DIR
cargo build --release
cd ..

for col in "${cols[@]}"; do
    touch "$SWEEP_DIR"/"$col"_merkle_log
    #"$MERKLE_DIR"/target/release/standard build $col
    { RUST_LOG=debug /usr/bin/time -v "$MERKLE_DIR"/target/release/standard "$SWEEP_DIR"/"$col"_merkle.json $col; } 2> "$SWEEP_DIR"/"$col"_merkle_log
    echo "$(jq --arg tmp $(echo "scale=6; $(cat "$SWEEP_DIR"/"$col"_merkle_log | grep "Maximum resident set size" | tr -d -c 0-9)/1024" | bc) '.+={"MemoryConsumption": $tmp }' "$SWEEP_DIR"/"$col"_merkle.json)" > "$SWEEP_DIR"/"$col"_merkle.json
done

cd $ML_DIR
cargo build --release
cd ..

for col in "${cols[@]}"; do
    touch "$SWEEP_DIR"/"$col"_mnist_log
    "$ML_DIR"/target/release/time_circuit mnist "$ML_DIR"/examples/mnist/model.msgpack "$ML_DIR"/examples/mnist/inp.msgpack build "$SWEEP_DIR"/"$col"_mnist.json $col
    { RUST_LOG=debug /usr/bin/time -v "$ML_DIR"/target/release/time_circuit mnist "$ML_DIR"/examples/mnist/model.msgpack "$ML_DIR"/examples/mnist/inp.msgpack prove "$SWEEP_DIR"/"$col"_mnist.json $col; } 2> "$SWEEP_DIR"/"$col"_mnist_log
    echo "$(jq --arg tmp $(echo "scale=6; $(cat "$SWEEP_DIR"/"$col"_mnist_log | grep "Maximum resident set size" | tr -d -c 0-9)/1024" | bc) '.+={"MemoryConsumption": $tmp }' "$SWEEP_DIR"/"$col"_mnist.json)" > "$SWEEP_DIR"/"$col"_mnist.json
    echo "$(jq '. += {"Circuit": "MNIST" }' "$SWEEP_DIR"/"$col"_mnist.json)" > "$SWEEP_DIR"/"$col"_mnist.json

    touch "$SWEEP_DIR"/"$col"_dlrm_log
    "$ML_DIR"/target/release/time_circuit dlrm "$ML_DIR"/examples/dlrm/model.msgpack "$ML_DIR"/examples/dlrm/inp.msgpack build "$SWEEP_DIR"/"$col"_dlrm.json $col
    { RUST_LOG=debug /usr/bin/time -v "$ML_DIR"/target/release/time_circuit dlrm "$ML_DIR"/examples/dlrm/model.msgpack "$ML_DIR"/examples/dlrm/inp.msgpack prove "$SWEEP_DIR"/"$col"_dlrm.json $col; } 2> "$SWEEP_DIR"/"$col"_dlrm_log
    echo "$(jq --arg tmp $(echo "scale=6; $(cat "$SWEEP_DIR"/"$col"_dlrm_log | grep "Maximum resident set size" | tr -d -c 0-9)/1024" | bc) '.+={"MemoryConsumption": $tmp }' "$SWEEP_DIR"/"$col"_dlrm.json)" > "$SWEEP_DIR"/"$col"_dlrm.json
    echo "$(jq '. += {"Circuit": "DLRM" }' "$SWEEP_DIR"/"$col"_dlrm.json)" > "$SWEEP_DIR"/"$col"_dlrm.json
done

curl -d "plonky2 sweep" ntfy.sh/zk_benchmark