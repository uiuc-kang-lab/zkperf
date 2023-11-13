#!/bin/bash

rustup toolchain install nightly-2022-10-28
rustup override set nightly-2022-10-28
cargo update -p clap --precise 4.0.13
cargo update -p toml_datetime --precise 0.6.3

cargo build --release