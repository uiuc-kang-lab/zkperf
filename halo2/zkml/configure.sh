#!/bin/bash

rustup toolchain install nightly-2023-07-12
rustup override set nightly-2023-07-12

cargo build --release