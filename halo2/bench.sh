#!/bin/bash

cd ecdsa
./benchmark.sh
cd ../merkle_tree
./benchmark.sh
cd ../zkml
./benchmark_mnist.sh
./benchmark_dlrm.sh
cd ../ezkl
./bench_mnist.sh
./bench_dlrm.sh
