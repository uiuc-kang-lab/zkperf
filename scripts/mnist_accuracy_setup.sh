#!/bin/bash

INPUT_DIR="../halo2/zkml/examples/mnist/inputs"
OUTPUT_DIR="../halo2/zkml/examples/mnist/outputs"
CONFIG_MSGPACK="../halo2/zkml/examples/mnist/model.msgpack"
MODEL="../plonky2/ml/examples/mnist/model.tflite"

mkdir -p $INPUT_DIR
mkdir -p $OUTPUT_DIR

python mnist_data_to_npy.py "$INPUT_DIR"/npy

for file in "$INPUT_DIR"/npy/*; do
    if [ -f "$file" ]; then
        filename=$(basename -- "$file")
        filename="${filename%.*}"
        python input_converter.py --model_config $CONFIG_MSGPACK --inputs $file --output "$INPUT_DIR"/"$filename".msgpack
    fi
done
python tflite_output.py --model $MODEL --input "$INPUT_DIR"/npy --output "$OUTPUT_DIR"