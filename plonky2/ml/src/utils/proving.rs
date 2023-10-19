use ndarray::Array;
use std::{fs::File, io::BufWriter, time::Instant};

use plonky2::{
  field::extension::Extendable,
  hash::hash_types::RichField,
  iop::generator::generate_partial_witness,
  iop::witness::{PartialWitness, Witness},
  plonk::{
    circuit_builder::CircuitBuilder,
    config::GenericConfig,
  },
};

use crate::{gadgets::gadget::convert_to_u64, model::ModelCircuit};

pub fn time_circuit<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize>(
  circuit: ModelCircuit,
  mut builder: CircuitBuilder<F, D>,
  pw: PartialWitness<F>,
) {
  let result_targets = circuit.construct(&mut builder);
  let pw2 = pw.clone();

  println!("building circuit");
  let start = Instant::now();
  let data = builder.build::<C>();
  let build_duration = start.elapsed();
  println!("circuit build duration: {:?}", build_duration);

  let proof = data.prove(pw).unwrap();
  let proof_duration = start.elapsed();
  println!("Proving time: {:?}", proof_duration - build_duration);

  let proof_bytes = proof.to_bytes();
  println!("Proof size: {} bytes", proof_bytes.len());

  data.verify(proof.clone()).expect("verify error");
  let verify_duration = start.elapsed();
  println!("Verifying time: {:?}", verify_duration - proof_duration);

  println!("generating witness");
  let witness = generate_partial_witness(pw2, &data.prover_only, &data.common);

  if result_targets.len() > 0 {
    let out = Array::from_iter(result_targets[0].iter().cloned());
    let mut values: Vec<i64> = vec![];
    for (idx, t) in out.iter().enumerate() {
      let value = witness.get_target(**t);
      let bias: i64  = 1 << 60 as i64;

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
}
