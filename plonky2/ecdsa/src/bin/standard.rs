use plonky2::field::secp256k1_base::Secp256K1Base;
use plonky2::get_gate_tag_impl;
use plonky2::read_gate_impl;
use plonky2_ecdsa::gadgets::glv::GLVDecompositionGenerator;
use plonky2_ecdsa::gadgets::nonnative::NonNativeInverseGenerator;
use plonky2_ecdsa::gadgets::nonnative::NonNativeSubtractionGenerator;
use plonky2_u32::gates::add_many_u32::U32AddManyGate;
use plonky2_u32::gates::add_many_u32::U32AddManyGenerator;
use plonky2_u32::gates::arithmetic_u32::U32ArithmeticGate;
use plonky2_u32::gates::arithmetic_u32::U32ArithmeticGenerator;
use plonky2_u32::gates::comparison::ComparisonGate;
use plonky2_u32::gates::comparison::ComparisonGenerator;
use plonky2_u32::gates::range_check_u32::U32RangeCheckGate;
use plonky2_u32::gates::range_check_u32::U32RangeCheckGenerator;
use plonky2_u32::gates::subtraction_u32::U32SubtractionGate;
use plonky2_u32::gates::subtraction_u32::U32SubtractionGenerator;
use std::fs::File;
use std::io::Write;
use std::time::Instant;

use log::Level;
use plonky2::field::extension::Extendable;
use plonky2::field::secp256k1_scalar::Secp256K1Scalar;
use plonky2::field::types::Sample;
use plonky2::hash::hash_types::RichField;
use plonky2::iop::witness::PartialWitness;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::CircuitConfig;
use plonky2::plonk::config::{GenericConfig, KeccakGoldilocksConfig};
use plonky2::plonk::prover::prove;
use plonky2::util::serialization::{GateSerializer, WitnessGeneratorSerializer};
use plonky2::util::timing::TimingTree;
use plonky2::{impl_gate_serializer, impl_generator_serializer};
use plonky2_ecdsa::gadgets::ecdsa::*;

use plonky2::gadgets::arithmetic::EqualityGenerator;
use plonky2::gadgets::arithmetic_extension::QuotientGeneratorExtension;
use plonky2::gadgets::range_check::LowHighGenerator;
use plonky2::gadgets::split_base::BaseSumGenerator;
use plonky2::gadgets::split_join::{SplitGenerator, WireSplitGenerator};
use plonky2::gates::arithmetic_base::ArithmeticBaseGenerator;
use plonky2::gates::arithmetic_base::ArithmeticGate;
use plonky2::gates::arithmetic_extension::ArithmeticExtensionGate;
use plonky2::gates::arithmetic_extension::ArithmeticExtensionGenerator;
use plonky2::gates::base_sum::BaseSplitGenerator;
use plonky2::gates::base_sum::BaseSumGate;
use plonky2::gates::constant::ConstantGate;
use plonky2::gates::coset_interpolation::CosetInterpolationGate;
use plonky2::gates::coset_interpolation::InterpolationGenerator;
use plonky2::gates::exponentiation::ExponentiationGate;
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
use plonky2::gates::reducing_extension::ReducingExtensionGate;
use plonky2::gates::reducing_extension::ReducingGenerator as ReducingExtensionGenerator;
use plonky2::get_generator_tag_impl;
use plonky2::iop::generator::{
    ConstantGenerator, CopyGenerator, NonzeroTestGenerator, RandomValueGenerator,
};
use plonky2::read_generator_impl;
use plonky2_ecdsa::curve::curve_types::{Curve, CurveScalar};
use plonky2_ecdsa::curve::ecdsa::{sign_message, ECDSAPublicKey, ECDSASecretKey, ECDSASignature};
use plonky2_ecdsa::curve::secp256k1::Secp256K1;
use plonky2_ecdsa::gadgets::curve::CircuitBuilderCurve;
use plonky2_ecdsa::gadgets::nonnative::CircuitBuilderNonNative;
use plonky2_ecdsa::gadgets::nonnative::{
    NonNativeAdditionGenerator, NonNativeMultiplicationGenerator,
};

use jemallocator::Jemalloc;
use serde_json::json;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

pub struct ECDSAGateSerializer;
impl<F: RichField + Extendable<D>, const D: usize> GateSerializer<F, D> for ECDSAGateSerializer {
    impl_gate_serializer! {
        DefaultGateSerializer,
        ArithmeticGate,
        ArithmeticExtensionGate<D>,
        BaseSumGate<2>,
        BaseSumGate<4>,
        ComparisonGate<F, D>,
        ConstantGate,
        CosetInterpolationGate<F, D>,
        ExponentiationGate<F, D>,
        LookupGate,
        LookupTableGate,
        MulExtensionGate<D>,
        NoopGate,
        PoseidonMdsGate<F, D>,
        PoseidonGate<F, D>,
        PublicInputGate,
        RandomAccessGate<F, D>,
        ReducingExtensionGate<D>,
        ReducingGate<D>,
        U32AddManyGate<F, D>,
        U32ArithmeticGate<F, D>,
        U32RangeCheckGate<F, D>,
        U32SubtractionGate<F, D>
    }
}
pub struct ECDSAGeneratorSerializer {}

