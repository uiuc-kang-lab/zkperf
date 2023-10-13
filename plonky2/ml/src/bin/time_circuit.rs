use plonky2::plonk::config::{PoseidonGoldilocksConfig, GenericConfig};
use zkml::{
  model::ModelCircuit,
  utils::proving::time_circuit,
};

fn main() {
  let config_fname = std::env::args().nth(1).expect("config file path");
  let inp_fname = std::env::args().nth(2).expect("input file path");

  const D: usize = 2;
  type C = PoseidonGoldilocksConfig;
  type F = <C as GenericConfig<D>>::F;
  let (circuit, builder, pw) = ModelCircuit::generate_from_file(&config_fname, &inp_fname);
  time_circuit::<F, C, D>(circuit, builder, pw);
}
