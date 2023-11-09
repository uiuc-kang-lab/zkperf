use plonky2::field::extension::Extendable;
use plonky2::gadgets::arithmetic::EqualityGenerator;
use plonky2::gadgets::arithmetic_extension::QuotientGeneratorExtension;
use plonky2::gadgets::range_check::LowHighGenerator;
use plonky2::gadgets::split_base::BaseSumGenerator;
use plonky2::gadgets::split_join::{SplitGenerator, WireSplitGenerator};
use plonky2::gates::arithmetic_base::ArithmeticBaseGenerator;
use plonky2::gates::arithmetic_extension::ArithmeticExtensionGenerator;
use plonky2::gates::base_sum::BaseSplitGenerator;
use plonky2::gates::coset_interpolation::InterpolationGenerator;
use plonky2::gates::exponentiation::ExponentiationGenerator;
use plonky2::gates::lookup::LookupGenerator;
use plonky2::gates::lookup_table::LookupTableGenerator;
use plonky2::gates::multiplication_extension::MulExtensionGenerator;
use plonky2::gates::poseidon::PoseidonGenerator;
use plonky2::gates::poseidon_mds::PoseidonMdsGenerator;
use plonky2::gates::random_access::RandomAccessGenerator;
use plonky2::gates::reducing::ReducingGenerator;
use plonky2::gates::reducing_extension::ReducingGenerator as ReducingExtensionGenerator;
use plonky2::get_generator_tag_impl;
use plonky2::hash::hash_types::RichField;
use plonky2::impl_generator_serializer;
use plonky2::iop::generator::{
    ConstantGenerator, CopyGenerator, NonzeroTestGenerator, RandomValueGenerator,
};
use plonky2::iop::target::{BoolTarget, Target};
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::read_generator_impl;
use plonky2::util::serialization::WitnessGeneratorSerializer;
use serde::{Deserialize, Serialize};

use crate::u32::arithmetic_u32::U32Target;

use super::sha256::{CircuitBuilderHashSha2, WitnessHashSha2};
use super::{CircuitBuilderHash, Hash256Target, WitnessHash};

fn select_hash256<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    bit: BoolTarget,
    left: &Hash256Target,
    right: &Hash256Target,
) -> Hash256Target {
    let a = U32Target(builder.select(bit, left[0].0, right[0].0));
    let b = U32Target(builder.select(bit, left[1].0, right[1].0));
    let c = U32Target(builder.select(bit, left[2].0, right[2].0));
    let d = U32Target(builder.select(bit, left[3].0, right[3].0));
    let e = U32Target(builder.select(bit, left[4].0, right[4].0));
    let f = U32Target(builder.select(bit, left[5].0, right[5].0));
    let g = U32Target(builder.select(bit, left[6].0, right[6].0));
    let h = U32Target(builder.select(bit, left[7].0, right[7].0));

    [a, b, c, d, e, f, g, h]
}

pub fn compute_merkle_root<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    index_bits: &[BoolTarget],
    value: Hash256Target,
    siblings: &[Hash256Target],
) -> Hash256Target {
    let mut current = value;
    for (i, sibling) in siblings.iter().enumerate() {
        let bit = index_bits[i];

        let left = select_hash256(builder, bit, sibling, &current);
        let right = select_hash256(builder, bit, &current, sibling);
        current = builder.two_to_one_sha256(left, right);
    }
    current
}

pub struct MerkleProofSha256Gadget {
    pub root: Hash256Target,
    pub value: Hash256Target,
    pub siblings: Vec<Hash256Target>,
    pub index: Target,
}

impl MerkleProofSha256Gadget {
    pub fn add_virtual_to<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
        height: usize,
    ) -> Self {
        let siblings: Vec<Hash256Target> = (0..height)
            .map(|_| builder.add_virtual_hash256_target())
            .collect();

        let value = builder.add_virtual_hash256_target();
        let index = builder.add_virtual_target();
        let index_bits = builder.split_le(index, height);
        let root = compute_merkle_root(builder, &index_bits, value, &siblings);

        Self {
            root,
            value,
            siblings,
            index,
        }
    }

    pub fn set_witness<F: RichField, W: WitnessHashSha2<F>>(
        &self,
        witness: &mut W,
        index: u64,
        value: &[u8; 32],
        siblings: &[[u8; 32]],
    ) {
        witness.set_hash256_target(&self.value, value);
        witness.set_target(self.index, F::from_noncanonical_u64(index));
        for (i, sibling) in self.siblings.iter().enumerate() {
            witness.set_hash256_target(sibling, &siblings[i]);
        }
    }
}

pub struct DeltaMerkleProofSha256Gadget {
    pub old_root: Hash256Target,
    pub old_value: Hash256Target,

    pub new_root: Hash256Target,
    pub new_value: Hash256Target,

    pub siblings: Vec<Hash256Target>,
    pub index: Target,
}

