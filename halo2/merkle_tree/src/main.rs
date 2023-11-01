#![allow(unused_imports)]
#![feature(return_position_impl_trait_in_trait)]
#![allow(incomplete_features)]
pub mod keccak_merkle;
pub mod keccak_one;
pub mod cmd;
pub mod scaffold;
pub mod test_circuit;


use cmd::Cli;
use clap::Parser;
use keccak_merkle::merkle_tree_keccak;
use keccak_one::compute_fixed_len_keccak;
use scaffold::run_eth;
// use test_circuit::run_merkle;


fn main() {
    env_logger::init();
    let args = Cli::parse();
    run_eth(merkle_tree_keccak, args);
}
