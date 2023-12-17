use clap::{App, Arg};
use log::Level;
use plonky2::impl_gate_serializer;
use plonky2::plonk::prover::prove;
use plonky2::read_gate_impl;
use plonky2::{
    field::{goldilocks_field::GoldilocksField, types::Field},
    hash::{
        hash_types::{HashOut, HashOutTarget, RichField},
        poseidon::PoseidonHash,
    },
    iop::witness::{PartialWitness, WitnessWrite},
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData, CommonCircuitData, VerifierOnlyCircuitData},
        config::{GenericConfig, KeccakGoldilocksConfig},
        proof,
    },
    util::timing::TimingTree,
};
use plonky2::{get_gate_tag_impl, util::serialization::GateSerializer};
use plonky2_merkle_trees::simple_merkle_tree::simple_merkle_tree::MerkleTree;
use serde_json::json;
use std::io::Read;
use std::{fs::File, io::Write, time::Instant};

use jemallocator::Jemalloc;

use rand::Rng;

use plonky2::field::extension::Extendable;
use plonky2::gadgets::arithmetic::EqualityGenerator;
use plonky2::gadgets::arithmetic_extension::QuotientGeneratorExtension;
use plonky2::gadgets::range_check::LowHighGenerator;
use plonky2::gadgets::split_base::BaseSumGenerator;
use plonky2::gadgets::split_join::{SplitGenerator, WireSplitGenerator};
use plonky2::gates::arithmetic_base::ArithmeticBaseGenerator;
use plonky2::gates::arithmetic_base::ArithmeticGate;
use plonky2::gates::arithmetic_extension::ArithmeticExtensionGenerator;
use plonky2::gates::base_sum::BaseSplitGenerator;
use plonky2::gates::base_sum::BaseSumGate;
use plonky2::gates::constant::ConstantGate;
use plonky2::gates::coset_interpolation::CosetInterpolationGate;
use plonky2::gates::coset_interpolation::InterpolationGenerator;
use plonky2::gates::exponentiation::ExponentiationGenerator;
use plonky2::gates::lookup::LookupGate;
use plonky2::gates::lookup::LookupGenerator;
use plonky2::gates::lookup_table::LookupTableGate;
use plonky2::gates::lookup_table::LookupTableGenerator;
use plonky2::gates::multiplication_extension::MulExtensionGate;
use plonky2::gates::multiplication_extension::MulExtensionGenerator;
use plonky2::gates::noop::NoopGate;
use plonky2::gates::poseidon::PoseidonGate;
use plonky2::gates::poseidon::PoseidonGenerator;
use plonky2::gates::poseidon_mds::PoseidonMdsGate;
use plonky2::gates::poseidon_mds::PoseidonMdsGenerator;
use plonky2::gates::public_input::PublicInputGate;
use plonky2::gates::random_access::RandomAccessGate;
use plonky2::gates::random_access::RandomAccessGenerator;
use plonky2::gates::reducing::ReducingGate;
use plonky2::gates::reducing::ReducingGenerator;
use plonky2::gates::reducing_extension::ReducingGenerator as ReducingExtensionGenerator;
use plonky2::get_generator_tag_impl;
use plonky2::impl_generator_serializer;
use plonky2::iop::generator::{
    ConstantGenerator, CopyGenerator, NonzeroTestGenerator, RandomValueGenerator,
};

use plonky2::read_generator_impl;
use plonky2::util::serialization::WitnessGeneratorSerializer;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

pub struct MerkleGateSerializer;
impl<F: RichField + Extendable<D>, const D: usize> GateSerializer<F, D> for MerkleGateSerializer {
    impl_gate_serializer! {
        DefaultGateSerializer,
        ArithmeticGate,
        BaseSumGate<2>,
        BaseSumGate<4>,
        ConstantGate,
        CosetInterpolationGate<F, D>,
        LookupGate,
        LookupTableGate,
        MulExtensionGate<D>,
        NoopGate,
        PoseidonMdsGate<F, D>,
        PoseidonGate<F, D>,
        PublicInputGate,
        RandomAccessGate<F, D>,
        ReducingGate<D>
    }
}
pub struct MerkleGeneratorSerializer {}

