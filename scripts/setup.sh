#!/bin/bash

MAIN_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )/../

# circom
cd $MAIN_DIR/circom
cd ecdsa
./configure.sh
cd ../merkle_tree
./configure.sh
cd ../zkml
./configure.sh

# halo2
cd $MAIN_DIR/halo2
cd ecdsa
./configure.sh
cd ../merkle_tree
./configure.sh
mkdir -p params
cd ../zkml
./configure.sh
mkdir -p params_kzg
mkdir -p params_ipa
cd ../ezkl
./configure.sh

# plonky2
cd $MAIN_DIR/plonky2/ecdsa/
rustup override set nightly
cargo build --release
cd ../merkle_tree/keccak/
rustup override set nightly
cargo build --release
cd ../ml
rustup override set nightly
cargo build --release

