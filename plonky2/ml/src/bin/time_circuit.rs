use plonky2::plonk::config::{GenericConfig, KeccakGoldilocksConfig};
use zkml::{
  model::ModelCircuit,
  utils::proving::time_circuit,
};

use jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn main() {
  env_logger::init();
  let config_fname = std::env::args().nth(1).expect("config file path");
  let inp_fname = std::env::args().nth(2).expect("input file path");
  let outp_json = std::env::args().nth(3).expect("output json file path");

  const D: usize = 2;
  type C = KeccakGoldilocksConfig;
  type F = <C as GenericConfig<D>>::F;
  let (circuit, builder, pw) = ModelCircuit::generate_from_file::<F, C, D>(&config_fname, &inp_fname);
  time_circuit::<F, C, D>(circuit, builder, pw, outp_json);
}