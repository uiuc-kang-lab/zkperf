#!/bin/bash

OUTPUT_PATH=../breakdown
INPUT="merkle_breakdown.txt"
OUTPUT="merkle_breakdown.json"
EXECUTABLE="./target/release/merkle_tree"

cargo build --release --features "print-trace"

$EXECUTABLE --name mt_keccak -k 12 full > $INPUT
python "$OUTPUT_PATH/breakdown.py" merkle $INPUT "$OUTPUT_PATH/$OUTPUT"