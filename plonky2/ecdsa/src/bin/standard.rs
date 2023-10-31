use std::time::Instant;

use log::Level;
use plonky2::field::secp256k1_scalar::Secp256K1Scalar;
use plonky2::field::types::Sample;
use plonky2::iop::witness::PartialWitness;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::CircuitConfig;
use plonky2::plonk::config::{GenericConfig, KeccakGoldilocksConfig};
use plonky2::plonk::prover::prove;
use plonky2::util::timing::TimingTree;
use plonky2_ecdsa::gadgets::ecdsa::*;

use plonky2_ecdsa::curve::curve_types::{Curve, CurveScalar};
use plonky2_ecdsa::curve::ecdsa::{sign_message, ECDSAPublicKey, ECDSASecretKey, ECDSASignature};
use plonky2_ecdsa::curve::secp256k1::Secp256K1;
use plonky2_ecdsa::gadgets::curve::CircuitBuilderCurve;
use plonky2_ecdsa::gadgets::nonnative::CircuitBuilderNonNative;

use jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;


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
    // let config = CircuitConfig::standard_ecc_config();

    let pw = PartialWitness::new();
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

    // dbg!(builder.num_gates());
    println!("building circuit");
    let start = Instant::now();
    let data = builder.build::<C>();
    let build_duration = start.elapsed();
    println!("circuit build duration: {:?}", build_duration);
    
    println!("proving circuit");
  let mut timing = TimingTree::new("prove", Level::Info);
  let proof = prove::<F, C, D>(
    &data.prover_only,
    &data.common,
    pw,
    &mut timing).unwrap();
  timing.pop();
  timing.print();

//   println!("Proving time: {:?}", proof_duration);

  let proof_bytes = proof.to_bytes();
  let proof_len = proof_bytes.len();
  println!("Proof size: {} bytes", proof_len);

  println!("verifying circuit");
  let mut timing = TimingTree::new("verify", Level::Info);
  data.verify(proof.clone()).expect("verify error");
  timing.pop();
  timing.print();

//   let verify_duration = timing.duration();
//   println!("Verifying time: {:?}", verify_duration);
}
