use plonky2::plonk::config::{PoseidonGoldilocksConfig, GenericConfig};
use zkml::{
  model::ModelCircuit,
  utils::proving::time_circuit,
};

fn main() {
  let config_fname = std::env::args().nth(1).expect("config file path");
  let inp_fname = std::env::args().nth(2).expect("input file path");
  // let kzg_or_ipa = std::env::args().nth(3).expect("kzg or ipa");

  // if kzg_or_ipa != "kzg" && kzg_or_ipa != "ipa" {
  //   panic!("Must specify kzg or ipa");
  // }

  const D: usize = 2;
  type C = PoseidonGoldilocksConfig;
  type F = <C as GenericConfig<D>>::F;
  let (circuit, builder, pw) = ModelCircuit::generate_from_file(&config_fname, &inp_fname);
  time_circuit::<F, C, D>(circuit, builder, pw);

  // if kzg_or_ipa == "kzg" {
  //   let circuit = ModelCircuit::<Fr>::generate_from_file(&config_fname, &inp_fname);
  //   time_circuit_kzg(circuit);
  // } else {
  //   let circuit = ModelCircuit::<Fp>::generate_from_file(&config_fname, &inp_fname);
  //   time_circuit_ipa(circuit);
  // }
}
