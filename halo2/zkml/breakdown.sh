#!/bin/bash

OUTPUT_PATH=../breakdown

DLRM_INPUT="dlrm_breakdown.txt"
DLRM_OUTPUT="dlrm_breakdown.json"
DLRM_MODEL="./examples/dlrm/dlrm_model.msgpack"
DLRM_INPUT_MSG="./examples/dlrm/dlrm_input.msgpack"

MNIST_INPUT="mnist_breakdown.txt"
MNIST_OUTPUT="mnist_breakdown.json"
MNIST_MODEL="./examples/mnist/model.msgpack"
MNIST_INPUT_MSG="./examples/mnist/inp.msgpack"

EXECUTABLE="./target/release/breakdown_circuit"

cargo build --release --features "print-trace"

$EXECUTABLE $MNIST_MODEL $MNIST_INPUT_MSG > $MNIST_INPUT
python "$OUTPUT_PATH/breakdown.py" mnist $MNIST_INPUT "$OUTPUT_PATH/$MNIST_OUTPUT"

$EXECUTABLE $DLRM_MODEL $DLRM_INPUT_MSG > $DLRM_INPUT
python "$OUTPUT_PATH/breakdown.py" dlrm $DLRM_INPUT "$OUTPUT_PATH/$DLRM_OUTPUT"

