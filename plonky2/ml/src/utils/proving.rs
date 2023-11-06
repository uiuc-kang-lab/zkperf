use log::Level;
use ndarray::Array;
use plonky2::fri::oracle::PolynomialBatch;
use plonky2::iop::challenger::Challenger;
use plonky2::iop::witness::WitnessWrite;
use plonky2::plonk::circuit_data::CircuitData;
use plonky2::plonk::prover::prove;
use plonky2::plonk::prover::set_lookup_wires;
use plonky2::util::timing::TimingTree;
use plonky2_field::polynomial::PolynomialValues;
use plonky2_maybe_rayon::MaybeParIter;
use plonky2_maybe_rayon::ParallelIterator;
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

pub fn time_circuit<
  F: RichField + Extendable<D>,
  C: GenericConfig<D, F = F> + 'static,
  const D: usize,
>(
  circuit: ModelCircuit,
  mut builder: CircuitBuilder<F, D>,
  mut pw: PartialWitness<F>,
  outp_json: String,
) {
  let (result_targets, rand_targets) = circuit.construct::<F, C, D>(&mut builder);

  println!("building circuit");
  let start = Instant::now();
  let data = builder.build::<C>();
  let build_duration = start.elapsed();
  println!("circuit build duration: {:?}", build_duration);

  println!("generating random values");
  // set the pw_commit rand targets to 0
  let mut pw_commit = pw.clone();
  for t in &rand_targets {
    pw_commit.set_target(*t, F::ZERO);
  }

  let CircuitData {
    prover_only: ref prover_data,
    verifier_only: _,
    common: ref common_data,
  } = data;

  let mut partition_witness = generate_partial_witness(pw_commit, &prover_data, &common_data);
  set_lookup_wires(&prover_data, &common_data, &mut partition_witness);

  let witness = partition_witness.full_witness();

  let wires_values: Vec<PolynomialValues<F>> = witness
    .wire_values
    .par_iter()
    .map(|column| PolynomialValues::new(column.clone()))
    .collect();

  let wires_commitment = PolynomialBatch::<F, C, D>::from_values(
    wires_values,
    common_data.config.fri_config.rate_bits,
    common_data.config.zero_knowledge,
    common_data.config.fri_config.cap_height,
    &mut TimingTree::default(),
    prover_data.fft_root_table.as_ref(),
  );

  let mut challenger = Challenger::<F, C::Hasher>::new();

  // Observe the instance.
  challenger.observe_cap::<C::Hasher>(&wires_commitment.merkle_tree.cap);
  let rand_values = challenger.get_n_challenges(rand_targets.len());

  for i in 0..rand_targets.len() {
    pw.set_target(rand_targets[i], rand_values[i]);
  }

  let pw2 = pw.clone();

  println!("proving circuit");
  let mut timing = TimingTree::new("prove", Level::Info);
  let proof = prove::<F, C, D>(&prover_data, &common_data, pw, &mut timing).unwrap();
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
  let witness = generate_partial_witness(pw2, &prover_data, &common_data);

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
