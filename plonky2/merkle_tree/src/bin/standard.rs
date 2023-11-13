use log::Level;
// use plonky2::read_gate_impl;
// use plonky2::{get_gate_tag_impl, util::serialization::GateSerializer};
// use plonky2_crypto::u32::gates::arithmetic_u32::{U32ArithmeticGate, U32ArithmeticGenerator};
// use plonky2_crypto::u32::gates::interleave_u32::{U32InterleaveGate, U32InterleaveGenerator};
// use plonky2_crypto::u32::gates::subtraction_u32::{U32SubtractionGate, U32SubtractionGenerator};
// use plonky2_crypto::u32::gates::uninterleave_to_u32::{UninterleaveToU32Gate, UninterleaveToU32Generator};
use plonky2_field::types::Field;
use serde_json::json;
use std::{fs::File, io::Write, time::Instant};

use jemallocator::Jemalloc;
use num::BigUint;
use plonky2::{
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData},
        config::{GenericConfig, KeccakGoldilocksConfig},
        prover::prove,
    },
    util::timing::TimingTree,
};

use plonky2_crypto::{
    biguint::{BigUintTarget, CircuitBuilderBiguint},
    hash::{
        keccak256::{CircuitBuilderHashKeccak, WitnessHashKeccak, KECCAK256_R},
        CircuitBuilderHash, HashInputTarget, HashOutputTarget,
    },
    simple_merkle_tree::MerkleTree,
    u32::arithmetic_u32::CircuitBuilderU32,
};
use plonky2_field::goldilocks_field::GoldilocksField;
use rand::Rng;

// use plonky2::field::extension::Extendable;
// use plonky2::gadgets::arithmetic::EqualityGenerator;
// use plonky2::gadgets::arithmetic_extension::QuotientGeneratorExtension;
// use plonky2::gadgets::range_check::LowHighGenerator;
// use plonky2::gadgets::split_base::BaseSumGenerator;
// use plonky2::gadgets::split_join::{SplitGenerator, WireSplitGenerator};
// use plonky2::gates::arithmetic_base::ArithmeticBaseGenerator;
// use plonky2::gates::arithmetic_base::ArithmeticGate;
// use plonky2::gates::arithmetic_extension::ArithmeticExtensionGenerator;
// use plonky2::gates::base_sum::BaseSplitGenerator;
// use plonky2::gates::base_sum::BaseSumGate;
// use plonky2::gates::constant::ConstantGate;
// use plonky2::gates::coset_interpolation::CosetInterpolationGate;
// use plonky2::gates::coset_interpolation::InterpolationGenerator;
// use plonky2::gates::exponentiation::ExponentiationGenerator;
// use plonky2::gates::lookup::LookupGate;
// use plonky2::gates::lookup::LookupGenerator;
// use plonky2::gates::lookup_table::LookupTableGate;
// use plonky2::gates::lookup_table::LookupTableGenerator;
// use plonky2::gates::multiplication_extension::MulExtensionGate;
// use plonky2::gates::multiplication_extension::MulExtensionGenerator;
// use plonky2::gates::noop::NoopGate;
// use plonky2::gates::poseidon::PoseidonGate;
// use plonky2::gates::poseidon::PoseidonGenerator;
// use plonky2::gates::poseidon_mds::PoseidonMdsGate;
// use plonky2::gates::poseidon_mds::PoseidonMdsGenerator;
// use plonky2::gates::public_input::PublicInputGate;
// use plonky2::gates::random_access::RandomAccessGate;
// use plonky2::gates::random_access::RandomAccessGenerator;
// use plonky2::gates::reducing::ReducingGate;
// use plonky2::gates::reducing::ReducingGenerator;
// use plonky2::gates::reducing_extension::ReducingGenerator as ReducingExtensionGenerator;
// use plonky2::get_generator_tag_impl;
// use plonky2::hash::hash_types::RichField;
// use plonky2::impl_generator_serializer;
// use plonky2::iop::generator::{
//     ConstantGenerator, CopyGenerator, NonzeroTestGenerator, RandomValueGenerator,
// };

// use plonky2::read_generator_impl;
// use plonky2::util::serialization::WitnessGeneratorSerializer;
// use serde::{Deserialize, Serialize};

// pub struct MerkleGateSerializer;
// impl<F: RichField + Extendable<D>, const D: usize> GateSerializer<F, D> for MerkleGateSerializer {
//     impl_gate_serializer! {
//         DefaultGateSerializer,
//         ArithmeticGate,
//         BaseSumGate<2>,
//         BaseSumGate<4>,
//         ConstantGate,
//         CosetInterpolationGate<F, D>,
//         LookupGate,
//         LookupTableGate,
//         MulExtensionGate<D>,
//         NoopGate,
//         PoseidonMdsGate<F, D>,
//         PoseidonGate<F, D>,
//         PublicInputGate,
//         RandomAccessGate<F, D>,
//         ReducingGate<D>,
//         U32ArithmeticGate<F, D>,
//         U32InterleaveGate,
//         U32SubtractionGate<F, D>,
//         UninterleaveToU32Gate
//     }
// }
// pub struct MerkleGeneratorSerializer {}

