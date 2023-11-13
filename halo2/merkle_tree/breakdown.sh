#!/bin/bash

OUTPUT_PATH=../breakdown
NAME="merkle_breakdown.txt"
EXECUTABLE="./target/release/merkle_tree"

cargo build --release --features "print-trace"

$EXECUTABLE --name mt_keccak -k 12 full > "$OUTPUT_PATH/$NAME"