#!/bin/bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
ML_DIR=$SCRIPT_DIR/ml
MNIST_OUTPUT="$SCRIPT_DIR"/mnist.json
DLRM_OUTPUT="$SCRIPT_DIR"/dlrm.json

cd $ML_DIR
cargo build --release
cd ..

# Sweep Var 25

RUST_LOG=debug "$ML_DIR"/target/release/time_circuit mnist "$ML_DIR"/examples/mnist/model.msgpack "$ML_DIR"/examples/mnist/inp.msgpack build "$MNIST_OUTPUT" 25

RUST_LOG=debug "$ML_DIR"/target/release/time_circuit dlrm "$ML_DIR"/examples/dlrm/model.msgpack "$ML_DIR"/examples/dlrm/inp.msgpack build "$DLRM_OUTPUT" 25

# Sweep Var 50

RUST_LOG=debug "$ML_DIR"/target/release/time_circuit mnist "$ML_DIR"/examples/mnist/model.msgpack "$ML_DIR"/examples/mnist/inp.msgpack build "$MNIST_OUTPUT" 50

RUST_LOG=debug "$ML_DIR"/target/release/time_circuit dlrm "$ML_DIR"/examples/dlrm/model.msgpack "$ML_DIR"/examples/dlrm/inp.msgpack build "$DLRM_OUTPUT" 50