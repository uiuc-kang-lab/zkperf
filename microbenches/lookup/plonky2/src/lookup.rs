// Ref: https://github.com/0xPolygonZero/plonky2/blob/5800e6ad64751544f56bdf2ed2058bd85c20ac36/plonky2/src/lookup_test.rs
use rand::Rng;
use serde_json::json;
use std::{io::prelude::*, sync::Arc, fs::File, time::Instant};
use log::Level;
use itertools::Itertools;
use plonky2::field::types::Field;
use plonky2::iop::witness::{PartialWitness, WitnessWrite};
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::CircuitConfig;
use plonky2::plonk::config::{GenericConfig, PoseidonGoldilocksConfig};
use plonky2::plonk::prover::prove;
use plonky2::gates::lookup_table::LookupTable;
use plonky2::util::timing::TimingTree;

pub fn run_lookup(n: u16, k:usize){
    let mut rng = rand::thread_rng();
    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;
    let table_vec: Vec<u16> = (0..n).collect();
    let table: LookupTable = Arc::new((0..n).zip_eq(table_vec).collect());
    let config = CircuitConfig::standard_recursion_zk_config();
    let mut builder = CircuitBuilder::<F, D>::new(config);


    let mut look_val_vec: Vec<usize> = vec![];
    for _i in 0..k{
        look_val_vec.push(rng.gen_range(0..n).into())
    }
    let mut out_val_vec: Vec<usize> = vec![];
    for i in 0..k{
        out_val_vec.push(table[look_val_vec[i]].1.into())
    }

    // println!("{:?}", look_val_vec);
    // println!("{:?}", table);
    // println!("{:?}", out_val_vec);
    let table_index = builder.add_lookup_table_from_pairs(table);
    let mut initial_query_vec = vec![];
    for _i in 0..k{
        initial_query_vec.push(builder.add_virtual_target());
    }
    let mut lookup_output_vec = vec![];
    for i in 0..k{
        lookup_output_vec.push(builder.add_lookup_from_index(initial_query_vec[i], table_index));
    }
    let mut pw = PartialWitness::new();
    for i in 0..k{
        pw.set_target(initial_query_vec[i], F::from_canonical_usize(look_val_vec[i]))
    }

    let data = builder.build::<C>();
    let mut timing = TimingTree::new("prove", Level::Info);
    let time_start = Instant::now();
    let proof = prove(&data.prover_only, &data.common, pw, &mut timing).unwrap();
    let prover_time = time_start.elapsed();
    println!("Prover time: {:?}", prover_time);

    let time_start = Instant::now();
    data.verify(proof.clone()).unwrap();
    let verifier_time  = time_start.elapsed();
    println!("Verifier time: {:?}", verifier_time);

    let proof_bytes = proof.to_bytes();
    let proof_len = proof_bytes.len();
    println!("Proof size: {} bytes", proof_len);

    let results = json!({
        "Framework": "plonky2",
        "Backend": "PLONK-FRI",
        "TableSize": n,
        "NbQueries": k,
        "NbConstraints": "Not Valid",
        "ProverTime": prover_time.as_secs_f32(),
        "VerifierTime": verifier_time.as_nanos() as f32 / 1000000.,
        "ProofSize": proof_len
      });

      let json_string = serde_json::to_string(&results).unwrap();
      let output_path = "plonky2lookup_".to_owned() + &n.to_string() + "_" +&k.to_string()+"_" + ".json";
      let mut file = File::create(output_path).unwrap();
      let _ = file.write_all(json_string.as_bytes());
}