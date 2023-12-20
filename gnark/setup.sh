# Setup Script

go build -o gmc
./gmc -circuit MerkleTree -backend Groth16 -step Setup -curve BN254 -outputPath merkle_groth16 
./gmc -circuit MerkleTree -backend Plonk -step Setup -curve BN254 -outputPath merkle_plonk

./gmc -circuit MNIST -backend Groth16 -step Setup -curve BN254 -outputPath mnist_groth16 
./gmc -circuit MNIST -backend Plonk -step Setup -curve BN254 -outputPath mnist_groth16 

./gmc -circuit DLRM -backend Groth16 -step Setup -curve BN254 -outputPath dlrm_groth16 
./gmc -circuit DLRM -backend Plonk -step Setup -curve BN254 -outputPath dlrm_groth16 
