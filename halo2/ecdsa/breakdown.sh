#!/bin/bash

OUTPUT_PATH=../breakdown
INPUT="ecdsa_breakdown.txt"
OUTPUT="ecdsa_breakdown.json"
EXECUTABLE="./target/release/breakdown_ecdsa"

cargo build --release --features "print-trace"

$EXECUTABLE > "$INPUT"
python "$OUTPUT_PATH/breakdown.py" ecdsa $INPUT "$OUTPUT_PATH/$OUTPUT"