// impl<F, const D: usize> WitnessGeneratorSerializer<F, D> for MerkleGeneratorSerializer
// where
//     F: RichField + Extendable<D>,
// {
//     impl_generator_serializer! {
//         DefaultGeneratorSerializer,
//         ArithmeticBaseGenerator<F, D>,
//         ArithmeticExtensionGenerator<F, D>,
//         BaseSplitGenerator<2>,
//         BaseSumGenerator<2>,
//         ConstantGenerator<F>,
//         CopyGenerator,
//         EqualityGenerator,
//         ExponentiationGenerator<F, D>,
//         InterpolationGenerator<F, D>,
//         LookupGenerator,
//         LookupTableGenerator,
//         LowHighGenerator,
//         MulExtensionGenerator<F, D>,
//         NonzeroTestGenerator,
//         PoseidonGenerator<F, D>,
//         PoseidonMdsGenerator<D>,
//         QuotientGeneratorExtension<D>,
//         RandomAccessGenerator<F, D>,
//         RandomValueGenerator,
//         ReducingGenerator<D>,
//         ReducingExtensionGenerator<D>,
//         SplitGenerator,
//         U32ArithmeticGenerator<F, D>,
//         U32InterleaveGenerator,
//         U32SubtractionGenerator<F, D>,
//         UninterleaveToU32Generator,
//         WireSplitGenerator
//     }
// }

// Returns the cricuit data for verifying the Merkle Proof + the target for witness (non-public) input data
// the second part might not be necessary, but don't know how to set that data otherwise in the testing part
// #[derive(Serialize, Deserialize)]
// pub struct MerkleTargets {
//     targets: Vec<HashOutputTarget>,
// }

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn connect_two(
    builder: &mut CircuitBuilder<GoldilocksField, 2>,
    hash_target: &HashInputTarget,
    inp1: &HashOutputTarget,
    inp2: &HashOutputTarget,
    one: &BigUintTarget,
    last: &BigUintTarget,
    zero: &BigUintTarget,
) {
    builder.connect_hash_input(&hash_target, &inp1, 0);
    builder.connect_hash_input(&hash_target, &inp2, 8);

    // make compatible with sha3::Keccak256
    builder.connect_hash_input(&hash_target, &one, 16);
    builder.connect_hash_input(&hash_target, &last, 33);
    builder.connect_hash_input(&hash_target, &zero, 17);
}

fn verify_merkle_proof_circuit(
    leaf_index: usize,
    nr_layers: usize,
    cols: usize,
) -> (
    CircuitData<GoldilocksField, KeccakGoldilocksConfig, 2>,
    Vec<HashOutputTarget>,
) {
    const D: usize = 2;
    type C = KeccakGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;

    let mut targets: Vec<BigUintTarget> = Vec::new();

    let config: CircuitConfig = CircuitConfig {
        num_routed_wires: cols,
        ..CircuitConfig::standard_recursion_zk_config()
    };

    let mut builder: CircuitBuilder<plonky2::field::goldilocks_field::GoldilocksField, 2> =
        CircuitBuilder::<F, D>::new(config);

    // The leaf to prove is in the Merkle Tree
    let leaf_to_prove = builder.add_virtual_biguint_target(8);

    // The first hashing outside of the loop, since it uses the leaf_to_prove
    let merkle_proof_elm = builder.add_virtual_biguint_target(8);

    let mut next_hash_inp = builder.add_virtual_hash_input_target(1, KECCAK256_R);

    let one = builder.constant_biguint(&BigUint::from(1u32));
    let last = builder.constant_biguint(&BigUint::from(2147483648u32));
    let zero = builder.hash_zero(16);

    if leaf_index % 2 == 0 {
        connect_two(
            &mut builder,
            &next_hash_inp,
            &leaf_to_prove,
            &merkle_proof_elm,
            &one,
            &last,
            &zero,
        );
    } else {
        connect_two(
            &mut builder,
            &next_hash_inp,
            &merkle_proof_elm,
            &leaf_to_prove,
            &one,
            &last,
            &zero,
        );
    }
    targets.push(leaf_to_prove);
    targets.push(merkle_proof_elm);

    let mut next_hash = builder.hash_keccak256(&next_hash_inp);

    let mut current_layer_index = leaf_index / 2;

    for _layer in 1..nr_layers {
        let merkle_proof_elm = builder.add_virtual_biguint_target(8);

        next_hash_inp = builder.add_virtual_hash_input_target(1, KECCAK256_R);
        if current_layer_index % 2 == 0 {
            connect_two(
                &mut builder,
                &next_hash_inp,
                &next_hash,
                &merkle_proof_elm,
                &one,
                &last,
                &zero,
            );
        } else {
            connect_two(
                &mut builder,
                &next_hash_inp,
                &merkle_proof_elm,
                &next_hash,
                &one,
                &last,
                &zero,
            );
        }
        targets.push(merkle_proof_elm);
        next_hash = builder.hash_keccak256(&next_hash_inp);
        current_layer_index = current_layer_index / 2;
    }
    // This is the expected root value
    for i in 0..next_hash.num_limbs() {
        builder.register_u32_public_input(next_hash.get_limb(i));
    }
    targets.push(next_hash);

    // let json_string = serde_json::to_string(&merkle_targets).unwrap();
    // let mut file = File::create("merkle.targets").unwrap();
    // let _ = file.write_all(json_string.as_bytes());

    println!("building circuit");
    builder.print_gate_counts(0);
    let start = Instant::now();
    let data = builder.build::<C>();
    let build_duration = start.elapsed();
    println!("circuit build duration: {:?}", build_duration);

    // let gate_serializer = MerkleGateSerializer {};
    // let generator_serializer = MerkleGeneratorSerializer {};
    // let mut file = File::create("merkle.data").unwrap();
    // let _ = file.write_all(
    //     &data
    //         .to_bytes(&gate_serializer, &generator_serializer)
    //         .unwrap(),
    // );
    (data, targets)
}

