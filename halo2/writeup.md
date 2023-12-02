# Halo2 Circuits Description

## ECDSA
We use existing ECDSA gadget implemented in [Halo2Wrong](https://github.com/privacy-scaling-explorations/halo2wrong) for benchmark. Internally, each coordinate in sep256k1 point is represented by value of 65 bits and it is chunked into 4 pieces with an overflow bit. Each chunk has 16 bits and using a lookup table can efficiently enforce this size constraint. Given the decomposition, it executes large integer arithmetic calculation in chunks and that forms the basics for elliptic curve operations. To reduce elliptic curve addition overhead, elliptic curve multiplication in halo2 gadget is completed by windowed method. 

## Merkle Tree
