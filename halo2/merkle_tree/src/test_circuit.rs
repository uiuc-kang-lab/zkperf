#![allow(unused_imports)]
use halo2_base::{
    gates::{
        flex_gate::{FlexGateConfig, GateStrategy, GateChip, GateInstructions},
        range::RangeChip,
    },
    utils::{fe_to_biguint, fs::gen_srs, value_to_option, ScalarField},
    SKIP_FIRST_PASS,
    halo2_proofs::{
        arithmetic::FieldExt,
        circuit::{Layouter, SimpleFloorPlanner, Value},
        dev::MockProver,
        halo2curves::bn256::{Bn256, Fr, G1Affine},
        plonk::*,
        poly::commitment::{Params, ParamsProver},
        poly::kzg::{
            commitment::{KZGCommitmentScheme, ParamsKZG},
            multiopen::{ProverGWC, ProverSHPLONK, VerifierSHPLONK},
            strategy::SingleStrategy,
        },
        transcript::{Blake2bRead, Blake2bWrite, Challenge255},
        transcript::{TranscriptReadBuffer, TranscriptWriterBuffer},
    }
};
use axiom_eth::{
    Field,
    keccak::{KeccakCircuitBuilder, FnSynthesize, KeccakChip, FixedLenRLCs, VarLenRLCs},
    rlp::{
        RlpChip,
        builder::RlcThreadBuilder,
        rlc::RlcConfig
    }
};
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    env::{var, set_var},
    fs::File,
    path::PathBuf
};
use crate::keccak_merkle::CircuitInput;
use crate::Cli;

fn test_keccak_merkle_circuit<F: Field>(
    k: u32,
    mut builder: RlcThreadBuilder<F>,
    input: CircuitInput,
) -> KeccakCircuitBuilder<F, impl FnSynthesize<F>>{
    let prover = builder.witness_gen_only();
    let range = RangeChip::<F>::default(8);
    let keccak = RefCell::<KeccakChip<F>>::default();
    let ctx = builder.gate_builder.main(0);

    let bitify_gate = GateChip::<F>::default();
    let selector_gate = GateChip::<F>::default();

    let index = ctx.load_witness(F::from(input.index));
    
    let sels = bitify_gate.num_to_bits(ctx, index, (input.n_levels-1) as usize);
    
    // let mut init_input = input.key.clone();
    // init_input.extend(input.val.iter().cloned());
    // let bytes_assigned = ctx.assign_witnesses(init_input.iter().map(|byte| F::from(*byte as u64)));
    
    // let input_hash_idx = keccak.borrow_mut().keccak_fixed_len(ctx, &range.gate, bytes_assigned, Some(init_input));

    // let mut curr = keccak.borrow_mut().fixed_len_queries[input_hash_idx].output_assigned.clone();
    let mut curr = ctx.assign_witnesses(input.child.iter().map(|byte| F::from(*byte as u64)));

    for i in 0..(input.n_levels-1) as usize {
        let sibling = ctx.assign_witnesses(input.siblings[i].iter().map(|x| F::from((*x) as u64)));
        let mut l_list = vec![];
        let mut r_list = vec![];
        for j in 0..32 {

            let l_o = selector_gate.select(ctx, sibling[j], curr[j], sels[i]);
            let r_o = selector_gate.select(ctx, curr[j], sibling[j], sels[i]);
            l_list.push(l_o);
            r_list.push(r_o);
        }
        l_list.extend(r_list.iter());
        let parent_hash_idx = keccak.borrow_mut().keccak_fixed_len(ctx, &range.gate, l_list, None);
        curr = keccak.borrow_mut().fixed_len_queries[parent_hash_idx].output_assigned.clone();
    }

    let root = ctx.assign_witnesses(input.root.iter().map(|x| F::from((*x) as u64)));
    for i in 0..32 {
        ctx.constrain_equal(&root[i], &curr[i]);
    }

    let circuit = KeccakCircuitBuilder::new(
        builder,
        keccak,
        range,
        None,
        |_: &mut RlcThreadBuilder<F>, _: RlpChip<F>, _: (FixedLenRLCs<F>, VarLenRLCs<F>)| {},
    );
    if !prover {
        let unusable_rows =
            var("UNUSABLE_ROWS").unwrap_or_else(|_| "109".to_string()).parse().unwrap();
        circuit.config(k as usize, Some(unusable_rows));
    }
    circuit
}


pub fn run_merkle(cli: Cli) {
    let name = cli.name;
    let k = cli.degree;
    set_var("KECCAK_ROWS", "50");

    let input_path = PathBuf::from("data")
        .join(cli.input_path.clone().unwrap_or_else(|| PathBuf::from(format!("{name}.json"))));
    let input: CircuitInput = serde_json::from_reader(
    File::open(&input_path)
        .unwrap_or_else(|e| panic!("Input file not found at {input_path:?}. {e:?}")),
    )
    .expect("Input file should be a valid JSON file");

    let circuit = test_keccak_merkle_circuit(k, RlcThreadBuilder::mock(), input.clone());
    MockProver::<Fr>::run(k, &circuit, vec![]).unwrap().assert_satisfied();
}
