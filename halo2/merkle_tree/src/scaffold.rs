


use std::{
    cell::RefCell,
    env::{set_var, var},
    fs::{self, File},
    io::{BufReader, BufWriter},
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
use halo2_base::{
    gates::builder::{CircuitBuilderStage, GateThreadBuilder},
    halo2_proofs::{
        dev::MockProver,
        halo2curves::bn256::{Bn256, Fr, G1Affine},
        plonk::{verify_proof, Circuit, VerifyingKey, ProvingKey},
        poly::{
            commitment::{Params, ParamsProver},
            kzg::{
                commitment::{KZGCommitmentScheme, ParamsKZG},
                multiopen::VerifierSHPLONK,
                strategy::SingleStrategy,
            },
        },
        SerdeFormat,
    },
    utils::fs::gen_srs,
    safe_types::RangeChip,
    AssignedValue, Context,
};
use serde::de::DeserializeOwned;
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

    let params = gen_srs(k);
    println!("Universal trusted setup (unsafe!) available at: params/kzg_bn254_{k}.srs");
    match cli.command {
        SnarkCmd::Mock => {
            let circuit = precircuit.create_circuit(CircuitBuilderStage::Mock, None, &params);
            MockProver::run(k, &circuit, circuit.instances()).unwrap().assert_satisfied();
        }
        SnarkCmd::Keygen => {
            let pk_path = data_path.join(PathBuf::from(format!("{name}.pk")));
            if pk_path.exists() {
                fs::remove_file(&pk_path).unwrap();
            }
            let pinning_path = config_path.join(PathBuf::from(format!("{name}.json")));
            let pk = precircuit.create_pk(&params, &pk_path, pinning_path);
            println!("Proving key written to: {pk_path:?}");

            let vk_path = data_path.join(PathBuf::from(format!("{name}.vk")));
            let f = File::create(&vk_path).unwrap();
            let mut writer = BufWriter::new(f);
            pk.get_vk()
                .write(&mut writer, SerdeFormat::RawBytes)
                .expect("writing vkey should not fail");
            println!("Verifying key written to: {vk_path:?}");
        }
        SnarkCmd::Prove => {
            let pinning_path = config_path.join(PathBuf::from(format!("{name}.json")));
            let pinning = P::Pinning::from_path(pinning_path);
            pinning.set_var();
            let circuit =
                precircuit.create_circuit(CircuitBuilderStage::Prover, Some(pinning), &params);
            let pk_path = data_path.join(PathBuf::from(format!("{name}.pk")));
            let pk = custom_read_pk(pk_path, &circuit);

            let snark_path = data_path.join(PathBuf::from(format!("{name}.snark")));
            if snark_path.exists() {
                fs::remove_file(&snark_path).unwrap();
            }
            let start = Instant::now();
            let snark = gen_snark_shplonk(&params, &pk, circuit, Some(&snark_path));
            let duration = start.elapsed();
            println!("Proof Generation Time: {:?}", duration);
            println!("Proof Size: {}", snark.proof.len());
            println!("Snark written to: {snark_path:?}");
        }
        SnarkCmd::Verify => {
            let vk_path = data_path.join(PathBuf::from(format!("{name}.vk")));
            let circuit = precircuit.create_circuit(CircuitBuilderStage::Keygen, None, &params);
            let vk = custom_read_vk(vk_path, &circuit);
            let snark_path = data_path.join(PathBuf::from(format!("{name}.snark")));
            let snark = read_snark(&snark_path)
                .unwrap_or_else(|e| panic!("Snark not found at {snark_path:?}. {e:?}"));

            let verifier_params = params.verifier_params();
            let strategy = SingleStrategy::new(&params);
            let mut transcript =
                PoseidonTranscript::<NativeLoader, &[u8]>::new::<0>(&snark.proof[..]);
            let instance = &snark.instances[0][..];
            let start = Instant::now();
            verify_proof::<
                KZGCommitmentScheme<Bn256>,
                VerifierSHPLONK<'_, Bn256>,
                _,
                _,
                SingleStrategy<'_, Bn256>,
            >(verifier_params, &vk, strategy, &[&[instance]], &mut transcript)
            .unwrap();
            let duration = start.elapsed();
            println!("Verification Time: {:?}", duration);
            println!("Snark verified successfully!");
        }
        // SnarkCmd::Full => {
        //     let pk_path = data_path.join(PathBuf::from(format!("{name}.pk")));
        //     if pk_path.exists() {
        //         fs::remove_file(&pk_path).unwrap();
        //     }
        //     let pinning_path = config_path.join(PathBuf::from(format!("{name}.json")));
        //     let pk = precircuit.clone().create_pk(&params, &pk_path, pinning_path.clone());
        //     println!("Proving key written to: {pk_path:?}");

        //     let vk_path = data_path.join(PathBuf::from(format!("{name}.vk")));
        //     let f = File::create(&vk_path).unwrap();
        //     let mut writer = BufWriter::new(f);
        //     pk.get_vk()
        //         .write(&mut writer, SerdeFormat::RawBytes)
        //         .expect("writing vkey should not fail");
        //     println!("Verifying key written to: {vk_path:?}");

        //     let pinning = P::Pinning::from_path(pinning_path);
        //     pinning.set_var();

        //     let circuit =
        //         precircuit.create_circuit(CircuitBuilderStage::Prover, Some(pinning), &params);
        //     let snark_path = data_path.join(PathBuf::from(format!("{name}.snark")));
        //     if snark_path.exists() {
        //         fs::remove_file(&snark_path).unwrap();
        //     }

        //     let vk = custom_read_vk(vk_path, &circuit);

        //     gen_snark_shplonk(&params, &pk, circuit, Some(&snark_path));
        //     println!("Proof written to: {snark_path:?}");
            
        //     let snark = read_snark(&snark_path)
        //         .unwrap_or_else(|e| panic!("Snark not found at {snark_path:?}. {e:?}"));
        //     let verifier_params = params.verifier_params();
        //     let strategy = SingleStrategy::new(&params);
        //     let mut transcript =
        //         PoseidonTranscript::<NativeLoader, &[u8]>::new::<0>(&snark.proof[..]);
        //     let instance = &snark.instances[0][..];
        //     verify_proof::<
        //         KZGCommitmentScheme<Bn256>,
        //         VerifierSHPLONK<'_, Bn256>,
        //         _,
        //         _,
        //         SingleStrategy<'_, Bn256>,
        //     >(verifier_params, &vk, strategy, &[&[instance]], &mut transcript)
        //     .unwrap();
        //     println!("Proof verified successfully!");
        // }
    }
}

fn custom_read_pk<C, P>(fname: P, _: &C) -> ProvingKey<G1Affine>
where
    C: Circuit<Fr>,
    P: AsRef<Path>,
{
    read_pk::<C>(fname.as_ref())
        .unwrap_or_else(|e| panic!("Failed to open file: {:?}: {e:?}", fname.as_ref()))
}

fn custom_read_vk<C, P>(fname: P, _: &C) -> VerifyingKey<G1Affine>
where
    C: Circuit<Fr>,
    P: AsRef<Path>,
{
    let f = File::open(&fname)
        .unwrap_or_else(|e| panic!("Failed to open file: {:?}: {e:?}", fname.as_ref()));
    let mut bufreader = BufReader::new(f);
    VerifyingKey::read::<_, C>(&mut bufreader, SerdeFormat::RawBytes).expect("Could not read vkey")
}

