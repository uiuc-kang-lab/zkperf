#!/bin/bash

INPUT_DIR="examples/mnist/inputs"
OUTPUT_DIR="examples/mnist/outputs"
CONFIG_MSGPACK="examples/mnist/model.msgpack"
MODEL="../plonky2/ml/examples/mnist/model.tflite"

cd ../halo2/zkml
mkdir -p outputs

for file in "$INPUT_DIR"/*; do
    if [ -f "$file" ]; then
        filename=$(basename -- "$file")
        filename="${filename%.*}"
        ./target/release/time_circuit $CONFIG_MSGPACK $file kzg
        cp out.msgpack outputs/mnist/"$filename".msgpack
    fi
done

cd ../../scripts
python accuracy_stats.py ../halo2/zkml/outputs/mnist ../halo2/zkml/examples/mnist/output