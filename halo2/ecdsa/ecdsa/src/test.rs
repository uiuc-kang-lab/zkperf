use crate::halo2;
use crate::integer;
use crate::maingate;
use crate::ecdsa::{AssignedEcdsaSig, AssignedPublicKey, EcdsaChip};
use crate::curves::bn256::{Fr as BnScalar, Bn256, G1Affine};
use crate::curves::secp256k1::Secp256k1Affine as Secp256k1;
use ecc::halo2::halo2curves::secp256k1::Fp;
use ecc::halo2::halo2curves::secp256k1::Fq;
use ecc::halo2::plonk::ProvingKey;
use ecc::integer::Range;
use ecc::maingate::big_to_fe;
use ecc::maingate::fe_to_big;
use ecc::maingate::RegionCtx;
use ecc::{EccConfig, GeneralEccChip};
use halo2::{
    dev::MockProver,
    transcript::{
      Blake2bRead, Blake2bWrite, Challenge255, TranscriptReadBuffer, TranscriptWriterBuffer,
    },
    SerdeFormat,
};
use halo2::poly::{
    commitment::Params,
    kzg::{
      commitment::{KZGCommitmentScheme, ParamsKZG},
      multiopen::{ProverSHPLONK, VerifierSHPLONK},
      strategy::SingleStrategy,
    },
};
use halo2::arithmetic::CurveAffine;
use halo2::circuit::{Layouter, SimpleFloorPlanner, Value};
use halo2::halo2curves::{
    ff::{Field, PrimeField},
    group::{Curve, Group},
};
use halo2::plonk::{Circuit, ConstraintSystem, Error, create_proof, keygen_pk, keygen_vk, verify_proof, VerifyingKey};
use integer::IntegerInstructions;
use maingate::{MainGate, MainGateConfig, RangeChip, RangeConfig, RangeInstructions};
use std::marker::PhantomData;
use std::{
    fs::File,
    io::{BufReader, Write},
    path::Path,
};
use serde::{Deserialize, Serialize};
use serde_json;

const BIT_LEN_LIMB: usize = 68;
const NUMBER_OF_LIMBS: usize = 4;


#[derive(Clone, Debug)]
struct TestCircuitEcdsaVerifyConfig {
    main_gate_config: MainGateConfig,
    range_config: RangeConfig,
}

impl TestCircuitEcdsaVerifyConfig {
    pub fn new<C: CurveAffine, N: PrimeField>(meta: &mut ConstraintSystem<N>) -> Self {
        let (rns_base, rns_scalar) =
            GeneralEccChip::<C, N, NUMBER_OF_LIMBS, BIT_LEN_LIMB>::rns();
        let main_gate_config = MainGate::<N>::configure(meta);
        let mut overflow_bit_lens: Vec<usize> = vec![];
        overflow_bit_lens.extend(rns_base.overflow_lengths());
        overflow_bit_lens.extend(rns_scalar.overflow_lengths());
        let composition_bit_lens = vec![BIT_LEN_LIMB / NUMBER_OF_LIMBS];

        let range_config = RangeChip::<N>::configure(
            meta,
            &main_gate_config,
            composition_bit_lens,
            overflow_bit_lens,
        );
        TestCircuitEcdsaVerifyConfig {
            main_gate_config,
            range_config,
        }
    }

    pub fn ecc_chip_config(&self) -> EccConfig {
        EccConfig::new(self.range_config.clone(), self.main_gate_config.clone())
    }

    pub fn config_range<N: PrimeField>(
        &self,
        layouter: &mut impl Layouter<N>,
    ) -> Result<(), Error> {
        let range_chip = RangeChip::<N>::new(self.range_config.clone());
        range_chip.load_table(layouter)?;

        Ok(())
    }
}

#[derive(Default, Clone)]
struct TestCircuitEcdsaVerify<E: CurveAffine, N: PrimeField> {
    public_key: Value<E>,
    signature: Value<(E::Scalar, E::Scalar)>,
    msg_hash: Value<E::Scalar>,

    aux_generator: E,
    window_size: usize,
    _marker: PhantomData<N>,
}

