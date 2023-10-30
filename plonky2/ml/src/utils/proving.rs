use log::Level;
use ndarray::Array;
use plonky2::plonk::prover::prove;
use plonky2::util::timing::TimingTree;
use serde_json::json;
use std::io::prelude::*;
use std::{fs::File, io::BufWriter, time::Instant};

use plonky2::{
  field::extension::Extendable,
  hash::hash_types::RichField,
  iop::generator::generate_partial_witness,
  iop::witness::{PartialWitness, Witness},
  plonk::{circuit_builder::CircuitBuilder, config::GenericConfig},
};

use crate::{gadgets::gadget::convert_to_u64, model::ModelCircuit};

pub fn time_circuit<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>  + 'static, const D: usize>(
  circuit: ModelCircuit,
  mut builder: CircuitBuilder<F, D>,
  pw: PartialWitness<F>,
  outp_json: String,
) {
  let result_targets = circuit.construct::<F, C, D>(&mut builder);

  println!("building circuit");
  let start = Instant::now();
  let data = builder.build::<C>();
  let build_duration = start.elapsed();
  println!("circuit build duration: {:?}", build_duration);

  let pw2 = pw.clone();

  println!("proving circuit");
  let mut timing = TimingTree::new("prove", Level::Info);
  let proof = prove::<F, C, D>(
    &data.prover_only,
    &data.common,
    pw,
    &mut timing).unwrap();
  timing.pop();
  timing.print();

  let proof_duration = timing.duration();
  println!("Proving time: {:?}", proof_duration);

  let proof_bytes = proof.to_bytes();
  let proof_len = proof_bytes.len();
  println!("Proof size: {} bytes", proof_len);

  println!("verifying circuit");
  let mut timing = TimingTree::new("verify", Level::Info);
  data.verify(proof.clone()).expect("verify error");
  timing.pop();
  timing.print();

  let verify_duration = timing.duration();
  println!("Verifying time: {:?}", verify_duration);

  println!("generating witness");
  let witness = generate_partial_witness(pw2, &data.prover_only, &data.common);

  if result_targets.len() > 0 {
    let out = Array::from_iter(result_targets[0].iter().cloned());
    let mut values: Vec<i64> = vec![];
    for (idx, t) in out.iter().enumerate() {
      let value = witness.get_target(**t);
      let bias: i64 = 1 << 60 as i64;

      let v_pos = value + F::from_canonical_u64(bias as u64);
      let v = convert_to_u64(&v_pos) as i64 - bias as i64;
      println!("final out [{}] x: {}", idx, v);
      values.push(v);
    }
    let out_fname = "out.msgpack";
    let f = File::create(out_fname).unwrap();
    let mut buf = BufWriter::new(f);
    rmp_serde::encode::write_named(&mut buf, &values).unwrap();
  }

  println!("writing results");
  let results = json!({
    "Framework": "plonky2",
    "Backend": "Plonk+FRI",
    "Curve": "NaN",
    "ProverTime": proof_duration.as_secs_f32(),
    "VerifierTime": verify_duration.as_nanos() as f32 / 1000000.,
    "ProofSize": proof_len
  });

  let json_string = serde_json::to_string(&results).unwrap();

  let mut file = File::create(outp_json).unwrap();
  let _ = file.write_all(json_string.as_bytes());

}