impl DeltaMerkleProofSha256Gadget {
    pub fn add_virtual_to<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
        height: usize,
    ) -> Self {
        let siblings: Vec<Hash256Target> = (0..height)
            .map(|_| builder.add_virtual_hash256_target())
            .collect();

        let old_value = builder.add_virtual_hash256_target();
        let new_value = builder.add_virtual_hash256_target();
        let index = builder.add_virtual_target();
        let index_bits = builder.split_le(index, height);
        let old_root = compute_merkle_root(builder, &index_bits, old_value, &siblings);
        let new_root = compute_merkle_root(builder, &index_bits, new_value, &siblings);

        Self {
            old_root,
            old_value,
            new_root,
            new_value,
            siblings,
            index,
        }
    }

    pub fn set_witness<F: RichField, W: WitnessHashSha2<F>>(
        &self,
        witness: &mut W,
        index: u64,
        old_value: &[u8; 32],
        new_value: &[u8; 32],
        siblings: &[[u8; 32]],
    ) {
        witness.set_hash256_target(&self.old_value, old_value);
        witness.set_hash256_target(&self.new_value, new_value);
        witness.set_target(self.index, F::from_noncanonical_u64(index));
        for (i, sibling) in self.siblings.iter().enumerate() {
            witness.set_hash256_target(sibling, &siblings[i]);
        }
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

#[derive(Serialize, Deserialize)]
pub struct MerkleTargets {
    start_hash_target: [U32Target; 8],
    hash0: [U32Target; 8],
}

#[cfg(test)]
mod tests {

    use crate::hash::merkle_utils::MerkleProof256;
    use crate::hash::sha256_merkle::{
        MerkleGeneratorSerializer, MerkleProofSha256Gadget, MerkleTargets,
    };
    use crate::hash::{CircuitBuilderHash, WitnessHash};
    use log::Level;
    use plonky2::iop::witness::PartialWitness;
    use plonky2::plonk::circuit_builder::CircuitBuilder;
    use plonky2::plonk::circuit_data::{CircuitConfig, CircuitData};
    use plonky2::plonk::config::{
        GenericConfig, Hasher, KeccakGoldilocksConfig, PoseidonGoldilocksConfig,
    };
    use plonky2::plonk::prover::prove;
    use plonky2::util::serialization::DefaultGateSerializer;
    use plonky2::util::timing::TimingTree;
    use rand::Rng;
    use serde_json::json;

    use std::fs::File;
    use std::io::{Read, Write};
    use std::time::Instant;

    use crate::hash::simple_merkle_tree::MerkleTree;
    use jemallocator::Jemalloc;
    use plonky2::field::types::Field;
    use plonky2::hash::keccak::KeccakHash;
    use plonky2_field::goldilocks_field::GoldilocksField;

    #[global_allocator]
    static GLOBAL: Jemalloc = Jemalloc;

    #[test]
    fn test_verify_small_merkle_proof() {
        // build circuit once
        const D: usize = 2;
        type C = PoseidonGoldilocksConfig;
        type F = <C as GenericConfig<D>>::F;

        let config = CircuitConfig::standard_recursion_config();
        let mut builder = CircuitBuilder::<F, D>::new(config);

        let merkle_proof_gadget = MerkleProofSha256Gadget::add_virtual_to(&mut builder, 3);
        let expected_root_target = builder.add_virtual_hash256_target();
        builder.connect_hash256(expected_root_target, merkle_proof_gadget.root);
        let num_gates = builder.num_gates();
        // let copy_constraints = builder.copy_constraints.len();
        let data = builder.build::<C>();
        println!(
            "circuit num_gates={}, quotient_degree_factor={}",
            num_gates, data.common.quotient_degree_factor
        );

        let mut pw = PartialWitness::new();
        let proof_serialized = r#"
        {
          "root": "7e286a6721a66675ea033a4dcdec5abbdc7d3c81580e2d6ded7433ed113b7737",
          "siblings": [
            "0000000000000000000000000000000000000000000000000000000000000007",
            "ce44a8ee02db1a76906b0e9fd753893971c4db9a2341b0049d61f7fcd2a60adf",
            "81b1e323f0e91a785dfd155817e09949a7d66fe8fdc4f31f39530845e88ab63c"
          ],
          "index": 2,
          "value": "0000000000000000000000000000000000000000000000000000000000000003"
        }
        "#;
        let proof: MerkleProof256 =
            serde_json::from_str::<MerkleProof256>(proof_serialized).unwrap();
        merkle_proof_gadget.set_witness_from_proof(&mut pw, &proof);
        pw.set_hash256_target(&expected_root_target, &proof.root.0);

        let start_time = std::time::Instant::now();

        let proof = data.prove(pw).unwrap();
        let duration_ms = start_time.elapsed().as_millis();
        println!("proved in {}ms", duration_ms);
        assert!(data.verify(proof).is_ok());
    }

    #[test]
    fn build_merkle() {
        // const GOLDILOCKS_FIELD_ORDER: u64 = 18446744069414584321;
        const D: usize = 2;
        type C = KeccakGoldilocksConfig;
        type F = <C as GenericConfig<D>>::F;
        // const N: usize = 32;

        let config = CircuitConfig {
            num_routed_wires: 25,
            ..CircuitConfig::standard_recursion_zk_config()
        };

        // let mut rng = rand::thread_rng();
        // let mut leaves: Vec<GoldilocksField> = Vec::new();
        // for _ in 0..1024 {
        //     leaves.push(F::from_canonical_u64(
        //         rng.gen_range(0..GOLDILOCKS_FIELD_ORDER),
        //     ));
        // }
        // let tree: MerkleTree = MerkleTree::build(leaves.clone());

        let mut builder = CircuitBuilder::<F, D>::new(config);

        let start_hash_target = builder.add_virtual_hash256_target();
        // let start_hash_target = builder.add_virtual_hash_input_target();
        let hash0 = builder.add_virtual_hash256_target();

        let merkle_targets = MerkleTargets {
            start_hash_target,
            hash0,
        };

        let json_string = serde_json::to_string(&merkle_targets).unwrap();
        let mut file = File::create("merkle.targets").unwrap();
        let _ = file.write_all(json_string.as_bytes());

        // 1024 nodes, 10 deep
        // let res_leaf_2 = tree.clone().get_merkle_proof(10);

        // Input into circuit is hashed leaf (H_2) we're proving is part of the tree
        // Step 1: hash H_2 with H_3. H_3 is the first hash in the proof (res_leaf_2)
        // Input into circuit
        // let leaf_hashed = KeccakHash::<N>::hash_or_noop(&[leaves.clone()[10]]);

        builder.print_gate_counts(0);
        println!("building circuit");
        let start = Instant::now();
        let data = builder.build::<C>();
        let build_duration = start.elapsed();
        println!("circuit build duration: {:?}", build_duration);

        let gate_serializer = DefaultGateSerializer {};
        let generator_serializer = MerkleGeneratorSerializer {};

        let mut file = File::create("merkle.data").unwrap();
        let _ = file.write_all(
            &data
                .to_bytes(&gate_serializer, &generator_serializer)
                .unwrap(),
        );
    }

    #[test]
    fn test_merkle() {
        env_logger::init();
        const GOLDILOCKS_FIELD_ORDER: u64 = 18446744069414584321;
        const D: usize = 2;
        type C = KeccakGoldilocksConfig;
        type F = <C as GenericConfig<D>>::F;
        const N: usize = 32;

        let mut rng = rand::thread_rng();
        let mut leaves: Vec<GoldilocksField> = Vec::new();
        for _ in 0..1024 {
            leaves.push(F::from_canonical_u64(
                rng.gen_range(0..GOLDILOCKS_FIELD_ORDER),
            ));
        }
        let tree: MerkleTree = MerkleTree::build(leaves.clone());

        // let mut builder = CircuitBuilder::<F, D>::new(config);
        // let start_hash_target = builder.add_virtual_hash256_target();
        // let hash0 = builder.add_virtual_hash256_target();

        // 1024 nodes, 10 deep
        let res_leaf_2 = tree.clone().get_merkle_proof(10);

        // Input into circuit is hashed leaf (H_2) we're proving is part of the tree
        // Step 1: hash H_2 with H_3. H_3 is the first hash in the proof (res_leaf_2)
        // Input into circuit
        let leaf_hashed = KeccakHash::<N>::hash_or_noop(&[leaves.clone()[10]]);

        let mut file = File::open("merkle.targets").expect("File not found");

        let mut buffer = String::new();
        file.read_to_string(&mut buffer)
            .expect("Failed to read the file");

        let merkle_targets: MerkleTargets = serde_json::from_str(&buffer).unwrap();

        let mut pw: PartialWitness<F> = PartialWitness::new();
        pw.set_hash256_target(&merkle_targets.start_hash_target, &leaf_hashed.0);
        // pw.set_keccak256_input_target(&start_hash_target, &leaf_hashed.0);
        pw.set_hash256_target(&merkle_targets.hash0, &(res_leaf_2[0].0));

        // let data = builder.build::<C>();
        let mut file = File::open("merkle.data").expect("File not found");

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .expect("Failed to read the file");

        let gate_serializer = DefaultGateSerializer {};
        let generator_serializer = MerkleGeneratorSerializer {};
        let data: CircuitData<F, C, D> =
            CircuitData::from_bytes(&buffer, &gate_serializer, &generator_serializer).unwrap();

        println!("proving circuit");
        let mut timing = TimingTree::new("prove", Level::Info);
        let proof = prove::<F, C, D>(&data.prover_only, &data.common, pw, &mut timing).unwrap();
        timing.pop();
        timing.print();
        let proof_duration = timing.duration();
        println!("Proving time: {:?}", proof_duration);

        let proof_bytes = proof.to_bytes();
        let proof_len = proof_bytes.len();
        println!("Proof size: {} bytes", proof_len);

        println!("verifying circuit");
        let mut timing = TimingTree::new("verify", Level::Info);
        data.verify(proof).expect("verify error");
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

        let mut file = File::create("../merkle.json").unwrap();
        let _ = file.write_all(json_string.as_bytes());
    }
}
