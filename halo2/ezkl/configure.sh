#!/bin/bash

rustup toolchain install nightly-2024-07-31
rustup override set nightly-2024-07-31

cargo update -p "revm-interpreter" --precise 1.1.2
cargo update -p "revm-precompile" --precise 2.0.3
cargo update -p "revm-primitives" --precise 1.1.2
cargo update -p "subtle" --precise 2.5.0

cargo build --release --features "stats"
