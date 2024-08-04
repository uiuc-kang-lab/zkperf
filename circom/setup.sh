#!/bin/bash

sudo yum install gcc gcc-c++ make cmake gmp-devel php-sodium nasm m4
git clone https://github.com/iden3/rapidsnark
cd rapidsnark
git submodule init
git submodule update
./build_gmp.sh host
mkdir build_prover && cd build_prover
cmake .. -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=../package
make -j4 && make install
