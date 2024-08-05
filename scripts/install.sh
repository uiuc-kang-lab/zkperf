#!/bin/bash

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
