#!/bin/bash

cd ecdsa
./benchmark_rapidsnark.sh
cd ../merkle_tree
./benchmark_rapidsnark.sh
cd ../zkml
./verify_mnist_rapidsnark.sh
./verify_dlrm_rapidsnark.sh
