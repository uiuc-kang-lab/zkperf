#!/bin/bash

cd ecdsa
./benchmark.sh
cd ../merkle_tree
./benchmark.sh
cd ../zkml
./verify_mnist.sh
./verify_dlrm.sh
