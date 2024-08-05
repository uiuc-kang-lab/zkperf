# ZKPerf: A ZK Proving Benchmark

ZKPerf benchmarks four circuits: ECDSA, Merkle Tree Membership Verification,
MNIST Convolutional Neural Network, and Deep Learning Recommendation Model
(DLRM) over four frameworks: Plonky2, Halo2, gnark, and circom-rapidsnark.

## Install dependencies

For Ubuntu:

From `$HOME` or the intended rapidsnark install location, run `circom/setup.sh`

```
./scripts/install.sh
source "$HOME/.cargo/env"
```

Otherwise, please follow the following instructions per framework:

### circom

Install rapidsnark from the "Dependencies" and "Compile prover in standalone
mode" sections https://github.com/iden3/rapidsnark/tree/main

For example, in Ubuntu:

```
sudo apt-get update
sudo apt-get install build-essential cmake libgmp-dev libsodium-dev nasm curl m4 npm

git submodule init
git submodule update
./build_gmp.sh host
mkdir build_prover && cd build_prover
cmake .. -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=../package
make -j4 && make install
```

### gnark

We have tested our circuits on Golang 1.21.0.

General installation instructions: https://go.dev/doc/install

In Ubuntu:

```
wget https://go.dev/dl/go1.21.0.linux-amd64.tar.gz
sudo tar -C /usr/local -xzf go1.21.0.linux-amd64.tar.gz
echo 'export PATH=$PATH:/usr/local/go/bin' >> ~/.bashrc
```

### plonky2/halo2 (Rust)

Install `rustup`:

```
curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
source "$HOME/.cargo/env"
```

Install ML circuit utilities:

```
curl -O https://bootstrap.pypa.io/get-pip.py
python3 get-pip.py --user
pip install tensorflow tflite msgpack
echo 'alias python="python3"' >> ~/.bashrc
source ~/.bashrc
```

## Setup

This performs additional configuration and building for some circuits:

```
./scripts/setup.sh
```

## Run the benchmark

All scripts will write `*.json` files containing stats for proving time (s),
verifying time (ms), proof size (Bytes), and proving memory consumption (MB).

### circom

The benchmark scripts assume that `rapidsnark` was installed in `$HOME`. If they
are installed elsewhere, then the path `~/rapidsnark` in `RAPIDSNARK_PROVER` and
`RAPIDSNARK_VERIFIER` will need to be changed in
`circom/ecdsa/benchmark_rapidsnark.sh`,
`circom/merkle_tree/benchmark_rapidsnark.sh`,
`circom/zkml/verify_mnist_rapidsnark.sh`,
`circom/zkml/verify_dlrm_rapidsnark.sh` to the correct path.

```
cd circom
./bench_rapidsnark.sh
```

Outputs will be in `circom/<circuit-name>/measurement.json`

### gnark

```
cd gnark
./run.sh
```

Outputs will be in `gnark/<circuit-name>-{groth16, plonk}.json`

### plonky2

```
cd plonky2
./bench.sh
```

Outputs will be in `plonky2/<circuit-name>/<circuit-name>.json`

### halo2

```
cd halo2
./bench.sh
```

Outputs will be in `halo2/<circuit-name>/<circuit-name>_measurement.json`

### Lookup microbenchmarks

Plonky2, Halo2, and gnark support lookups

```
cd plonky2
cargo run
cd ../halo2
cargo build --release
./target/release/lookup
cd ../gnark
./run.sh
```

### Column sweep benchmarks

Halo2 DLRM and all circuits in Plonky2 have column sweep experiments

```
cd halo2/zkml
./col_experiments.sh
cd ../../plonky2
./col_sweep.sh
```
