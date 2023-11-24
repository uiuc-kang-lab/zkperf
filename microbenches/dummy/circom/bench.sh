#!/bin/bash


N=1000

python3 gen_data.py $N

echo "****COMPILING CIRCUIT****"
circom dummy_main.circom --r1cs --wasm --sym --c --wat --output .

echo "****GENERATING WITNESS FOR SAMPLE INPUT****"
node dummy_main_js/generate_witness.js dummy_main_js/dummy_main.wasm input.json witness.wtns

echo "****GENERATING ZKEY 0****"
npx snarkjs groth16 setup dummy_main.r1cs powers.ptau dummy_main_0.zkey

echo "****GENERATING FINAL ZKEY****"
npx snarkjs zkey beacon dummy_main_0.zkey dummy_main.zkey 0102030405060708090a0b0c0d0e0f101112231415161718221a1b1c1d1e1f 10 -n="Final Beacon phase2"

echo "****VERIFYING FINAL ZKEY****"
npx snarkjs zkey verify dummy_main.r1cs powers.ptau dummy_main.zkey

echo "** Exporting vkey"
npx snarkjs zkey export verificationkey dummy_main.zkey vkey.json

echo "****GENERATING PROOF FOR SAMPLE INPUT****"
start=`date +%s%N`
npx snarkjs groth16 prove dummy_main.zkey witness.wtns proof.json public.json
end=`date +%s%N`
echo "DONE ($(echo "scale=6; $((end-start))/1000000" | bc)ms)"


echo "****VERIFYING PROOF FOR SAMPLE INPUT****"
npx snarkjs groth16 verify vkey.json public.json proof.json

