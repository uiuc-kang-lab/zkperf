#!/bin/bash

OUTPUT_DIR="experiments"
OUTPUT_PATH=../breakdown


if [ ! -d "$OUTPUT_DIR" ]; then
    mkdir -p "$OUTPUT_DIR"
fi

if [ ! -d "./params_kzg" ]; then
    echo "No params directory found. Creating the directory..."
    mkdir -p "./params_kzg"
fi

cargo build --release --features "print-trace"

NUM_COLS=48
K=16

for i in $(seq $NUM_COLS -6 6)
do
    echo "Test DLRM with $i Columns"
    python3 python/converter.py --model examples/dlrm/dlrm_mlp_checked_float32.tflite --model_output examples/dlrm/dlrm_model.msgpack --config python/dlrm_config.msgpack --scale_factor 512 --num_cols "$i" --num_witness_cols $(expr "$i" / 2) --k "$K"
    ./target/release/breakdown_circuit examples/dlrm/dlrm_model.msgpack examples/dlrm/dlrm_input.msgpack kzg > $OUTPUT_DIR/dlrm_breakdown_"$i"_$K.txt 2>error.txt
    while [ $(cat error.txt | grep -c "NotEnoughRowsAvailable") -ne 0 ];
    do
        rm $OUTPUT_DIR/dlrm_breakdown_"$i"_$K.txt
        rm error.txt
        K=$(expr "$K" + 1)
        python3 python/converter.py --model examples/dlrm/dlrm_mlp_checked_float32.tflite --model_output examples/dlrm/dlrm_model.msgpack --config python/dlrm_config.msgpack --scale_factor 512 --num_cols $i --num_witness_cols $(expr "$i" / 2) --k "$K"
        ./target/release/breakdown_circuit examples/dlrm/dlrm_model.msgpack examples/dlrm/dlrm_input.msgpack kzg > $OUTPUT_DIR/dlrm_breakdown_"$i"_$K.txt 2>error.txt
    done
    python3 "$OUTPUT_PATH/breakdown.py" dlrm dlrm_breakdown_"$i"_$K.txt "$OUTPUT_PATH/dlrm_breakdown_"$i"_$K.json"
done