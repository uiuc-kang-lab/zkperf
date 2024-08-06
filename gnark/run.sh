go build -o gmc
# Merkle
./gmc -circuit MerkleTree -backend Groth16 -step Setup -curve BN254 -outputPath merkle_groth16 
{ /usr/bin/time -v ./gmc -circuit MerkleTree -backend Groth16 -step Prover -curve BN254 -outputPath merkle_groth16; } 2> /tmp/test
echo "$(jq --arg tmp $(echo "scale=6; $(cat /tmp/test | grep "Maximum resident set size" | tr -d -c 0-9)/1024" | bc) '.+={"MemoryConsumption": $tmp }' "merkle_groth16.json")" > "merkle_groth16.json"
./gmc -circuit MerkleTree -backend Groth16 -step Verifier -curve BN254 -outputPath merkle_groth16

./gmc -circuit MerkleTree -backend Plonk -step Setup -curve BN254 -outputPath merkle_plonk
{ /usr/bin/time -v ./gmc -circuit MerkleTree -backend Plonk -step Prover -curve BN254 -outputPath merkle_plonk; } 2> /tmp/test
echo "$(jq --arg tmp $(echo "scale=6; $(cat /tmp/test | grep "Maximum resident set size" | tr -d -c 0-9)/1024" | bc) '.+={"MemoryConsumption": $tmp }' "merkle_plonk.json")" > "merkle_plonk.json"
./gmc -circuit MerkleTree -backend Plonk -step Verifier -curve BN254 -outputPath merkle_plonk

# MNIST
./gmc -circuit MNIST -backend Groth16 -step Setup -curve BN254 -outputPath mnist_groth16 
{ /usr/bin/time -v ./gmc -circuit MNIST -backend Groth16 -step Prover -curve BN254 -outputPath mnist_groth16; } 2> /tmp/test
echo "$(jq --arg tmp $(echo "scale=6; $(cat /tmp/test | grep "Maximum resident set size" | tr -d -c 0-9)/1024" | bc) '.+={"MemoryConsumption": $tmp }' "mnist_groth16.json")" > "mnist_groth16.json"
./gmc -circuit MNIST -backend Groth16 -step Verifier -curve BN254 -outputPath mnist_groth16

./gmc -circuit MNIST -backend Plonk -step Setup -curve BN254 -outputPath mnist_plonk
{ /usr/bin/time -v ./gmc -circuit MNIST -backend Plonk -step Prover -curve BN254 -outputPath mnist_plonk; } 2> /tmp/test
echo "$(jq --arg tmp $(echo "scale=6; $(cat /tmp/test | grep "Maximum resident set size" | tr -d -c 0-9)/1024" | bc) '.+={"MemoryConsumption": $tmp }' "mnist_plonk.json")" > "mnist_plonk.json"
./gmc -circuit MNIST -backend Plonk -step Verifier -curve BN254 -outputPath mnist_plonk

# DLRM
./gmc -circuit DLRM -backend Groth16 -step Setup -curve BN254 -outputPath dlrm_groth16 
{ /usr/bin/time -v ./gmc -circuit DLRM -backend Groth16 -step Prover -curve BN254 -outputPath dlrm_groth16; } 2> /tmp/test
echo "$(jq --arg tmp $(echo "scale=6; $(cat /tmp/test | grep "Maximum resident set size" | tr -d -c 0-9)/1024" | bc) '.+={"MemoryConsumption": $tmp }' "dlrm_groth16.json")" > "dlrm_groth16.json"
./gmc -circuit DLRM -backend Groth16 -step Verifier -curve BN254 -outputPath dlrm_groth16

./gmc -circuit DLRM -backend Plonk -step Setup -curve BN254 -outputPath dlrm_plonk
{ /usr/bin/time -v ./gmc -circuit DLRM -backend Plonk -step Prover -curve BN254 -outputPath dlrm_plonk; } 2> /tmp/test
echo "$(jq --arg tmp $(echo "scale=6; $(cat /tmp/test | grep "Maximum resident set size" | tr -d -c 0-9)/1024" | bc) '.+={"MemoryConsumption": $tmp }' "dlrm_plonk.json")" > "dlrm_plonk.json"
./gmc -circuit DLRM -backend Plonk -step Verifier -curve BN254 -outputPath dlrm_plonk

go build -o gmc
# Poseidon Merkle
./gmc -circuit PoseidonMerkle -backend Groth16 -step Setup -curve BN254 -outputPath poseidon_merkle_groth16 
{ /usr/bin/time -v ./gmc -circuit PoseidonMerkle -backend Groth16 -step Prover -curve BN254 -outputPath poseidon_merkle_groth16; } 2> /tmp/test
echo "$(jq --arg tmp $(echo "scale=6; $(cat /tmp/test | grep "Maximum resident set size" | tr -d -c 0-9)/1024" | bc) '.+={"MemoryConsumption": $tmp }' "poseidon_merkle_groth16.json")" > "poseidon_merkle_groth16.json"
./gmc -circuit PoseidonMerkle -backend Groth16 -step Verifier -curve BN254 -outputPath poseidon_merkle_groth16

./gmc -circuit PoseidonMerkle -backend Plonk -step Setup -curve BN254 -outputPath poseidon_merkle_plonk
{ /usr/bin/time -v ./gmc -circuit PoseidonMerkle -backend Plonk -step Prover -curve BN254 -outputPath poseidon_merkle_plonk; } 2> /tmp/test
echo "$(jq --arg tmp $(echo "scale=6; $(cat /tmp/test | grep "Maximum resident set size" | tr -d -c 0-9)/1024" | bc) '.+={"MemoryConsumption": $tmp }' "poseidon_merkle_plonk.json")" > "poseidon_merkle_plonk.json"
./gmc -circuit PoseidonMerkle -backend Plonk -step Verifier -curve BN254 -outputPath poseidon_merkle_plonk
