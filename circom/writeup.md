# Circom Circuit Description

## ECDSA
We are using [circom-ecdsa](https://github.com/0xPARC/circom-ecdsa) implementation for circom ECDSA benchmark. It represents scalar number in 4 chunks of 64 bits value. In scalar multiplication, the number is formulated into polynomial similar to bitwise composition with different base and that reduces the multiplication to linear in terms of the number of chunks. For elliptic curve multiplication, it uses the cached window method with a window size of 8 to obtain the least number of constraints.


## Merkle
For circom merkle tree benchmark, we are using [keccak256-circom](https://github.com/vocdoni/keccak256-circom) for hash implementation. Unfortunately, circom does not implement any lookup function, every logical operation like shift and or in permutation phase has to be done bit by bit. With hash implementation as a blackbox, given verification hashes and index, our merkle circuit goes through the path, computes the final hash and constrains it to be the same as public merkle root. 
