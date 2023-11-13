#!/bin/bash

OUTPUT_PATH=../breakdown
NAME="ecdsa_breakdown.txt"
EXECUTABLE="./target/release/breakdown_ecdsa"

cargo build --release --features "print-trace"

$EXECUTABLE > "$OUTPUT_PATH/$NAME"