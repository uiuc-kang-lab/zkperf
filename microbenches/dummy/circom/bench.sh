#!/bin/bash


N=1000
R=3

for i in $(seq 0 1 $R)
do 
    echo "Run size: $N"
    python3 gen_data.py $N
    circom dummy_main.circom --r1cs --wasm --sym --c --wat --output .
    node dummy_main_js/generate_witness.js dummy_main_js/dummy_main.wasm input.json witness.wtns
    npx snarkjs groth16 setup dummy_main.r1cs powers.ptau dummy_main_0.zkey
    npx snarkjs zkey beacon dummy_main_0.zkey dummy_main.zkey 0102030405060708090a0b0c0d0e0f101112231415161718221a1b1c1d1e1f 10 -n="Final Beacon phase2"
    npx snarkjs zkey verify dummy_main.r1cs powers.ptau dummy_main.zkey
    npx snarkjs zkey export verificationkey dummy_main.zkey vkey.json
    start=`date +%s%N`
    npx snarkjs groth16 prove dummy_main.zkey witness.wtns proof.json public.json
    end=`date +%s%N`
    echo "Proving time: ($(echo "scale=6; $((end-start))/1000000" | bc)ms)"
    npx snarkjs groth16 verify vkey.json public.json proof.json
    N=$(expr "$N" "*" 10)
done