impl<F, const D: usize> WitnessGeneratorSerializer<F, D> for ECDSAGeneratorSerializer
where
    F: RichField + Extendable<D>,
{
    impl_generator_serializer! {
        DefaultGeneratorSerializer,
        ArithmeticBaseGenerator<F, D>,
        ArithmeticExtensionGenerator<F, D>,
        BaseSplitGenerator<2>,
        BaseSumGenerator<2>,
        ComparisonGenerator<F, D>,
        ConstantGenerator<F>,
        CopyGenerator,
        EqualityGenerator,
        ExponentiationGenerator<F, D>,
        GLVDecompositionGenerator<F, D>,
        InterpolationGenerator<F, D>,
        LookupGenerator,
        LookupTableGenerator,
        LowHighGenerator,
        MulExtensionGenerator<F, D>,
        NonNativeMultiplicationGenerator<F, D, Secp256K1Base>,
        NonNativeAdditionGenerator<F, D, Secp256K1Base>,
        NonNativeInverseGenerator<F, D, Secp256K1Base>,
        NonNativeSubtractionGenerator<F, D, Secp256K1Base>,
        NonzeroTestGenerator,
        PoseidonGenerator<F, D>,
        PoseidonMdsGenerator<D>,
        QuotientGeneratorExtension<D>,
        RandomAccessGenerator<F, D>,
        RandomValueGenerator,
        ReducingGenerator<D>,
        ReducingExtensionGenerator<D>,
        SplitGenerator,
        U32AddManyGenerator<F, D>,
        U32ArithmeticGenerator<F, D>,
        U32RangeCheckGenerator<F, D>,
        U32SubtractionGenerator<F, D>,
        WireSplitGenerator
    }
}

fn main() {
    env_logger::init();
    const D: usize = 2;
    type C = KeccakGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;

    type Curve = Secp256K1;

    let ecdsa_config = {
        CircuitConfig {
            zero_knowledge: true,
            num_routed_wires: 25,
            ..CircuitConfig::standard_ecc_config()
        }
    };
    // let build_prove = std::env::args().nth(1).expect("build or prove");
    // if build_prove != "build" && build_prove != "prove" {
    //     panic!("Must specify build or prove");
    // }

    // if build_prove == "build" {

    // TODO figure out issue with generator serialization to enable prover mem measurement
    let mut builder = CircuitBuilder::<F, D>::new(ecdsa_config);

    let msg = Secp256K1Scalar::rand();
    let msg_target = builder.constant_nonnative(msg);

    let sk = ECDSASecretKey::<Curve>(Secp256K1Scalar::rand());
    let pk = ECDSAPublicKey((CurveScalar(sk.0) * Curve::GENERATOR_PROJECTIVE).to_affine());

    let pk_target = ECDSAPublicKeyTarget(builder.constant_affine_point(pk.0));

    let sig = sign_message(msg, sk);

    let ECDSASignature { r, s } = sig;
    let r_target = builder.constant_nonnative(r);
    let s_target = builder.constant_nonnative(s);
    let sig_target = ECDSASignatureTarget {
        r: r_target,
        s: s_target,
    };

    verify_message_circuit(&mut builder, msg_target, sig_target, pk_target);
    builder.print_gate_counts(0);

    // dbg!(builder.num_gates());
    println!("building circuit");
    let start = Instant::now();
    let data = builder.build::<C>();
    let build_duration = start.elapsed();
    println!("circuit build duration: {:?}", build_duration);

    //     let gate_serializer = ECDSAGateSerializer {};
    //     let generator_serializer = ECDSAGeneratorSerializer {};

    //     let mut file = File::create("ecdsa.data").unwrap();
    //     let _ = file.write_all(
    //         &data
    //             .to_bytes(&gate_serializer, &generator_serializer)
    //             .unwrap(),
    //     );
    // } else {
    //     let mut file = File::open("ecdsa.data").expect("File not found");

    //     let mut buffer = Vec::new();
    //     file.read_to_end(&mut buffer)
    //         .expect("Failed to read the file");

    //     let gate_serializer = ECDSAGateSerializer {};
    //     let generator_serializer = ECDSAGeneratorSerializer {};
    //     let data: CircuitData<F, C, D> =
    //         CircuitData::from_bytes(&buffer, &gate_serializer, &generator_serializer).unwrap();
    let pw = PartialWitness::new();
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
      "Circuit": "ECDSA",
      "Backend": "Plonk+FRI",
      "Curve": "NaN",
      "ProverTime": proof_duration.as_secs_f32(),
      "VerifierTime": verify_duration.as_nanos() as f32 / 1000000.,
      "ProofSize": proof_len
    });

    let json_string = serde_json::to_string(&results).unwrap();

    let mut file = File::create("ecdsa.json").unwrap();
    let _ = file.write_all(json_string.as_bytes());
}
// }
