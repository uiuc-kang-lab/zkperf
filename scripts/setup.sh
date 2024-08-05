#!/bin/bash

MAIN_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )/../

sudo apt-get update
sudo apt-get install build-essential cmake libgmp-dev libsodium-dev nasm curl m4 npm

curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
source "$HOME/.cargo/env"

wget https://go.dev/dl/go1.21.0.linux-amd64.tar.gz
sudo tar -C /usr/local -xzf go1.21.0.linux-amd64.tar.gz
echo 'export PATH=$PATH:/usr/local/go/bin' >> ~/.bashrc

curl -O https://bootstrap.pypa.io/get-pip.py
python3 get-pip.py --user
pip install tensorflow tflite msgpack
echo 'alias python="python3"' >> ~/.bashrc
source ~/.bashrc

cd $MAIN_DIR/circom
git clone https://github.com/iden3/circom.git
cd circom
cargo build --release
cargo install --path circom
npm install -g snarkjs
cd ../ecdsa
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
cd ../zkml
./configure.sh
mkdir params_kzg
mkdir params_ipa
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

