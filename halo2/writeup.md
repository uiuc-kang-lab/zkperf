# Halo2 Circuits Description

## ECDSA
We use existing ECDSA gadget implemented in [Halo2Wrong](https://github.com/privacy-scaling-explorations/halo2wrong) for benchmark. Internally, each coordinate in sep256k1 point is represented using residue numeral system(RNS) due to different size of prime groups. (By default, Halo2 uses bn254 and bn254 prime field is strictly less than sep256k1 prime field). Each value in RNS is composed of 4 chunks of 68 bits. With this integer representation for coordinate values, the circuits executes arithmetic calculation in chunks and that forms the basics for elliptic curve operations. To reduce elliptic curve addition overhead, elliptic curve multiplication in halo2 gadget is completed by sliding window method. 

## Merkle Tree
Our merkle tree verification in Halo2 is built upon Keccak256 implementation from [Axioms](https://github.com/axiom-crypto/halo2-lib). It separates the hashing into three main phases: absorbing, squeezing and block permutation. Each nonlinear operation such as finding parity of the value and conducting bit wise operation in block permutation is indexed by lookup tables. Given the keccak256 implementation and the verification path, our implementation traverses the trajectory and constrains the result with public commitment of the tree

## DLRM
For DLRM model in Halo2, we replaces the word embedding as part of the private inputs. The main component is 6 fully connected layers. We use Freivald's algorithm to efficiently verify matrix multiplication, relying on randomness provided by Halo2 challenger API. Each output of the fully connected layer will be directed to a ReLU layer and that's done by lookup table. 