impl<F, const D: usize> WitnessGeneratorSerializer<F, D> for MerkleGeneratorSerializer
where
    F: RichField + Extendable<D>,
{
    impl_generator_serializer! {
        DefaultGeneratorSerializer,
        ArithmeticBaseGenerator<F, D>,
        ArithmeticExtensionGenerator<F, D>,
        BaseSplitGenerator<2>,
        BaseSumGenerator<2>,
        ConstantGenerator<F>,
        CopyGenerator,
        EqualityGenerator,
        ExponentiationGenerator<F, D>,
        InterpolationGenerator<F, D>,
        LookupGenerator,
        LookupTableGenerator,
        LowHighGenerator,
        MulExtensionGenerator<F, D>,
        NonzeroTestGenerator,
        PoseidonGenerator<F, D>,
        PoseidonMdsGenerator<D>,
        QuotientGeneratorExtension<D>,
        RandomAccessGenerator<F, D>,
        RandomValueGenerator,
        ReducingGenerator<D>,
        ReducingExtensionGenerator<D>,
        SplitGenerator,
        WireSplitGenerator
    }
}

/**
 * zkp for veryfing merkle proof
 */

// Returns the cricuit data for verifying the Merkle Proof + the target for witness (non-public) input data
// the second part might not be necessary, but don't know how to set that data otherwise in the testing part
pub fn verify_merkle_proof_circuit(
    leaf_index: usize,
    nr_layers: usize,
) -> (
    CircuitData<GoldilocksField, KeccakGoldilocksConfig, 2>,
    Vec<HashOutTarget>,
) {
    const D: usize = 2;
    type C = KeccakGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;

    let mut targets: Vec<HashOutTarget> = Vec::new();

    let config = CircuitConfig::standard_recursion_config();
    let mut builder: CircuitBuilder<plonky2::field::goldilocks_field::GoldilocksField, 2> =
        CircuitBuilder::<F, D>::new(config);

    // The leaf to prove is in the Merkle Tree
    let leaf_to_prove = builder.add_virtual_hash();
    targets.push(leaf_to_prove);

    // The first hashing outside of the loop, since it uses the leaf_to_prove
    let merkle_proof_elm = builder.add_virtual_hash();
    targets.push(merkle_proof_elm);

    let mut next_hash: plonky2::hash::hash_types::HashOutTarget;
    if leaf_index % 2 == 0 {
        next_hash = builder.hash_or_noop::<PoseidonHash>(
            [
                leaf_to_prove.elements.to_vec(),
                merkle_proof_elm.elements.to_vec(),
            ]
            .concat(),
        );
    } else {
        next_hash = builder.hash_or_noop::<PoseidonHash>(
            [
                merkle_proof_elm.elements.to_vec(),
                leaf_to_prove.elements.to_vec(),
            ]
            .concat(),
        );
    }

    let mut current_layer_index = leaf_index / 2;

    for _layer in 1..nr_layers {
        let merkle_proof_elm = builder.add_virtual_hash();
        targets.push(merkle_proof_elm);

        if current_layer_index % 2 == 0 {
            next_hash = builder.hash_or_noop::<PoseidonHash>(
                [
                    next_hash.elements.to_vec(),
                    merkle_proof_elm.elements.to_vec(),
                ]
                .concat(),
            );
        } else {
            next_hash = builder.hash_or_noop::<PoseidonHash>(
                [
                    merkle_proof_elm.elements.to_vec(),
                    next_hash.elements.to_vec(),
                ]
                .concat(),
            );
        }
        current_layer_index = current_layer_index / 2;
    }
    // This is the expected root value
    builder.register_public_inputs(&next_hash.elements);

    let json_string = serde_json::to_string(&targets).unwrap();
    let mut file = File::create("merkle_targets").unwrap();
    file.write_all(json_string.as_bytes()).unwrap();

    println!("building circuit");
    builder.print_gate_counts(0);
    let start = Instant::now();
    let data = builder.build::<C>();
    let build_duration = start.elapsed();
    println!("circuit build duration: {:?}", build_duration);

    let gate_serializer = MerkleGateSerializer {};
    let generator_serializer = MerkleGeneratorSerializer {};
    let mut file = File::create("merkle_data").unwrap();
    let _ = file.write_all(
        &data
            .to_bytes(&gate_serializer, &generator_serializer)
            .unwrap(),
    );

    (data, targets)
}