fn get_tree(nr_leaves: u64) -> MerkleTree {
    const GOLDILOCKS_FIELD_ORDER: u64 = 18446744069414584321;
    const D: usize = 2;
    type C = KeccakGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;

    let mut rng = rand::thread_rng();
    let mut leaves: Vec<GoldilocksField> = Vec::new();
    for _ in 0..nr_leaves {
        leaves.push(F::from_canonical_u64(
            rng.gen_range(0..GOLDILOCKS_FIELD_ORDER),
        ));
    }
    let tree: MerkleTree = MerkleTree::build(leaves.clone());
    tree
}

fn main() {
    env_logger::init();
    let x = std::env::args().nth(1).expect("cols");
    let cols = x.parse::<usize>().unwrap();

    if cols < 25 {
        panic!("Invalid cols")
    }

    const D: usize = 2;
    type C = KeccakGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;

    let tree: MerkleTree = get_tree(1024);

    let merkle_proof_leaf0 = tree.clone().get_merkle_proof(0);
    println!("{:?}", merkle_proof_leaf0);

    let (circuit_data, targets) = verify_merkle_proof_circuit(0, 10, cols);

    let mut pw = plonky2::iop::witness::PartialWitness::new();
    // non-public inputs to witness: leaf and elements of merkle proof
    pw.set_keccak256_output_target(&targets[0], &tree.tree[0][0]);
    for i in 1..11 {
        pw.set_keccak256_output_target(&targets[i], &merkle_proof_leaf0[i - 1]);
    }
    // public input: root of merkle tree
    let expected_public_inputs = &targets[targets.len() - 1];

    pw.set_keccak256_output_target(expected_public_inputs, &tree.root);

    println!("proving circuit");
    let mut timing = TimingTree::new("prove", Level::Info);
    let proof = prove::<F, C, D>(
        &circuit_data.prover_only,
        &circuit_data.common,
        pw,
        &mut timing,
    )
    .unwrap();
    timing.pop();
    timing.print();
    let proof_duration = timing.duration();
    println!("Proving time: {:?}", proof_duration);

    let proof_bytes = proof.to_bytes();
    let proof_len = proof_bytes.len();
    println!("Proof size: {} bytes", proof_len);

    println!("verifying circuit");
    let mut timing = TimingTree::new("verify", Level::Info);
    circuit_data.verify(proof).expect("verify error");
    timing.pop();
    timing.print();

    let verify_duration = timing.duration();
    println!("Verifying time: {:?}", verify_duration);

    println!("writing results");
    let results = json!({
      "Framework": "plonky2",
      "Circuit": "MerkleTree",
      "Backend": "Plonk+FRI",
      "Curve": "NaN",
      "ProverTime": proof_duration.as_secs_f32(),
      "VerifierTime": verify_duration.as_nanos() as f32 / 1000000.,
      "ProofSize": proof_len
    });

    let json_string = serde_json::to_string(&results).unwrap();

    let mut file = File::create("merkle.json").unwrap();
    let _ = file.write_all(json_string.as_bytes());
}