impl<E: CurveAffine, N: PrimeField> Circuit<N> for TestCircuitEcdsaVerify<E, N> {
    type Config = TestCircuitEcdsaVerifyConfig;
    type FloorPlanner = SimpleFloorPlanner;
    #[cfg(feature = "circuit-params")]
    type Params = ();

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<N>) -> Self::Config {
        TestCircuitEcdsaVerifyConfig::new::<E, N>(meta)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<N>,
    ) -> Result<(), Error> {
        let mut ecc_chip = GeneralEccChip::<E, N, NUMBER_OF_LIMBS, BIT_LEN_LIMB>::new(
            config.ecc_chip_config(),
        );

        layouter.assign_region(
            || "assign aux values",
            |region| {
                let offset = 0;
                let ctx = &mut RegionCtx::new(region, offset);

                ecc_chip.assign_aux_generator(ctx, Value::known(self.aux_generator))?;
                ecc_chip.assign_aux(ctx, self.window_size, 2)?;
                Ok(())
            },
        )?;

        let ecdsa_chip = EcdsaChip::new(ecc_chip.clone());
        let scalar_chip = ecc_chip.scalar_field_chip();

        layouter.assign_region(
            || "region 0",
            |region| {
                let offset = 0;
                let ctx = &mut RegionCtx::new(region, offset);

                let r = self.signature.map(|signature| signature.0);
                let s = self.signature.map(|signature| signature.1);
                let integer_r = ecc_chip.new_unassigned_scalar(r);
                let integer_s = ecc_chip.new_unassigned_scalar(s);
                let msg_hash = ecc_chip.new_unassigned_scalar(self.msg_hash);

                let r_assigned =
                    scalar_chip.assign_integer(ctx, integer_r, Range::Remainder)?;
                let s_assigned =
                    scalar_chip.assign_integer(ctx, integer_s, Range::Remainder)?;
                let sig = AssignedEcdsaSig {
                    r: r_assigned,
                    s: s_assigned,
                };

                let pk_in_circuit = ecc_chip.assign_point(ctx, self.public_key)?;
                let pk_assigned = AssignedPublicKey {
                    point: pk_in_circuit,
                };
                let msg_hash = scalar_chip.assign_integer(ctx, msg_hash, Range::Remainder)?;
                ecdsa_chip.verify(ctx, &sig, &pk_assigned, &msg_hash)
            },
        )?;

        config.config_range(&mut layouter)?;

        Ok(())
    }
}

pub fn get_kzg_params(params_dir: &str, degree: u32) -> ParamsKZG<Bn256> {
    let rng = rand::thread_rng();
    let path = format!("{}/{}.params", params_dir, degree);
    let params_path = Path::new(&path);
    if File::open(&params_path).is_err() {
      let params = ParamsKZG::<Bn256>::setup(degree, rng);
      let mut buf = Vec::new();
  
      params.write(&mut buf).expect("Failed to write params");
      let mut file = File::create(&params_path).expect("Failed to create params file");
      file
        .write_all(&buf[..])
        .expect("Failed to write params to file");
    }
  
    let mut params_fs = File::open(&params_path).expect("couldn't load params");
    let params = ParamsKZG::<Bn256>::read(&mut params_fs).expect("Failed to read params");
    params
}
  