fn get_tree(nr_leaves: u64) -> MerkleTree {
    const GOLDILOCKS_FIELD_ORDER: u64 = 18446744069414584321;
    const D: usize = 2;
    type C = KeccakGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;
    let mut rng = rand::thread_rng();
    let mut leaves: Vec<GoldilocksField> = Vec::new();
    for i in 0..nr_leaves {
        leaves.push(F::from_canonical_u64(
            rng.gen_range(0..GOLDILOCKS_FIELD_ORDER),
        ));
    }
    let tree: MerkleTree = MerkleTree::build(leaves.clone());
    tree
}

fn main() {
    env_logger::init();
    let matches = App::new("standard")
        .arg(
            Arg::with_name("build_prove")
                .index(1)
                .value_name("build_prove")
                .help("build or prove")
                .required(true),
        )
        .arg(
            Arg::with_name("output")
                .index(2)
                .value_name("outp_json")
                .help("output json file path")
                .required(true),
        )
        .arg(
            Arg::with_name("cols")
                .short("c")
                .long("cols")
                .value_name("cols")
                .help("number of columns")
                .takes_value(true),
        )
        .get_matches();

    let build_prove = matches.value_of("build_prove").unwrap().to_string();
    let outp_json = matches.value_of("output").unwrap().to_string();

    if build_prove != "build" && build_prove != "prove" {
        panic!("Must specify build or prove");
    }

    let cols = if let Some(col) = matches.value_of("cols") {
        col.parse::<usize>().unwrap()
    } else {
        25 as usize
    };

    if cols < 25 {
        panic!("Invalid cols")
    }

    const D: usize = 2;
    type C = KeccakGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;

    if build_prove == "build" {
        let tree: MerkleTree = get_tree(1024);
        let merkle_proof_leaf0 = tree.clone().get_merkle_proof(0);
        println!("{:?}", merkle_proof_leaf0);

        let (circuit_data, targets) = verify_merkle_proof_circuit(0, 10);
    } else {
        let tree: MerkleTree = get_tree(1024);

        let merkle_proof_leaf0 = tree.clone().get_merkle_proof(0);
        println!("{:?}", merkle_proof_leaf0);
        let mut target_file = File::open("merkle_targets").unwrap();
        let mut target_string = String::new();

        target_file.read_to_string(&mut target_string).unwrap();

        let targets: Vec<HashOutTarget> = serde_json::from_str(&target_string).unwrap();

        let mut file = File::open("merkle_data").expect("File not found");

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .expect("Failed to read the file");

        let gate_serializer = MerkleGateSerializer {};
        let generator_serializer = MerkleGeneratorSerializer {};
        let circuit_data: CircuitData<F, C, D> =
            CircuitData::from_bytes(&buffer, &gate_serializer, &generator_serializer).unwrap();

        let mut pw = plonky2::iop::witness::PartialWitness::new();
        // non-public inputs to witness: leaf and elements of merkle proof
        pw.set_hash_target(targets[0], tree.tree[0][0]); // leaf index 0

        for i in 1..11 {
          pw.set_hash_target(targets[i], merkle_proof_leaf0[i-1]);
        }

        // public input: root of merkle tree
        let expected_public_inputs = circuit_data.prover_only.public_inputs.clone();
        for i in 0..4 {
            pw.set_target(expected_public_inputs[i], tree.root.elements[i]);
        }

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

        let mut file = File::create(outp_json).unwrap();
        let _ = file.write_all(json_string.as_bytes());
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use plonky2::{
        gates::poseidon::PoseidonGenerator,
        hash::{hash_types::RichField, poseidon::PoseidonHash},
        iop::witness::WitnessWrite,
        plonk::config::{GenericConfig, Hasher, PoseidonGoldilocksConfig},
    };
    use plonky2_field::{
        goldilocks_field::GoldilocksField,
        types::{Field, Sample},
    };
    use plonky2_merkle_trees::simple_merkle_tree::simple_merkle_tree::MerkleTree;
    use rand::Rng;

    use crate::verify_merkle_proof_circuit;
    const GOLDILOCKS_FIELD_ORDER: u64 = 18446744069414584321;
    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;

    fn get_test_tree(nr_leaves: u64) -> MerkleTree {
        let mut rng = rand::thread_rng();
        let mut leaves: Vec<GoldilocksField> = Vec::new();
        for i in 0..nr_leaves {
            leaves.push(F::from_canonical_u64(
                rng.gen_range(0..GOLDILOCKS_FIELD_ORDER),
            ));
        }
        let tree: MerkleTree = MerkleTree::build(leaves.clone());
        tree
    }

    #[test]
    fn test_tree_4_leaves_index0() -> Result<()> {
        // Test tree, 4 leaves
        let tree: MerkleTree = get_test_tree(4);

        let merkle_proof_leaf0 = tree.clone().get_merkle_proof(0);
        // [
        // other leaf: HashOut { elements: [156728478, 0, 0, 0] },
        // other node: HashOut { elements: [6698018865469624861, 12486244005715193285, 11330639022572315007, 6059804404595156248] }
        // ]

        // The tree:
        //         R
        //    N0         N1
        // L0   L1    L2    L3

        // PROOF for L0: (the elements in brackets)
        //    N0          [N1]
        // L0   [L1]    L2    L3

        // Check proof L0:
        // H0 = Hash(L0||L1)
        // H1 = Hash(H0||N1)
        // CHECK Root = H1 equals R ?

        let (circuit_data, targets) = verify_merkle_proof_circuit(0, 2);

        /* The witness needs the following inputs:
          - leaf_to_prove
          - proof elm 0
          - poof elm 1
          Public:
          - expected_root
        */

        let mut pw = plonky2::iop::witness::PartialWitness::new();
        // non-public inputs to witness: leaf and elements of merkle proof
        pw.set_hash_target(targets[0], tree.tree[0][0]); // leaf index 0
        pw.set_hash_target(targets[1], merkle_proof_leaf0[0]);
        pw.set_hash_target(targets[2], merkle_proof_leaf0[1]);

        // Public input: root
        let expected_public_inputs = circuit_data.prover_only.public_inputs.clone();
        for i in 0..4 {
            pw.set_target(expected_public_inputs[i], tree.root.elements[i]);
        }

        let proof = circuit_data.prove(pw)?;
        // uncomment to print proof
        // println!("{:?}", proof);

        // Verify proof
        circuit_data.verify(proof)
    }

    #[test]
    fn test_tree_4_leaves_index3() -> Result<()> {
        let tree: MerkleTree = get_test_tree(4);
        let merkle_proof_leaf0 = tree.clone().get_merkle_proof(3);
        // println!("{:?}", merkle_proof_leaf0);

        // [HashOut { elements: [2876514289, 0, 0, 0] },
        // HashOut { elements: [6678006133445961348, 15827935749738443865, 6295652393730592048, 1546515167911236130] }]

        // The tree:
        //         R
        //    N0         N1
        // L0   L1    L2    L3

        // PROOF for L0: (the elements in brackets)
        //    [N0]         N1
        // L0   L1    [L2]    L3

        // Check proof L3:
        // H0 = Hash(L2||L3)
        // H1 = Hash(N0||H0)
        // CHECK Root = H1 equals R ?

        let (circuit_data, targets) = verify_merkle_proof_circuit(3, 2);

        /* The witness needs the following inputs:
          - leaf_to_prove
          - proof elm 0
          - poof elm 1
          Public:
          - expected_root
        */

        let mut pw = plonky2::iop::witness::PartialWitness::new();
        // non-public inputs to witness: leaf and elements of merkle proof
        pw.set_hash_target(targets[0], tree.tree[0][3]); // leaf index 3

        pw.set_hash_target(targets[1], merkle_proof_leaf0[0]);
        pw.set_hash_target(targets[2], merkle_proof_leaf0[1]);

        // Public input: root
        let expected_public_inputs = circuit_data.prover_only.public_inputs.clone();
        for i in 0..4 {
            pw.set_target(expected_public_inputs[i], tree.root.elements[i]);
        }

        let proof = circuit_data.prove(pw)?;
        // uncomment to print proof
        // println!("{:?}", proof);

        // Verify proof
        circuit_data.verify(proof)
    }

    #[test]
    fn test_tree_16_leaves_index_0() -> Result<()> {
        // Test tree, 16 leaves
        let tree: MerkleTree = get_test_tree(16);
        let merkle_proof_leaf0 = tree.clone().get_merkle_proof(0);
        println!("{:?}", merkle_proof_leaf0);

        let (circuit_data, targets) = verify_merkle_proof_circuit(0, 4);

        /* The witness needs the following inputs:
          - leaf_to_prove
          - merkle proof elm 0,1,2,3
          Public:
          - expected_root
        */

        let mut pw = plonky2::iop::witness::PartialWitness::new();
        // non-public inputs to witness: leaf and elements of merkle proof
        pw.set_hash_target(targets[0], tree.tree[0][0]); // leaf index 0

        pw.set_hash_target(targets[1], merkle_proof_leaf0[0]);
        pw.set_hash_target(targets[2], merkle_proof_leaf0[1]);
        pw.set_hash_target(targets[3], merkle_proof_leaf0[2]);
        pw.set_hash_target(targets[4], merkle_proof_leaf0[3]);

        // public input: root of merkle tree
        let expected_public_inputs = circuit_data.prover_only.public_inputs.clone();
        for i in 0..4 {
            pw.set_target(expected_public_inputs[i], tree.root.elements[i]);
        }

        let proof = circuit_data.prove(pw)?;
        // uncomment to print proof
        // println!("{:?}", proof);

        // Verify proof
        circuit_data.verify(proof)
    }

    #[test]
    fn test_tree_16_leaves_index_7() -> Result<()> {
        // Test tree, 16 leaves
        let tree: MerkleTree = get_test_tree(16);
        let merkle_proof_leaf7 = tree.clone().get_merkle_proof(7);
        println!("{:?}", merkle_proof_leaf7);

        let (circuit_data, targets) = verify_merkle_proof_circuit(7, 4);

        /* The witness needs the following inputs:
          - leaf_to_prove
          - merkle proof elm 0,1,2,3
          Public:
          - expected_root
        */

        let mut pw = plonky2::iop::witness::PartialWitness::new();
        // non-piblic input: leaf hash and merkle proof elements
        pw.set_hash_target(targets[0], tree.tree[0][7]);
        pw.set_hash_target(targets[1], merkle_proof_leaf7[0]);
        pw.set_hash_target(targets[2], merkle_proof_leaf7[1]);
        pw.set_hash_target(targets[3], merkle_proof_leaf7[2]);
        pw.set_hash_target(targets[4], merkle_proof_leaf7[3]);

        // Public input: root
        let expected_public_inputs = circuit_data.prover_only.public_inputs.clone();
        for i in 0..4 {
            pw.set_target(expected_public_inputs[i], tree.root.elements[i]);
        }

        let proof = circuit_data.prove(pw)?;
        // uncomment to print proof
        // println!("{:?}", proof);

        // Verify proof
        circuit_data.verify(proof)
    }
}
