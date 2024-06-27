#!/bin/bash

sudo apt-get install build-essential cmake libgmp-dev libsodium-dev nasm curl m4
git clone https://github.com/iden3/rapidsnark
git submodule init
git submodule update
./build_gmp.sh host
mkdir build_prover && cd build_prover
cmake .. -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=../package
make -j4 && make install