pub fn serialize(data: &Vec<u8>, path: &str) -> u64 {
    let mut file = File::create(path).unwrap();
    file.write_all(data).unwrap();
    file.metadata().unwrap().len()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CircuitInput {
    pub pub_key: ([u8; 32], [u8; 32]),
    pub msg_hash: [u8; 32],
    pub sig_r: [u8; 32],
    pub sig_s: [u8; 32],
    pub aux: ([u8; 32], [u8; 32]),
}



pub fn test_ecdsa_verifier(step: String) {
    fn mod_n<C: CurveAffine>(x: C::Base) -> C::Scalar {
        let x_big = fe_to_big(x);
        big_to_fe(x_big)
    }

    fn run(step: String) {

        type C = Secp256k1;
        type N = BnScalar;

        if step == "generate" {
            let rng = rand::thread_rng();
            let g = C::generator();
    
            // Generate a key pair
            let sk = <C as CurveAffine>::ScalarExt::random(rng.clone());
            let public_key = (g * sk).to_affine();
            public_key.x;
            // Generate a valid signature
            // Suppose `m_hash` is the message hash
            let msg_hash = <C as CurveAffine>::ScalarExt::random(rng.clone());
    
            // Draw arandomness
            let k = <C as CurveAffine>::ScalarExt::random(rng.clone());
            let k_inv = k.invert().unwrap();
    
            // Calculate `r`
            let r_point = (g * k).to_affine().coordinates().unwrap();
            let x = r_point.x();
            let r = mod_n::<C>(*x);
    
            // Calculate `s`
            let s = k_inv * (msg_hash + (r * sk));
    
            let aux_generator = <Secp256k1 as CurveAffine>::CurveExt::random(rng.clone()).to_affine();
            
            let rand_input = CircuitInput {
                pub_key: (public_key.x.to_bytes(), public_key.y.to_bytes()),
                msg_hash: msg_hash.to_bytes(),
                sig_r: r.to_bytes(),
                sig_s: s.to_bytes(),
                aux: (aux_generator.x.to_bytes(), aux_generator.y.to_bytes())
            };

            serde_json::to_writer(
                File::create("sample.json").unwrap(), 
                &rand_input).unwrap();
            
        } else if step == "setup" {
            let circuit_input: CircuitInput = serde_json::from_reader(
                File::open("sample.json").unwrap()
            ).unwrap();

            let circuit = TestCircuitEcdsaVerify::<C, N> {
                public_key: Value::known(Secp256k1::from_xy(
                    Fp::from_bytes(&circuit_input.pub_key.0).unwrap(), 
                    Fp::from_bytes(&circuit_input.pub_key.1).unwrap()
                ).unwrap()),
                signature: Value::known((
                    Fq::from_bytes(&circuit_input.sig_r).unwrap(), 
                    Fq::from_bytes(&circuit_input.sig_s).unwrap()
                )),
                msg_hash: Value::known(
                    Fq::from_bytes(&circuit_input.msg_hash).unwrap()
                ),
                aux_generator: Secp256k1::from_xy(
                    Fp::from_bytes(&circuit_input.aux.0).unwrap(), 
                    Fp::from_bytes(&circuit_input.aux.1).unwrap()
                ).unwrap(),
                window_size: 4,
                ..Default::default()
            };
            
            let degree = 18 as u32;
            let params = get_kzg_params("./params_kzg", degree);
            let vk_circuit = circuit.clone();
            let vk = keygen_vk(&params, &vk_circuit).unwrap();
            drop(vk_circuit);
            let _ = serialize(&vk.to_bytes(SerdeFormat::RawBytes), "vkey");
            let pk_circuit = circuit.clone();
            let pk = keygen_pk(&params, vk.clone(), &pk_circuit).unwrap();
            drop(pk_circuit);
            let _ = serialize(&pk.to_bytes(SerdeFormat::RawBytes), "pkey");
            let proof_circuit = circuit.clone();
            let _prover = MockProver::run(degree, &proof_circuit, vec![vec![]]).unwrap();
        } else if step == "prove" {
            let rng = rand::thread_rng();
            let circuit_input: CircuitInput = serde_json::from_reader(
                File::open("sample.json").unwrap()
            ).unwrap();
            let circuit = TestCircuitEcdsaVerify::<C, N> {
                public_key: Value::known(Secp256k1::from_xy(
                    Fp::from_bytes(&circuit_input.pub_key.0).unwrap(), 
                    Fp::from_bytes(&circuit_input.pub_key.1).unwrap()
                ).unwrap()),
                signature: Value::known((
                    Fq::from_bytes(&circuit_input.sig_r).unwrap(), 
                    Fq::from_bytes(&circuit_input.sig_s).unwrap()
                )),
                msg_hash: Value::known(
                    Fq::from_bytes(&circuit_input.msg_hash).unwrap()
                ),
                aux_generator: Secp256k1::from_xy(
                    Fp::from_bytes(&circuit_input.aux.0).unwrap(), 
                    Fp::from_bytes(&circuit_input.aux.1).unwrap()
                ).unwrap(),
                window_size: 4,
                ..Default::default()
            };
            let degree = 18 as u32;
            let params = get_kzg_params("./params_kzg", degree);
            let mut transcript = Blake2bWrite::<_, G1Affine, Challenge255<_>>::init(vec![]);
            let pk = ProvingKey::read::<BufReader<File>, TestCircuitEcdsaVerify<C, N>>(
                &mut BufReader::new(File::open("pkey").unwrap()),
                SerdeFormat::RawBytes,
              )
              .unwrap();
            create_proof::<
              KZGCommitmentScheme<Bn256>,
              ProverSHPLONK<'_, Bn256>,
              Challenge255<G1Affine>,
              _,
              Blake2bWrite<Vec<u8>, G1Affine, Challenge255<G1Affine>>,
              TestCircuitEcdsaVerify<C, N>,
            >(
              &params,
              &pk,
              &[circuit],
              &[&[&[]]],
              rng.clone(),
              &mut transcript,
            )
            .unwrap();
            let proof = transcript.finalize();
            let _ = serialize(&proof, "proof");
        } else if step == "verify" {
            let degree = 18 as u32;
            let params = get_kzg_params("./params_kzg", degree);
            let proof = std::fs::read("proof").unwrap();  
            let strategy = SingleStrategy::new(&params);
            let mut transcript_read = Blake2bRead::<_, _, Challenge255<_>>::init(&proof[..]);
            let vk = VerifyingKey::read::<BufReader<File>, TestCircuitEcdsaVerify::<C, N>>(
                &mut BufReader::new(File::open("vkey").unwrap()),
                SerdeFormat::RawBytes,
              )
              .unwrap();
            assert!(
            verify_proof::<
                KZGCommitmentScheme<Bn256>,
                VerifierSHPLONK<'_, Bn256>,
                Challenge255<G1Affine>,
                Blake2bRead<&[u8], G1Affine, Challenge255<G1Affine>>,
                SingleStrategy<'_, Bn256>,
            >(&params, &vk, strategy, &[&[&[]]], &mut transcript_read)
            .is_ok());
        }
    }

    run(step);
    
}