#!/bin/bash

OUTPUT_DIR="experiments"

if [ ! -d "$OUTPUT_DIR" ]; then
    mkdir -p "$OUTPUT_DIR"
fi

if [ ! -d "./params_kzg" ]; then
    echo "No params directory found. Creating the directory..."
    mkdir -p "./params_kzg"
fi

cargo build --release

NUM_COLS=120
K=15

for i in $(seq $NUM_COLS -6 6)
do
    echo "Test DLRM with $i Columns"
    python3 python/converter.py --model examples/dlrm/dlrm_mlp_checked_float32.tflite --model_output examples/dlrm/dlrm_model.msgpack --config python/dlrm_config.msgpack --scale_factor 512 --num_cols "$i" --num_witness_cols $(expr "$i" / 2) --k "$K"
    ./target/release/time_circuit examples/dlrm/dlrm_model.msgpack examples/dlrm/dlrm_input.msgpack kzg > $OUTPUT_DIR/dlrm_test_"$i"_$K.txt 2>error.txt
    while [ $(cat error.txt | grep -c "NotEnoughRowsAvailable") -ne 0 ];
    do
        rm $OUTPUT_DIR/dlrm_test_"$i"_$K.txt
        rm error.txt
        K=$(expr "$K" + 1)
        python3 python/converter.py --model examples/dlrm/dlrm_mlp_checked_float32.tflite --model_output examples/dlrm/dlrm_model.msgpack --config python/dlrm_config.msgpack --scale_factor 512 --num_cols $i --num_witness_cols $(expr "$i" / 2) --k "$K"
        ./target/release/time_circuit examples/dlrm/dlrm_model.msgpack examples/dlrm/dlrm_input.msgpack kzg > $OUTPUT_DIR/dlrm_test_"$i"_$K.txt 2>error.txt
    done
done