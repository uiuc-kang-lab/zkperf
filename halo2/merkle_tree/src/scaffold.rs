


use std::{
    cell::RefCell,
    env::{set_var, var},
    fs::{self, File},
    io::{BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    marker::PhantomData,
    time::Instant,
};

use axiom_eth::{
    keccak::{FixedLenRLCs, KeccakChip, VarLenRLCs},
    rlp::{builder::RlcThreadBuilder, RlpChip},
    util::{
        circuit::{PinnableCircuit, PreCircuit},
        EthConfigPinning, Halo2ConfigPinning
    },
    EthChip, EthCircuitBuilder, ETH_LOOKUP_BITS,
};
use bincode;
use halo2_base::{
    gates::builder::{CircuitBuilderStage, GateThreadBuilder},
    halo2_proofs::{
        dev::MockProver,
        halo2curves::bn256::{Bn256, Fr, G1Affine},
        plonk::{create_proof, verify_proof, Circuit, VerifyingKey, ProvingKey, keygen_pk, keygen_vk},
        poly::{
            commitment::{Params, ParamsProver},
            kzg::{
                commitment::{KZGCommitmentScheme, ParamsKZG},
                multiopen::{ProverSHPLONK, VerifierSHPLONK},
                strategy::SingleStrategy,
            },
        },
        transcript::{
            Blake2bRead, Blake2bWrite, Challenge255, TranscriptReadBuffer, TranscriptWriterBuffer,
        },
        SerdeFormat,
    },
    safe_types::RangeChip,
    AssignedValue, Context,
};
use itertools::Itertools;
use serde::de::DeserializeOwned;
use serde_json; 
use snark_verifier_sdk::{
    halo2::{gen_snark_shplonk, read_snark, PoseidonTranscript},
    CircuitExt, NativeLoader, read_pk
};
use crate::cmd::{Cli, SnarkCmd};

#[derive(Clone)]
pub struct EthScaffold<T, FN, F1> {
    f: FN,
    private_inputs: T,
    _f1: PhantomData<F1>,
}

impl<T, FN, F1> PreCircuit for EthScaffold<T, FN, F1>
where
    FN: FnOnce(
        &mut GateThreadBuilder<Fr>,
        &EthChip<Fr>,
        &mut KeccakChip<Fr>,
        T,
        &mut Vec<AssignedValue<Fr>>,
    ) -> F1,
    F1: FnOnce(&mut Context<Fr>, &mut Context<Fr>, &EthChip<Fr>) + Clone,
{
    type Pinning = EthConfigPinning;

    fn create_circuit(
        self,
        stage: CircuitBuilderStage,
        pinning: Option<Self::Pinning>,
        params: &ParamsKZG<Bn256>,
    ) -> impl PinnableCircuit<Fr> {
        let mut builder = RlcThreadBuilder::new(stage == CircuitBuilderStage::Prover);
        let lookup_bits: usize =
            var("LOOKUP_BITS").unwrap_or_else(|_| ETH_LOOKUP_BITS.to_string()).parse().unwrap();
        set_var("LOOKUP_BITS", lookup_bits.to_string());
        let range = RangeChip::default(lookup_bits);
        let chip = EthChip::new(RlpChip::new(&range, None), None);
        let mut keccak = KeccakChip::default();

        let mut assigned_instances = vec![];
        let f_phase1 = (self.f)(
            &mut builder.gate_builder,
            &chip,
            &mut keccak,
            self.private_inputs,
            &mut assigned_instances,
        );
        let break_points = pinning.map(|p| p.break_points);
        let circuit = EthCircuitBuilder::new(
            assigned_instances,
            builder,
            RefCell::new(keccak),
            range,
            break_points,
            |builder: &mut RlcThreadBuilder<Fr>,
                rlp: RlpChip<Fr>,
                keccak_rlcs: (FixedLenRLCs<Fr>, VarLenRLCs<Fr>)| {
                let chip = EthChip::new(rlp, Some(keccak_rlcs));
                let (ctx_gate, ctx_rlc) = builder.rlc_ctx_pair();
                (f_phase1)(ctx_gate, ctx_rlc, &chip);
                if ctx_gate.advice.is_empty() {
                    builder.gate_builder.threads[1].pop();
                }
            },
        );
        if stage != CircuitBuilderStage::Prover {
            circuit.config(params.k() as usize, Some(109));
        }
        circuit
    }
}

pub fn run_eth<T, FN, F1>(f: FN, cli: Cli)
where
    T: DeserializeOwned + Clone,
    FN: FnOnce(
        &mut Context<Fr>,
        &EthChip<Fr>,
        &mut KeccakChip<Fr>,
        T,
        &mut Vec<AssignedValue<Fr>>,
    ) -> F1 + Clone,
    F1: FnOnce(&mut Context<Fr>, &mut Context<Fr>, &EthChip<Fr>) + Clone,
{
    run_eth_builder(
        |builder, chip, keccak, inp, public| f(builder.main(0), chip, keccak, inp, public),
        cli,
    )
}

pub fn run_eth_builder<T, FN, F1>(f: FN, cli: Cli)
where
    T: DeserializeOwned + Clone,
    FN: FnOnce(
        &mut GateThreadBuilder<Fr>,
        &EthChip<Fr>,
        &mut KeccakChip<Fr>,
        T,
        &mut Vec<AssignedValue<Fr>>,
    ) -> F1 + Clone,
    F1: FnOnce(&mut Context<Fr>, &mut Context<Fr>, &EthChip<Fr>) + Clone,
{
    let name = &cli.name;
    let input_path = PathBuf::from("data")
        .join(cli.input_path.clone().unwrap_or_else(|| PathBuf::from(format!("{name}.json"))));
    let private_inputs: T = serde_json::from_reader(
        File::open(&input_path)
            .unwrap_or_else(|e| panic!("Input file not found at {input_path:?}. {e:?}")),
    )
    .expect("Input file should be a valid JSON file");
    run_eth_builder_on_inputs(f, cli, private_inputs)
}

pub fn run_eth_builder_on_inputs<T, FN, F1>(f: FN, cli: Cli, private_inputs: T)
where
    T: DeserializeOwned + Clone,
    FN: FnOnce(
        &mut GateThreadBuilder<Fr>,
        &EthChip<Fr>,
        &mut KeccakChip<Fr>,
        T,
        &mut Vec<AssignedValue<Fr>>,
    ) -> F1 + Clone,
    F1: FnOnce(&mut Context<Fr>, &mut Context<Fr>, &EthChip<Fr>) + Clone,
{
    let precircuit = EthScaffold { f, private_inputs, _f1: PhantomData };
    run_cli(precircuit, cli);
}

pub fn run_cli<P: PreCircuit + Clone>(precircuit: P, cli: Cli) {

    let name = cli.name;
    let k = cli.degree;

    let config_path = cli.config_path.unwrap_or_else(|| PathBuf::from("configs"));
    let data_path = cli.data_path.unwrap_or_else(|| PathBuf::from("data"));
    fs::create_dir_all(&config_path).unwrap();
    fs::create_dir_all(&data_path).unwrap();

    let params = get_kzg_params(k);
    match cli.command {
        SnarkCmd::Mock => {
            let circuit = precircuit.create_circuit(CircuitBuilderStage::Mock, None, &params);
            MockProver::run(k, &circuit, circuit.instances()).unwrap().assert_satisfied();
        }
        SnarkCmd::Keygen => {

            let pk_path = data_path.join(PathBuf::from(format!("{name}.pk")));
            let vk_path = data_path.join(PathBuf::from(format!("{name}.vk")));
            let pinning_path = config_path.join(PathBuf::from(format!("{name}.json")));
            let key_gen_circuit = precircuit.clone().create_circuit(CircuitBuilderStage::Keygen, None, &params);
            let vk = keygen_vk(&params, &key_gen_circuit).unwrap();
            let pk = keygen_pk(&params, vk.clone(), &key_gen_circuit).unwrap();

            key_gen_circuit.write_pinning(pinning_path.clone());
            serialize(&vk.to_bytes(SerdeFormat::RawBytes), &vk_path);
            serialize(&pk.to_bytes(SerdeFormat::RawBytes), &pk_path);

        }
        SnarkCmd::Prove => {
            let rng = rand::thread_rng();
            let pk_path = data_path.join(PathBuf::from(format!("{name}.pk")));
            let pinning_path = config_path.join(PathBuf::from(format!("{name}.json")));
            let proof_path = data_path.join(PathBuf::from(format!("{name}.proof")));
            let instances_path = data_path.join(PathBuf::from(format!("{name}.instances")));
            let pinning = P::Pinning::from_path(pinning_path);
            pinning.set_var();
            
            let circuit =
                precircuit.create_circuit(CircuitBuilderStage::Prover, Some(pinning), &params);
            let pk = custom_read_pk(&pk_path, &circuit);

            let instances = circuit.instances();
            bincode::serialize_into(File::create(instances_path).unwrap(), &instances).unwrap();
            
            let instances = instances.iter().map(Vec::as_slice).collect_vec();

            let mut transcript = Blake2bWrite::<_, G1Affine, Challenge255<_>>::init(vec![]);
            create_proof::<
                KZGCommitmentScheme<Bn256>,
                ProverSHPLONK<'_, Bn256>,
                Challenge255<G1Affine>,
                _,
                Blake2bWrite<Vec<u8>, G1Affine, Challenge255<G1Affine>>,
                _,
            >(
                &params,
                &pk,
                &[circuit],
                &[&instances],
                rng,
                &mut transcript,
            )
            .unwrap();
            let proof = transcript.finalize();
            serialize(&proof, &proof_path);
        }
        SnarkCmd::Verify => {
            let vk_path = data_path.join(PathBuf::from(format!("{name}.vk")));
            let proof_path = data_path.join(PathBuf::from(format!("{name}.proof")));
            let instances_path = data_path.join(PathBuf::from(format!("{name}.instances")));
            let circuit = precircuit.create_circuit(CircuitBuilderStage::Keygen, None, &params);
            let vk = custom_read_vk(vk_path, &circuit);
            let proof = std::fs::read(proof_path).unwrap();

            let strategy = SingleStrategy::new(&params);
            let mut transcript = Blake2bRead::<_, _, Challenge255<_>>::init(&proof[..]);

            let instances: Vec<Vec<Fr>> = bincode::deserialize_from(File::open(instances_path).unwrap()).unwrap();
            let instances = instances.iter().map(Vec::as_slice).collect_vec();
            verify_proof::<
                KZGCommitmentScheme<Bn256>,
                VerifierSHPLONK<'_, Bn256>,
                _,
                _,
                SingleStrategy<'_, Bn256>,
            >(&params, &vk, strategy, &[&instances], &mut transcript)
            .unwrap();
        }
        SnarkCmd::Full => {
            let rng = rand::thread_rng();
            let pinning_path = config_path.join(PathBuf::from(format!("{name}.json")));
            let key_gen_circuit = precircuit.clone().create_circuit(CircuitBuilderStage::Keygen, None, &params);
            let vk = keygen_vk(&params, &key_gen_circuit).unwrap();
            let pk = keygen_pk(&params, vk.clone(), &key_gen_circuit).unwrap();
            key_gen_circuit.write_pinning(pinning_path.clone());
            let pinning = P::Pinning::from_path(pinning_path);
            pinning.set_var();
            let circuit =
            precircuit.create_circuit(CircuitBuilderStage::Prover, Some(pinning), &params);
            let instances = circuit.instances();
            let instances = instances.iter().map(Vec::as_slice).collect_vec();

            let mut transcript = Blake2bWrite::<_, G1Affine, Challenge255<_>>::init(vec![]);
            create_proof::<
              KZGCommitmentScheme<Bn256>,
              ProverSHPLONK<'_, Bn256>,
              Challenge255<G1Affine>,
              _,
              Blake2bWrite<Vec<u8>, G1Affine, Challenge255<G1Affine>>,
              _,
            >(
              &params,
              &pk,
              &[circuit],
              &[&instances],
              rng,
              &mut transcript,
            )
            .unwrap();
            let proof = transcript.finalize();

            let strategy = SingleStrategy::new(&params);
            let mut transcript = Blake2bRead::<_, _, Challenge255<_>>::init(&proof[..]);

            verify_proof::<
                KZGCommitmentScheme<Bn256>,
                VerifierSHPLONK<'_, Bn256>,
                _,
                _,
                SingleStrategy<'_, Bn256>,
            >(&params, &vk, strategy, &[&instances], &mut transcript)
            .unwrap();
        }
    }
}


pub fn get_kzg_params(degree: u32) -> ParamsKZG<Bn256> {
    let rng = rand::thread_rng();
    let path = format!("params/kzg_bn254_{}.params", degree);
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

pub fn serialize(data: &Vec<u8>, path: &PathBuf) -> u64 {
    let mut file = File::create(path).unwrap();
    file.write_all(data).unwrap();
    file.metadata().unwrap().len()
}


fn custom_read_pk<C, P>(fname: P, _: &C) -> ProvingKey<G1Affine>
where
    C: Circuit<Fr>,
    P: AsRef<Path>,
{
    ProvingKey::read::<_, C>(
        &mut BufReader::new(File::open(&fname).unwrap()),
        SerdeFormat::RawBytes,
      )
      .unwrap_or_else(|e| panic!("Failed to open file: {:?}: {e:?}", fname.as_ref()))
}

fn custom_read_vk<C, P>(fname: P, _: &C) -> VerifyingKey<G1Affine>
where
    C: Circuit<Fr>,
    P: AsRef<Path>,
{
    VerifyingKey::read::<_, C>(
        &mut BufReader::new(File::open(&fname).unwrap()),
        SerdeFormat::RawBytes,
      )
      .unwrap_or_else(|e| panic!("Failed to open file: {:?}: {e:?}", fname.as_ref()))
}

