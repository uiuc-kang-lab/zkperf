use plonky2::plonk::config::{GenericConfig, PoseidonGoldilocksConfig};
use zkml::{model::ModelCircuit, utils::proving::time_circuit};

use jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn main() {
  env_logger::init();
  let circuit_type = std::env::args().nth(1).expect("circuit name");
  let config_fname = std::env::args().nth(2).expect("config file path");
  let inp_fname = std::env::args().nth(3).expect("input file path");
  let build_prove = std::env::args().nth(4).expect("build or prove");
  let outp_json = std::env::args().nth(5).expect("output json file path");
  let x = std::env::args().nth(6).expect("sweep variable");
  let sweep_variable = x.parse::<usize>().unwrap();
  if circuit_type != "mnist" &&  circuit_type != "dlrm" {
    panic!("Unsupported circuit type");
  }

  if build_prove != "build" && build_prove != "prove" {
    panic!("Must specify build or prove");
  }

  const D: usize = 2;
  type C = PoseidonGoldilocksConfig;
  type F = <C as GenericConfig<D>>::F;
  let (circuit, builder, pw) =
    ModelCircuit::generate_from_file::<F, C, D>(&config_fname, &inp_fname, &sweep_variable);
  time_circuit::<F, C, D>(circuit, builder, pw, circuit_type, build_prove, outp_json);
}
