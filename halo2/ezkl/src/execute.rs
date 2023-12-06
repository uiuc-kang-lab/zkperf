use crate::circuit::CheckMode;
#[cfg(not(target_arch = "wasm32"))]
use crate::commands::CalibrationTarget;
use crate::commands::Commands;
#[cfg(not(target_arch = "wasm32"))]
use crate::eth::{deploy_da_verifier_via_solidity, deploy_verifier_via_solidity};
#[cfg(not(target_arch = "wasm32"))]
use crate::eth::{fix_da_sol, get_contract_artifacts, verify_proof_via_solidity};
use crate::graph::input::GraphData;
use crate::graph::{GraphCircuit, GraphSettings, GraphWitness, Model};
#[cfg(not(target_arch = "wasm32"))]
use crate::graph::{TestDataSource, TestSources};
use crate::pfsys::evm::aggregation::AggregationCircuit;
#[cfg(not(target_arch = "wasm32"))]
use crate::pfsys::evm::{single::gen_evm_verifier, YulCode};
use crate::pfsys::{
    create_keys, load_pk, load_vk, save_params, save_pk, swap_proof_commitments_kzg, Snark,
    StrategyType, TranscriptType,
};
use crate::pfsys::{create_proof_circuit_kzg, verify_proof_circuit_kzg};
use crate::pfsys::{save_vk, srs::*};
use crate::RunArgs;
#[cfg(not(target_arch = "wasm32"))]
use ethers::types::H160;
use gag::Gag;
use halo2_proofs::dev::VerifyFailure;
use halo2_proofs::poly::commitment::Params;
use halo2_proofs::poly::commitment::ParamsProver;
use halo2_proofs::poly::kzg::commitment::KZGCommitmentScheme;
use halo2_proofs::poly::kzg::strategy::AccumulatorStrategy;
use halo2_proofs::poly::kzg::{
    commitment::ParamsKZG, strategy::SingleStrategy as KZGSingleStrategy,
};
#[cfg(not(target_arch = "wasm32"))]
use halo2_solidity_verifier;
use halo2curves::bn256::{Bn256, Fr, G1Affine};
#[cfg(not(target_arch = "wasm32"))]
use halo2curves::ff::Field;
#[cfg(not(target_arch = "wasm32"))]
use indicatif::{ProgressBar, ProgressStyle};
use instant::Instant;
#[cfg(not(target_arch = "wasm32"))]
use itertools::Itertools;
#[cfg(not(target_arch = "wasm32"))]
use log::debug;
use log::{info, trace};
#[cfg(feature = "render")]
use plotters::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use rand::Rng;
#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use std::error::Error;
use std::fs::File;
#[cfg(not(target_arch = "wasm32"))]
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
#[cfg(not(target_arch = "wasm32"))]
use std::process::Command;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
#[cfg(not(target_arch = "wasm32"))]
use std::sync::OnceLock;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;
use thiserror::Error;
#[cfg(not(target_arch = "wasm32"))]
use tokio_util::codec::{BytesCodec, FramedRead};

#[cfg(not(target_arch = "wasm32"))]
static _SOLC_REQUIREMENT: OnceLock<bool> = OnceLock::new();
#[cfg(not(target_arch = "wasm32"))]
fn check_solc_requirement() {
    info!("checking solc installation..");
    _SOLC_REQUIREMENT.get_or_init(|| match Command::new("solc").arg("--version").output() {
        Ok(output) => {
            debug!("solc output: {:#?}", output);
            debug!("solc output success: {:#?}", output.status.success());
            if !output.status.success() {
                log::error!(
                    "`solc` check failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                return false;
            }
            debug!("solc check passed, proceeding");
            true
        }
        Err(_) => {
            log::error!("`solc` check failed: solc not found");
            false
        }
    });
}

/// A wrapper for tensor related errors.
#[derive(Debug, Error)]
pub enum ExecutionError {
    /// Shape mismatch in a operation
    #[error("verification failed")]
    VerifyError(Vec<VerifyFailure>),
}

/// Run an ezkl command with given args
pub async fn run(command: Commands) -> Result<(), Box<dyn Error>> {
    match command {
        Commands::Empty => Ok(()),
        #[cfg(not(target_arch = "wasm32"))]
        Commands::Fuzz {
            witness,
            compiled_circuit,
            transcript,
            num_runs,
        } => fuzz(compiled_circuit, witness, transcript, num_runs),

        Commands::GenSrs { srs_path, logrows } => gen_srs_cmd(srs_path, logrows as u32),
        #[cfg(not(target_arch = "wasm32"))]
        Commands::GetSrs {
            srs_path,
            settings_path,
            logrows,
            check,
        } => get_srs_cmd(srs_path, settings_path, logrows, check).await,
        Commands::Table { model, args } => table(model, args),
        #[cfg(feature = "render")]
        Commands::RenderCircuit {
            model,
            output,
            args,
        } => render(model, output, args),
        Commands::GenSettings {
            model,
            settings_path,
            args,
        } => gen_circuit_settings(model, settings_path, args),
        #[cfg(not(target_arch = "wasm32"))]
        Commands::CalibrateSettings {
            model,
            settings_path,
            data,
            target,
            scales,
            max_logrows,
        } => calibrate(model, data, settings_path, target, scales, max_logrows),
        Commands::GenWitness {
            data,
            compiled_circuit,
            output,
            vk_path,
            srs_path,
        } => gen_witness(compiled_circuit, data, Some(output), vk_path, srs_path)
            .await
            .map(|_| ()),
        Commands::Mock { model, witness } => mock(model, witness),
        #[cfg(not(target_arch = "wasm32"))]
        Commands::CreateEVMVerifier {
            vk_path,
            srs_path,
            settings_path,
            sol_code_path,
            abi_path,
        } => create_evm_verifier(vk_path, srs_path, settings_path, sol_code_path, abi_path),
        #[cfg(not(target_arch = "wasm32"))]
        Commands::CreateEVMDataAttestation {
            vk_path,
            srs_path,
            settings_path,
            sol_code_path,
            abi_path,
            data,
        } => create_evm_data_attestation(
            vk_path,
            srs_path,
            settings_path,
            sol_code_path,
            abi_path,
            data,
        ),
        #[cfg(not(target_arch = "wasm32"))]
        Commands::CreateEVMVerifierAggr {
            vk_path,
            srs_path,
            sol_code_path,
            abi_path,
            aggregation_settings,
        } => create_evm_aggregate_verifier(
            vk_path,
            srs_path,
            sol_code_path,
            abi_path,
            aggregation_settings,
        ),
        Commands::CompileCircuit {
            model,
            compiled_circuit,
            settings_path,
        } => compile_circuit(model, compiled_circuit, settings_path),
        Commands::Setup {
            compiled_circuit,
            srs_path,
            vk_path,
            pk_path,
            witness,
        } => setup(compiled_circuit, srs_path, vk_path, pk_path, witness),
        #[cfg(not(target_arch = "wasm32"))]
        Commands::SetupTestEVMData {
            data,
            compiled_circuit,
            test_data,
            rpc_url,
            input_source,
            output_source,
        } => {
            setup_test_evm_witness(
                data,
                compiled_circuit,
                test_data,
                rpc_url,
                input_source,
                output_source,
            )
            .await
        }
        #[cfg(not(target_arch = "wasm32"))]
        Commands::TestUpdateAccountCalls {
            addr,
            data,
            rpc_url,
        } => test_update_account_calls(addr, data, rpc_url).await,
        #[cfg(not(target_arch = "wasm32"))]
        Commands::SwapProofCommitments {
            proof_path,
            witness_path,
        } => swap_proof_commitments(proof_path, witness_path),
        #[cfg(not(target_arch = "wasm32"))]
        Commands::Prove {
            witness,
            compiled_circuit,
            pk_path,
            proof_path,
            srs_path,
            proof_type,
            check_mode,
        } => prove(
            witness,
            compiled_circuit,
            pk_path,
            Some(proof_path),
            srs_path,
            proof_type,
            check_mode,
        )
        .map(|_| ()),
        Commands::MockAggregate {
            aggregation_snarks,
            logrows,
            split_proofs,
        } => mock_aggregate(aggregation_snarks, logrows, split_proofs),
        Commands::SetupAggregate {
            sample_snarks,
            vk_path,
            pk_path,
            srs_path,
            logrows,
            split_proofs,
        } => setup_aggregate(
            sample_snarks,
            vk_path,
            pk_path,
            srs_path,
            logrows,
            split_proofs,
        ),
        Commands::Aggregate {
            proof_path,
            aggregation_snarks,
            pk_path,
            srs_path,
            transcript,
            logrows,
            check_mode,
            split_proofs,
        } => aggregate(
            proof_path,
            aggregation_snarks,
            pk_path,
            srs_path,
            transcript,
            logrows,
            check_mode,
            split_proofs,
        ),
        Commands::Verify {
            proof_path,
            settings_path,
            vk_path,
            srs_path,
        } => verify(proof_path, settings_path, vk_path, srs_path),
        Commands::VerifyAggr {
            proof_path,
            vk_path,
            srs_path,
            logrows,
        } => verify_aggr(proof_path, vk_path, srs_path, logrows),
        #[cfg(not(target_arch = "wasm32"))]
        Commands::DeployEvmVerifier {
            sol_code_path,
            rpc_url,
            addr_path,
            optimizer_runs,
            private_key,
        } => {
            deploy_evm(
                sol_code_path,
                rpc_url,
                addr_path,
                optimizer_runs,
                private_key,
            )
            .await
        }
        #[cfg(not(target_arch = "wasm32"))]
        Commands::DeployEvmDataAttestation {
            data,
            settings_path,
            sol_code_path,
            rpc_url,
            addr_path,
            optimizer_runs,
            private_key,
        } => {
            deploy_da_evm(
                data,
                settings_path,
                sol_code_path,
                rpc_url,
                addr_path,
                optimizer_runs,
                private_key,
            )
            .await
        }
        #[cfg(not(target_arch = "wasm32"))]
        Commands::VerifyEVM {
            proof_path,
            addr_verifier,
            rpc_url,
            addr_da,
        } => verify_evm(proof_path, addr_verifier, rpc_url, addr_da).await,
        Commands::PrintProofHex { proof_path } => print_proof_hex(proof_path),
        #[cfg(not(target_arch = "wasm32"))]
        Commands::GetHubCredentials {
            api_key,
            username,
            url,
        } => get_hub_credentials(api_key.as_deref(), url.as_deref(), &username)
            .await
            .map(|_| ()),

        #[cfg(not(target_arch = "wasm32"))]
        Commands::CreateHubArtifact {
            api_key,
            uncompiled_circuit,
            data,
            organization_id,
            artifact_name,
            url,
            args,
            target,
        } => deploy_model(
            api_key.as_deref(),
            url.as_deref(),
            &uncompiled_circuit,
            &data,
            &artifact_name,
            &organization_id,
            &args,
            &target,
        )
        .await
        .map(|_| ()),
        #[cfg(not(target_arch = "wasm32"))]
        Commands::GetHubArtifact {
            api_key,
            artifact_id,
            url,
        } => get_deployed_model(api_key.as_deref(), url.as_deref(), &artifact_id)
            .await
            .map(|_| ()),
        #[cfg(not(target_arch = "wasm32"))]
        Commands::GetHubProof {
            api_key,
            proof_id,
            url,
        } => get_hub_proof(api_key.as_deref(), url.as_deref(), &proof_id)
            .await
            .map(|_| ()),
        #[cfg(not(target_arch = "wasm32"))]
        Commands::ProveHub {
            api_key,
            artifact_id,
            data,
            url,
        } => prove_hub(api_key.as_deref(), url.as_deref(), &artifact_id, &data)
            .await
            .map(|_| ()),
    }
}

pub(crate) fn gen_srs_cmd(srs_path: PathBuf, logrows: u32) -> Result<(), Box<dyn Error>> {
    let params = gen_srs::<KZGCommitmentScheme<Bn256>>(logrows);
    save_params::<KZGCommitmentScheme<Bn256>>(&srs_path, &params)?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
async fn fetch_srs(uri: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let pb = {
        let pb = init_spinner();
        pb.set_message("Downloading SRS (this may take a while) ...");
        pb
    };
    let client = reqwest::Client::new();
    // wasm doesn't require it to be mutable
    #[allow(unused_mut)]
    let mut resp = client.get(uri).body(vec![]).send().await?;
    let mut buf = vec![];
    while let Some(chunk) = resp.chunk().await? {
        buf.extend(chunk.to_vec());
    }

    pb.finish_with_message("SRS downloaded.");
    Ok(std::mem::take(&mut buf))
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) async fn get_srs_cmd(
    srs_path: PathBuf,
    settings_path: Option<PathBuf>,
    logrows: Option<u32>,
    check_mode: CheckMode,
) -> Result<(), Box<dyn Error>> {
    let k = if let Some(settings_p) = settings_path {
        if settings_p.exists() {
            let settings = GraphSettings::load(&settings_p)?;
            settings.run_args.logrows
        } else {
            let err_string = format!(
                "You will need to provide a valid settings file to use the settings option. You should run gen-settings to generate a settings file (and calibrate-settings to pick optimal logrows)."
            );
            return Err(err_string.into());
        }
    } else if let Some(k) = logrows {
        k
    } else {
        let err_string = format!(
            "You will need to provide a settings file or set the logrows. You should run gen-settings to generate a settings file (and calibrate-settings to pick optimal logrows)."
        );
        return Err(err_string.into());
    };

    let srs_uri = format!("{}{}", PUBLIC_SRS_URL, k);
    let mut reader = Cursor::new(fetch_srs(&srs_uri).await?);
    // check the SRS
    if matches!(check_mode, CheckMode::SAFE) {
        #[cfg(not(target_arch = "wasm32"))]
        let pb = init_spinner();
        #[cfg(not(target_arch = "wasm32"))]
        pb.set_message("Validating SRS (this may take a while) ...");
        ParamsKZG::<Bn256>::read(&mut reader)?;
        #[cfg(not(target_arch = "wasm32"))]
        pb.finish_with_message("SRS validated");
    }

    let mut file = std::fs::File::create(srs_path)?;
    file.write_all(reader.get_ref())?;

    info!("SRS downloaded");
    Ok(())
}

pub(crate) fn table(model: PathBuf, run_args: RunArgs) -> Result<(), Box<dyn Error>> {
    let model = Model::from_run_args(&run_args, &model)?;
    info!("\n {}", model.table_nodes());
    Ok(())
}

pub(crate) async fn gen_witness(
    compiled_circuit_path: PathBuf,
    data: PathBuf,
    output: Option<PathBuf>,
    vk_path: Option<PathBuf>,
    srs_path: Option<PathBuf>,
) -> Result<GraphWitness, Box<dyn Error>> {
    // these aren't real values so the sanity checks are mostly meaningless

    let mut circuit = GraphCircuit::load(compiled_circuit_path)?;
    let data = GraphData::from_path(data)?;
    let settings = circuit.settings().clone();

    let vk = if let Some(vk) = vk_path {
        Some(load_vk::<KZGCommitmentScheme<Bn256>, Fr, GraphCircuit>(
            vk,
            settings.clone(),
        )?)
    } else {
        None
    };

    let srs = if let Some(srs) = srs_path {
        Some(load_params_cmd(srs, settings.run_args.logrows)?)
    } else {
        None
    };

    #[cfg(not(target_arch = "wasm32"))]
    let mut input = circuit.load_graph_input(&data).await?;
    #[cfg(target_arch = "wasm32")]
    let mut input = circuit.load_graph_input(&data)?;

    let start_time = Instant::now();

    let witness = circuit.forward(&mut input, vk.as_ref(), srs.as_ref())?;

    // print each variable tuple (symbol, value) as symbol=value
    trace!(
        "witness generation {:?} took {:?}",
        circuit
            .settings()
            .run_args
            .variables
            .iter()
            .map(|v| { format!("{}={}", v.0, v.1) })
            .collect::<Vec<_>>(),
        start_time.elapsed()
    );

    if let Some(output_path) = output {
        serde_json::to_writer(&File::create(output_path)?, &witness)?;
    }
    Ok(witness)
}

/// Generate a circuit settings file
pub(crate) fn gen_circuit_settings(
    model_path: PathBuf,
    params_output: PathBuf,
    run_args: RunArgs,
) -> Result<(), Box<dyn Error>> {
    let circuit = GraphCircuit::from_run_args(&run_args, &model_path)?;
    let params = circuit.settings();
    params.save(&params_output).map_err(Box::<dyn Error>::from)
}

// not for wasm targets
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn init_spinner() -> ProgressBar {
    let pb = indicatif::ProgressBar::new_spinner();
    pb.set_draw_target(indicatif::ProgressDrawTarget::stdout());
    pb.enable_steady_tick(Duration::from_millis(200));
    pb.set_style(
        ProgressStyle::with_template("[{elapsed_precise}] {spinner:.blue} {msg}")
            .unwrap()
            .tick_strings(&[
                "------ - ✨ ",
                "------ - ⏳ ",
                "------ - 🌎 ",
                "------ - 🔎 ",
                "------ - 🥹 ",
                "------ - 🫠 ",
                "------ - 👾 ",
            ]),
    );
    pb
}

// not for wasm targets
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn init_bar(len: u64) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_draw_target(indicatif::ProgressDrawTarget::stdout());
    pb.enable_steady_tick(Duration::from_millis(200));
    let sty = ProgressStyle::with_template(
        "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
    )
    .unwrap()
    .progress_chars("##-");
    pb.set_style(sty);
    pb
}

#[cfg(not(target_arch = "wasm32"))]
use colored_json::ToColoredJson;

/// Calibrate the circuit parameters to a given a dataset
#[cfg(not(target_arch = "wasm32"))]
#[allow(trivial_casts)]
pub(crate) fn calibrate(
    model_path: PathBuf,
    data: PathBuf,
    settings_path: PathBuf,
    target: CalibrationTarget,
    scales: Option<Vec<crate::Scale>>,
    max_logrows: Option<u32>,
) -> Result<(), Box<dyn Error>> {
    let data = GraphData::from_path(data)?;
    // load the pre-generated settings
    let settings = GraphSettings::load(&settings_path)?;
    // now retrieve the run args
    // we load the model to get the input and output shapes
    // check if gag already exists

    #[cfg(unix)]
    let _r = match Gag::stdout() {
        Ok(r) => Some(r),
        Err(_) => None,
    };

    let model = Model::from_run_args(&settings.run_args, &model_path)?;
    // drop the gag
    #[cfg(unix)]
    std::mem::drop(_r);

    let range = if let Some(scales) = scales {
        scales
    } else {
        match target {
            CalibrationTarget::Resources { .. } => (8..10).collect::<Vec<crate::Scale>>(),
            CalibrationTarget::Accuracy => (10..14).collect::<Vec<crate::Scale>>(),
        }
    };

    let chunks = data.split_into_batches(model.graph.input_shapes()?)?;

    info!("num of calibration batches: {}", chunks.len());

    let mut found_params: Vec<GraphSettings> = vec![];

    let scale_rebase_multiplier = [1, 2, 10];

    // 2 x 2 grid
    let range_grid = range
        .iter()
        .cartesian_product(range.iter())
        .map(|(a, b)| (*a, *b))
        .collect::<Vec<(crate::Scale, crate::Scale)>>();

    // remove all entries where input_scale > param_scale
    let mut range_grid = range_grid
        .into_iter()
        .filter(|(a, b)| a <= b)
        .collect::<Vec<(crate::Scale, crate::Scale)>>();

    // if all integers
    let all_scale_0 = model
        .graph
        .get_input_types()?
        .iter()
        .all(|t| t.is_integer());
    if all_scale_0 {
        // set all a values to 0 then dedup
        range_grid = range_grid
            .iter()
            .map(|(_, b)| (0, *b))
            .sorted()
            .dedup()
            .collect::<Vec<(crate::Scale, crate::Scale)>>();
    }

    let range_grid = range_grid
        .iter()
        .cartesian_product(scale_rebase_multiplier.iter())
        .map(|(a, b)| (*a, *b))
        .collect::<Vec<((crate::Scale, crate::Scale), u32)>>();

    let pb = init_bar(range_grid.len() as u64);
    pb.set_message("calibrating...");

    for ((input_scale, param_scale), scale_rebase_multiplier) in range_grid {
        pb.set_message(format!(
            "input scale: {}, param scale: {}, scale rebase multiplier: {}",
            input_scale, param_scale, scale_rebase_multiplier
        ));
        // vec of settings copied chunks.len() times
        let run_args_iterable = vec![settings.run_args.clone(); chunks.len()];

        #[cfg(unix)]
        let _r = match Gag::stdout() {
            Ok(r) => Some(r),
            Err(_) => None,
        };
        #[cfg(unix)]
        let _q = match Gag::stderr() {
            Ok(r) => Some(r),
            Err(_) => None,
        };

        let tasks = chunks
            .iter()
            .zip(run_args_iterable)
            .map(|(chunk, run_args)| {
                // we need to create a new run args for each chunk
                // time it
                let chunk = chunk.clone();
                let local_run_args = RunArgs {
                    input_scale,
                    param_scale,
                    scale_rebase_multiplier,
                    ..run_args.clone()
                };

                let original_settings = settings.clone();

                let mut circuit = match GraphCircuit::from_run_args(&local_run_args, &model_path) {
                    Ok(c) => c,
                    Err(_) => {
                        return Err(format!("failed to create circuit from run args"))
                            as Result<GraphSettings, String>
                    }
                };

                let data = circuit
                    .load_graph_from_file_exclusively(&chunk)
                    .map_err(|e| format!("failed to load circuit inputs: {}", e))?;

                circuit
                    .calibrate(&data, max_logrows)
                    .map_err(|e| format!("failed to calibrate: {}", e))?;

                let settings = circuit.settings().clone();

                let found_run_args = RunArgs {
                    input_scale: settings.run_args.input_scale,
                    param_scale: settings.run_args.param_scale,
                    lookup_range: settings.run_args.lookup_range,
                    logrows: settings.run_args.logrows,
                    scale_rebase_multiplier: settings.run_args.scale_rebase_multiplier,
                    ..run_args.clone()
                };

                let found_settings = GraphSettings {
                    run_args: found_run_args,
                    required_lookups: settings.required_lookups,
                    model_output_scales: settings.model_output_scales,
                    model_input_scales: settings.model_input_scales,
                    num_rows: settings.num_rows,
                    total_assignments: settings.total_assignments,
                    total_const_size: settings.total_const_size,
                    ..original_settings.clone()
                };

                Ok(found_settings) as Result<GraphSettings, String>
            })
            .collect::<Vec<Result<GraphSettings, String>>>();

        let mut res: Vec<GraphSettings> = vec![];
        for task in tasks {
            if let Ok(task) = task {
                res.push(task);
            }
        }

        // drop the gag
        #[cfg(unix)]
        std::mem::drop(_r);
        #[cfg(unix)]
        std::mem::drop(_q);

        let max_lookup_range = res
            .iter()
            .map(|x| x.run_args.lookup_range.1)
            .max()
            .unwrap_or(0);
        let min_lookup_range = res
            .iter()
            .map(|x| x.run_args.lookup_range.0)
            .min()
            .unwrap_or(0);

        if let Some(mut best) = res.into_iter().max_by_key(|p| {
            (
                p.run_args.logrows,
                p.run_args.input_scale,
                p.run_args.param_scale,
            )
        }) {
            best.run_args.lookup_range = (min_lookup_range, max_lookup_range);
            // pick the one with the largest logrows
            found_params.push(best.clone());
            debug!(
                "found settings: \n {}",
                best.as_json()?.to_colored_json_auto()?
            );
        }

        pb.inc(1);
    }

    pb.finish_with_message("Calibration Done.");

    if found_params.is_empty() {
        return Err("calibration failed, could not find any suitable parameters given the calibration dataset".into());
    }

    debug!("Found {} sets of parameters", found_params.len());

    // now find the best params according to the target
    let mut best_params = match target {
        CalibrationTarget::Resources { .. } => {
            let mut param_iterator = found_params.iter().sorted_by_key(|p| p.run_args.logrows);

            let min_logrows = param_iterator
                .next()
                .ok_or("no params found")?
                .run_args
                .logrows;

            // pick the ones that have the minimum logrows but also the largest scale:
            // this is the best tradeoff between resource usage and accuracy
            found_params
                .iter()
                .filter(|p| p.run_args.logrows == min_logrows)
                .max_by_key(|p| {
                    (
                        p.run_args.input_scale,
                        p.run_args.param_scale,
                        // we want the largest rebase multiplier as it means we can use less constraints
                        p.run_args.scale_rebase_multiplier,
                    )
                })
                .ok_or("no params found")?
                .clone()
        }
        CalibrationTarget::Accuracy => {
            let param_iterator = found_params.iter().sorted_by_key(|p| {
                (
                    p.run_args.input_scale,
                    p.run_args.param_scale,
                    // we want the largest rebase multiplier as it means we can use less constraints
                    p.run_args.scale_rebase_multiplier,
                )
            });

            let last = param_iterator.last().ok_or("no params found")?;
            let max_scale = (
                last.run_args.input_scale,
                last.run_args.param_scale,
                last.run_args.scale_rebase_multiplier,
            );

            // pick the ones that have the max scale but also the smallest logrows:
            // this is the best tradeoff between resource usage and accuracy
            found_params
                .iter()
                .filter(|p| {
                    (
                        p.run_args.input_scale,
                        p.run_args.param_scale,
                        p.run_args.scale_rebase_multiplier,
                    ) == max_scale
                })
                .min_by_key(|p| p.run_args.logrows)
                .ok_or("no params found")?
                .clone()
        }
    };

    if matches!(target, CalibrationTarget::Resources { col_overflow: true }) {
        let lookup_log_rows = ((best_params.run_args.lookup_range.1
            - best_params.run_args.lookup_range.0) as f32)
            .log2()
            .ceil() as u32
            + 1;
        let mut reduction = std::cmp::max(
            (best_params
                .model_instance_shapes
                .iter()
                .map(|x| x.iter().product::<usize>())
                .sum::<usize>() as f32)
                .log2()
                .ceil() as u32
                + 1,
            lookup_log_rows,
        );
        reduction = std::cmp::max(reduction, crate::graph::MIN_LOGROWS);

        info!(
            "logrows > bits, shrinking logrows: {} -> {}",
            best_params.run_args.logrows, reduction
        );

        best_params.run_args.logrows = reduction;
    }

    best_params.save(&settings_path)?;

    debug!("Saved parameters.");

    Ok(())
}

pub(crate) fn mock(
    compiled_circuit_path: PathBuf,
    data_path: PathBuf,
) -> Result<(), Box<dyn Error>> {
    // mock should catch any issues by default so we set it to safe
    let mut circuit = GraphCircuit::load(compiled_circuit_path)?;

    let data = GraphWitness::from_path(data_path)?;

    circuit.load_graph_witness(&data)?;

    let public_inputs = circuit.prepare_public_inputs(&data)?;

    info!("Mock proof");

    let prover = halo2_proofs::dev::MockProver::run(
        circuit.settings().run_args.logrows,
        &circuit,
        vec![public_inputs],
    )
    .map_err(Box::<dyn Error>::from)?;
    prover
        .verify_par()
        .map_err(|e| Box::<dyn Error>::from(ExecutionError::VerifyError(e)))?;
    Ok(())
}

pub(crate) fn print_proof_hex(proof_path: PathBuf) -> Result<(), Box<dyn Error>> {
    let proof = Snark::load::<KZGCommitmentScheme<Bn256>>(&proof_path)?;
    for instance in proof.instances {
        println!("{:?}", instance);
    }
    info!("{}", hex::encode(proof.proof));
    Ok(())
}

#[cfg(feature = "render")]
pub(crate) fn render(model: PathBuf, output: PathBuf, args: RunArgs) -> Result<(), Box<dyn Error>> {
    let circuit = GraphCircuit::from_run_args(&args, &model)?;
    info!("Rendering circuit");

    // Create the area we want to draw on.
    // We could use SVGBackend if we want to render to .svg instead.
    // for an overview of how to interpret these plots, see https://zcash.github.io/halo2/user/dev-tools.html
    let root = BitMapBackend::new(&output, (512, 512)).into_drawing_area();
    root.fill(&TRANSPARENT)?;
    let root = root.titled("Layout", ("sans-serif", 20))?;

    halo2_proofs::dev::CircuitLayout::default()
        // We hide labels, else most circuits become impossible to decipher because of overlaid text
        .show_labels(false)
        .render(circuit.settings().run_args.logrows, &circuit, &root)?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn create_evm_verifier(
    vk_path: PathBuf,
    srs_path: PathBuf,
    settings_path: PathBuf,
    sol_code_path: PathBuf,
    abi_path: PathBuf,
) -> Result<(), Box<dyn Error>> {
    check_solc_requirement();
    let circuit_settings = GraphSettings::load(&settings_path)?;
    let params = load_params_cmd(srs_path, circuit_settings.run_args.logrows)?;

    let num_instance = circuit_settings.total_instances();
    let num_instance: usize = num_instance.iter().sum::<usize>();

    let vk = load_vk::<KZGCommitmentScheme<Bn256>, Fr, GraphCircuit>(vk_path, circuit_settings)?;
    trace!("params computed");

    let generator = halo2_solidity_verifier::SolidityGenerator::new(
        &params,
        &vk,
        halo2_solidity_verifier::BatchOpenScheme::Bdfg21,
        num_instance,
    );
    let verifier_solidity = generator.render()?;

    File::create(sol_code_path.clone())?.write_all(verifier_solidity.as_bytes())?;

    // fetch abi of the contract
    let (abi, _, _) = get_contract_artifacts(sol_code_path, "Halo2Verifier", 0)?;
    // save abi to file
    serde_json::to_writer(std::fs::File::create(abi_path)?, &abi)?;

    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn create_evm_data_attestation(
    vk_path: PathBuf,
    srs_path: PathBuf,
    settings_path: PathBuf,
    sol_code_path: PathBuf,
    abi_path: PathBuf,
    input: PathBuf,
) -> Result<(), Box<dyn Error>> {
    use crate::graph::{DataSource, VarVisibility};
    check_solc_requirement();

    let settings = GraphSettings::load(&settings_path)?;
    let params = load_params_cmd(srs_path, settings.run_args.logrows)?;

    let visibility = VarVisibility::from_args(&settings.run_args)?;

    let num_instance = settings.total_instances();
    let num_instance: usize = num_instance.iter().sum::<usize>();

    let vk = load_vk::<KZGCommitmentScheme<Bn256>, Fr, GraphCircuit>(vk_path, settings.clone())?;
    trace!("params computed");

    let yul_code: YulCode = gen_evm_verifier(&params, &vk, num_instance)?;

    let mut f = File::create(sol_code_path.clone())?;
    let _ = f.write(yul_code.as_bytes());

    let data = GraphData::from_path(input)?;

    let output_data = if let Some(DataSource::OnChain(source)) = data.output_data {
        if visibility.output.is_private() {
            return Err("private output data on chain is not supported on chain".into());
        }
        let mut on_chain_output_data = vec![];
        for call in source.calls {
            on_chain_output_data.push(call);
        }
        Some(on_chain_output_data)
    } else {
        None
    };

    let input_data = if let DataSource::OnChain(source) = data.input_data {
        if visibility.input.is_private() {
            return Err("private input data on chain is not supported on chain".into());
        }
        let mut on_chain_input_data = vec![];
        for call in source.calls {
            on_chain_input_data.push(call);
        }
        Some(on_chain_input_data)
    } else {
        None
    };

    if input_data.is_some() || output_data.is_some() {
        let output = fix_da_sol(input_data, output_data)?;
        let mut f = File::create(sol_code_path.clone())?;
        let _ = f.write(output.as_bytes());
        // fetch abi of the contract
        let (abi, _, _) = get_contract_artifacts(sol_code_path, "DataAttestation", 0)?;
        // save abi to file
        serde_json::to_writer(std::fs::File::create(abi_path)?, &abi)?;
    } else {
        return Err(
            "Neither input or output data source is on-chain. Atleast one must be on chain.".into(),
        );
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) async fn deploy_da_evm(
    data: PathBuf,
    settings_path: PathBuf,
    sol_code_path: PathBuf,
    rpc_url: Option<String>,
    addr_path: PathBuf,
    runs: usize,
    private_key: Option<String>,
) -> Result<(), Box<dyn Error>> {
    check_solc_requirement();
    let contract_address = deploy_da_verifier_via_solidity(
        settings_path,
        data,
        sol_code_path,
        rpc_url.as_deref(),
        runs,
        private_key.as_deref(),
    )
    .await?;
    info!("Contract deployed at: {}", contract_address);

    let mut f = File::create(addr_path)?;
    write!(f, "{:#?}", contract_address)?;

    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) async fn deploy_evm(
    sol_code_path: PathBuf,
    rpc_url: Option<String>,
    addr_path: PathBuf,
    runs: usize,
    private_key: Option<String>,
) -> Result<(), Box<dyn Error>> {
    check_solc_requirement();
    let contract_address = deploy_verifier_via_solidity(
        sol_code_path,
        rpc_url.as_deref(),
        runs,
        private_key.as_deref(),
    )
    .await?;

    info!("Contract deployed at: {:#?}", contract_address);

    let mut f = File::create(addr_path)?;
    write!(f, "{:#?}", contract_address)?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) async fn verify_evm(
    proof_path: PathBuf,
    addr_verifier: H160,
    rpc_url: Option<String>,
    addr_da: Option<H160>,
) -> Result<(), Box<dyn Error>> {
    use crate::eth::verify_proof_with_data_attestation;
    check_solc_requirement();

    let proof = Snark::load::<KZGCommitmentScheme<Bn256>>(&proof_path)?;

    let result = if let Some(addr_da) = addr_da {
        verify_proof_with_data_attestation(
            proof.clone(),
            addr_verifier,
            addr_da,
            rpc_url.as_deref(),
        )
        .await?
    } else {
        verify_proof_via_solidity(proof.clone(), addr_verifier, rpc_url.as_deref()).await?
    };

    info!("Solidity verification result: {}", result);

    if !result {
        return Err("Solidity verification failed".into());
    }

    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn create_evm_aggregate_verifier(
    vk_path: PathBuf,
    srs_path: PathBuf,
    sol_code_path: PathBuf,
    abi_path: PathBuf,
    circuit_settings: Vec<PathBuf>,
) -> Result<(), Box<dyn Error>> {
    check_solc_requirement();
    let params: ParamsKZG<Bn256> = load_srs::<KZGCommitmentScheme<Bn256>>(srs_path)?;

    let mut settings: Vec<GraphSettings> = vec![];

    for path in circuit_settings.iter() {
        let s = GraphSettings::load(path)?;
        settings.push(s);
    }

    let num_instance: usize = settings
        .iter()
        .map(|s| s.total_instances().iter().sum::<usize>())
        .sum();

    let num_instance = AggregationCircuit::num_instance(num_instance);
    assert_eq!(num_instance.len(), 1);
    let num_instance = num_instance[0];

    let agg_vk = load_vk::<KZGCommitmentScheme<Bn256>, Fr, AggregationCircuit>(vk_path, ())?;

    let mut generator = halo2_solidity_verifier::SolidityGenerator::new(
        &params,
        &agg_vk,
        halo2_solidity_verifier::BatchOpenScheme::Bdfg21,
        num_instance,
    );

    let acc_encoding = halo2_solidity_verifier::AccumulatorEncoding::new(
        0,
        AggregationCircuit::num_limbs(),
        AggregationCircuit::num_bits(),
    );

    generator = generator.set_acc_encoding(Some(acc_encoding));

    let verifier_solidity = generator.render()?;

    File::create(sol_code_path.clone())?.write_all(verifier_solidity.as_bytes())?;

    // fetch abi of the contract
    let (abi, _, _) = get_contract_artifacts(sol_code_path, "Halo2Verifier", 0)?;
    // save abi to file
    serde_json::to_writer(std::fs::File::create(abi_path)?, &abi)?;

    Ok(())
}

pub(crate) fn compile_circuit(
    model_path: PathBuf,
    compiled_circuit: PathBuf,
    settings_path: PathBuf,
) -> Result<(), Box<dyn Error>> {
    let settings = GraphSettings::load(&settings_path)?;
    let circuit = GraphCircuit::from_settings(&settings, &model_path, CheckMode::UNSAFE)?;
    circuit.save(compiled_circuit)?;
    Ok(())
}

pub(crate) fn setup(
    compiled_circuit: PathBuf,
    srs_path: PathBuf,
    vk_path: PathBuf,
    pk_path: PathBuf,
    witness: Option<PathBuf>,
) -> Result<(), Box<dyn Error>> {
    // these aren't real values so the sanity checks are mostly meaningless
    let mut circuit = GraphCircuit::load(compiled_circuit)?;
    if let Some(witness) = witness {
        let data = GraphWitness::from_path(witness)?;
        circuit.load_graph_witness(&data)?;
    }

    let params = load_params_cmd(srs_path, circuit.settings().run_args.logrows)?;

    let pk = create_keys::<KZGCommitmentScheme<Bn256>, Fr, GraphCircuit>(&circuit, &params)
        .map_err(Box::<dyn Error>::from)?;

    save_vk::<KZGCommitmentScheme<Bn256>>(&vk_path, pk.get_vk())?;
    save_pk::<KZGCommitmentScheme<Bn256>>(&pk_path, &pk)?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) async fn setup_test_evm_witness(
    data_path: PathBuf,
    compiled_circuit_path: PathBuf,
    test_data: PathBuf,
    rpc_url: Option<String>,
    input_source: TestDataSource,
    output_source: TestDataSource,
) -> Result<(), Box<dyn Error>> {
    use crate::graph::TestOnChainData;

    info!("run this command in background to keep the instance running for testing");
    let mut data = GraphData::from_path(data_path)?;
    let mut circuit = GraphCircuit::load(compiled_circuit_path)?;

    // if both input and output are from files fail
    if matches!(input_source, TestDataSource::File) && matches!(output_source, TestDataSource::File)
    {
        return Err("Both input and output cannot be from files".into());
    }

    let test_on_chain_data = TestOnChainData {
        data: test_data.clone(),
        rpc: rpc_url,
        data_sources: TestSources {
            input: input_source,
            output: output_source,
        },
    };

    circuit
        .populate_on_chain_test_data(&mut data, test_on_chain_data)
        .await?;

    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
use crate::pfsys::ProofType;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) async fn test_update_account_calls(
    addr: H160,
    data: PathBuf,
    rpc_url: Option<String>,
) -> Result<(), Box<dyn Error>> {
    use crate::eth::update_account_calls;

    check_solc_requirement();
    update_account_calls(addr, data, rpc_url.as_deref()).await?;

    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(clippy::too_many_arguments)]
pub(crate) fn prove(
    data_path: PathBuf,
    compiled_circuit_path: PathBuf,
    pk_path: PathBuf,
    proof_path: Option<PathBuf>,
    srs_path: PathBuf,
    proof_type: ProofType,
    check_mode: CheckMode,
) -> Result<Snark<Fr, G1Affine>, Box<dyn Error>> {
    use crate::pfsys::ProofSplitCommit;

    let data = GraphWitness::from_path(data_path)?;
    let mut circuit = GraphCircuit::load(compiled_circuit_path)?;

    circuit.load_graph_witness(&data)?;

    let public_inputs = circuit.prepare_public_inputs(&data)?;

    let circuit_settings = circuit.settings().clone();

    let params = load_params_cmd(srs_path, circuit_settings.run_args.logrows)?;

    let pk = load_pk::<KZGCommitmentScheme<Bn256>, Fr, GraphCircuit>(pk_path, circuit_settings)
        .map_err(Box::<dyn Error>::from)?;

    trace!("params computed");

    let strategy: StrategyType = proof_type.into();
    let transcript: TranscriptType = proof_type.into();
    let proof_split_commits: Option<ProofSplitCommit> = data.into();

    // creates and verifies the proof
    let snark = match strategy {
        StrategyType::Single => {
            let strategy = KZGSingleStrategy::new(&params);
            create_proof_circuit_kzg(
                circuit,
                &params,
                Some(public_inputs),
                &pk,
                transcript,
                strategy,
                check_mode,
                proof_split_commits,
            )?
        }
        StrategyType::Accum => {
            let strategy = AccumulatorStrategy::new(&params);
            create_proof_circuit_kzg(
                circuit,
                &params,
                Some(public_inputs),
                &pk,
                transcript,
                strategy,
                check_mode,
                proof_split_commits,
            )?
        }
    };

    if let Some(proof_path) = proof_path {
        snark.save(&proof_path)?;
    }

    Ok(snark)
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn fuzz(
    compiled_circuit_path: PathBuf,
    data_path: PathBuf,
    transcript: TranscriptType,
    num_runs: usize,
) -> Result<(), Box<dyn Error>> {
    check_solc_requirement();
    let passed = AtomicBool::new(true);

    // these aren't real values so the sanity checks are mostly meaningless
    let mut circuit = GraphCircuit::load(compiled_circuit_path)?;
    let logrows = circuit.settings().run_args.logrows;

    info!("setting up tests");

    let _r = Gag::stdout()?;
    let params = gen_srs::<KZGCommitmentScheme<Bn256>>(logrows);

    let data = GraphWitness::from_path(data_path)?;

    let pk = create_keys::<KZGCommitmentScheme<Bn256>, Fr, GraphCircuit>(&circuit, &params)
        .map_err(Box::<dyn Error>::from)?;

    circuit.load_graph_witness(&data)?;

    let public_inputs = circuit.prepare_public_inputs(&data)?;

    let strategy = KZGSingleStrategy::new(&params);
    std::mem::drop(_r);

    info!("starting fuzzing");

    info!("fuzzing pk");

    let fuzz_pk = || {
        let new_params = gen_srs::<KZGCommitmentScheme<Bn256>>(logrows);

        let bad_pk =
            create_keys::<KZGCommitmentScheme<Bn256>, Fr, GraphCircuit>(&circuit, &new_params)
                .map_err(|_| ())?;

        let bad_proof = create_proof_circuit_kzg(
            circuit.clone(),
            &params,
            Some(public_inputs.clone()),
            &bad_pk,
            transcript,
            strategy.clone(),
            CheckMode::UNSAFE,
            None,
        )
        .map_err(|_| ())?;

        verify_proof_circuit_kzg(
            params.verifier_params(),
            bad_proof,
            pk.get_vk(),
            strategy.clone(),
        )
        .map_err(|_| ())
    };

    run_fuzz_fn(num_runs, fuzz_pk, &passed);

    info!("fuzzing public inputs");

    let fuzz_public_inputs = || {
        let bad_inputs: Vec<Fr> = (0..public_inputs.len())
            .map(|_| Fr::random(rand::rngs::OsRng))
            .collect();

        let bad_proof = create_proof_circuit_kzg(
            circuit.clone(),
            &params,
            Some(bad_inputs.clone()),
            &pk,
            transcript,
            strategy.clone(),
            CheckMode::UNSAFE,
            None,
        )
        .map_err(|_| ())?;

        verify_proof_circuit_kzg(
            params.verifier_params(),
            bad_proof,
            pk.get_vk(),
            strategy.clone(),
        )
        .map_err(|_| ())
    };

    run_fuzz_fn(num_runs, fuzz_public_inputs, &passed);

    info!("fuzzing vk");

    let proof = create_proof_circuit_kzg(
        circuit.clone(),
        &params,
        Some(public_inputs.clone()),
        &pk,
        transcript,
        strategy.clone(),
        CheckMode::SAFE,
        None,
    )?;

    let fuzz_vk = || {
        let new_params = gen_srs::<KZGCommitmentScheme<Bn256>>(logrows);

        let bad_pk =
            create_keys::<KZGCommitmentScheme<Bn256>, Fr, GraphCircuit>(&circuit, &new_params)
                .map_err(|_| ())?;

        let bad_vk = bad_pk.get_vk();

        verify_proof_circuit_kzg(
            params.verifier_params(),
            proof.clone(),
            bad_vk,
            strategy.clone(),
        )
        .map_err(|_| ())
    };

    run_fuzz_fn(num_runs, fuzz_vk, &passed);

    info!("fuzzing proof bytes");

    let fuzz_proof_bytes = || {
        let mut rng = rand::thread_rng();

        let bad_proof_bytes: Vec<u8> = (0..proof.proof.len())
            .map(|_| rng.gen_range(0..20))
            .collect();

        let bad_proof = Snark::<_, _> {
            instances: proof.instances.clone(),
            proof: bad_proof_bytes,
            protocol: proof.protocol.clone(),
            transcript_type: transcript,
            split: None,
        };

        verify_proof_circuit_kzg(
            params.verifier_params(),
            bad_proof,
            pk.get_vk(),
            strategy.clone(),
        )
        .map_err(|_| ())
    };

    run_fuzz_fn(num_runs, fuzz_proof_bytes, &passed);

    info!("fuzzing proof instances");

    let fuzz_proof_instances = || {
        let mut bad_inputs = vec![vec![]];

        for l in &proof.instances {
            bad_inputs.push(
                (0..l.len())
                    .map(|_| Fr::random(rand::rngs::OsRng))
                    .collect(),
            );
        }

        let bad_proof = Snark::<_, _> {
            instances: bad_inputs.clone(),
            proof: proof.proof.clone(),
            protocol: proof.protocol.clone(),
            transcript_type: transcript,
            split: None,
        };

        verify_proof_circuit_kzg(
            params.verifier_params(),
            bad_proof,
            pk.get_vk(),
            strategy.clone(),
        )
        .map_err(|_| ())
    };

    run_fuzz_fn(num_runs, fuzz_proof_instances, &passed);

    if !passed.into_inner() {
        Err("fuzzing failed".into())
    } else {
        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn run_fuzz_fn(
    num_runs: usize,
    f: impl Fn() -> Result<(), ()> + std::marker::Sync + std::marker::Send,
    passed: &AtomicBool,
) {
    let num_failures = AtomicI64::new(0);
    let _r = Gag::stdout().unwrap();

    let pb = init_bar(num_runs as u64);
    pb.set_message("fuzzing...");
    (0..num_runs).into_par_iter().for_each(|_| {
        let result = f();
        if result.is_ok() {
            passed.swap(false, Ordering::Relaxed);
            num_failures.fetch_add(1, Ordering::Relaxed);
        }
        pb.inc(1);
    });
    pb.finish_with_message("Done.");
    std::mem::drop(_r);
    info!(
        "num failures: {} out of {}",
        num_failures.load(Ordering::Relaxed),
        num_runs
    );
}

pub(crate) fn swap_proof_commitments(
    proof_path: PathBuf,
    witness: PathBuf,
) -> Result<(), Box<dyn Error>> {
    let snark = Snark::load::<KZGCommitmentScheme<Bn256>>(&proof_path)?;
    let witness = GraphWitness::from_path(witness)?;
    let commitments = witness.get_kzg_commitments();

    if commitments.is_empty() {
        log::warn!("no commitments found in witness");
    }

    let snark_new = swap_proof_commitments_kzg(&snark, &commitments)?;

    if snark_new.proof != *snark.proof {
        log::warn!("swap proof has created a different proof");
    }

    snark_new.save(&proof_path)?;
    Ok(())
}

pub(crate) fn mock_aggregate(
    aggregation_snarks: Vec<PathBuf>,
    logrows: u32,
    split_proofs: bool,
) -> Result<(), Box<dyn Error>> {
    let mut snarks = vec![];
    for proof_path in aggregation_snarks.iter() {
        snarks.push(Snark::load::<KZGCommitmentScheme<Bn256>>(proof_path)?);
    }
    // proof aggregation
    #[cfg(not(target_arch = "wasm32"))]
    let pb = {
        let pb = init_spinner();
        pb.set_message("Aggregating (may take a while)...");
        pb
    };

    let circuit = AggregationCircuit::new(&G1Affine::generator().into(), snarks, split_proofs)?;

    let prover = halo2_proofs::dev::MockProver::run(logrows, &circuit, vec![circuit.instances()])
        .map_err(Box::<dyn Error>::from)?;
    prover
        .verify_par()
        .map_err(|e| Box::<dyn Error>::from(ExecutionError::VerifyError(e)))?;
    #[cfg(not(target_arch = "wasm32"))]
    pb.finish_with_message("Done.");
    Ok(())
}

pub(crate) fn setup_aggregate(
    sample_snarks: Vec<PathBuf>,
    vk_path: PathBuf,
    pk_path: PathBuf,
    srs_path: PathBuf,
    logrows: u32,
    split_proofs: bool,
) -> Result<(), Box<dyn Error>> {
    // the K used for the aggregation circuit
    let params = load_params_cmd(srs_path, logrows)?;

    let mut snarks = vec![];
    for proof_path in sample_snarks.iter() {
        snarks.push(Snark::load::<KZGCommitmentScheme<Bn256>>(proof_path)?);
    }

    let agg_circuit = AggregationCircuit::new(&params.get_g()[0].into(), snarks, split_proofs)?;
    let agg_pk =
        create_keys::<KZGCommitmentScheme<Bn256>, Fr, AggregationCircuit>(&agg_circuit, &params)?;

    let agg_vk = agg_pk.get_vk();

    // now save
    save_vk::<KZGCommitmentScheme<Bn256>>(&vk_path, agg_vk)?;
    save_pk::<KZGCommitmentScheme<Bn256>>(&pk_path, &agg_pk)?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn aggregate(
    proof_path: PathBuf,
    aggregation_snarks: Vec<PathBuf>,
    pk_path: PathBuf,
    srs_path: PathBuf,
    transcript: TranscriptType,
    logrows: u32,
    check_mode: CheckMode,
    split_proofs: bool,
) -> Result<(), Box<dyn Error>> {
    // the K used for the aggregation circuit
    let params = load_params_cmd(srs_path, logrows)?;

    let mut snarks = vec![];
    for proof_path in aggregation_snarks.iter() {
        snarks.push(Snark::load::<KZGCommitmentScheme<Bn256>>(proof_path)?);
    }

    let agg_pk = load_pk::<KZGCommitmentScheme<Bn256>, Fr, AggregationCircuit>(pk_path, ())?;
    // proof aggregation
    #[cfg(not(target_arch = "wasm32"))]
    let pb = {
        let pb = init_spinner();
        pb.set_message("Aggregating (may take a while)...");
        pb
    };

    {
        let agg_circuit = AggregationCircuit::new(&params.get_g()[0].into(), snarks, split_proofs)?;

        let now = Instant::now();
        let snark = create_proof_circuit_kzg(
            agg_circuit.clone(),
            &params,
            Some(agg_circuit.instances()),
            &agg_pk,
            transcript,
            AccumulatorStrategy::new(&params),
            check_mode,
            None,
        )?;

        let elapsed = now.elapsed();
        info!(
            "Aggregation proof took {}.{}",
            elapsed.as_secs(),
            elapsed.subsec_millis()
        );
        snark.save(&proof_path)?;
    }
    #[cfg(not(target_arch = "wasm32"))]
    pb.finish_with_message("Done.");

    Ok(())
}

pub(crate) fn verify(
    proof_path: PathBuf,
    settings_path: PathBuf,
    vk_path: PathBuf,
    srs_path: PathBuf,
) -> Result<(), Box<dyn Error>> {
    let circuit_settings = GraphSettings::load(&settings_path)?;
    let params = load_params_cmd(srs_path, circuit_settings.run_args.logrows)?;
    let proof = Snark::load::<KZGCommitmentScheme<Bn256>>(&proof_path)?;

    let strategy = KZGSingleStrategy::new(params.verifier_params());
    let vk = load_vk::<KZGCommitmentScheme<Bn256>, Fr, GraphCircuit>(vk_path, circuit_settings)?;
    let now = Instant::now();
    let result = verify_proof_circuit_kzg(params.verifier_params(), proof, &vk, strategy);
    let elapsed = now.elapsed();
    info!(
        "verify took {}.{}",
        elapsed.as_secs(),
        elapsed.subsec_millis()
    );
    info!("verified: {}", result.is_ok());
    result.map_err(|e| e.into())
}

pub(crate) fn verify_aggr(
    proof_path: PathBuf,
    vk_path: PathBuf,
    srs_path: PathBuf,
    logrows: u32,
) -> Result<(), Box<dyn Error>> {
    let params = load_params_cmd(srs_path, logrows)?;

    let proof = Snark::load::<KZGCommitmentScheme<Bn256>>(&proof_path)?;

    let strategy = AccumulatorStrategy::new(params.verifier_params());
    let vk = load_vk::<KZGCommitmentScheme<Bn256>, Fr, AggregationCircuit>(vk_path, ())?;
    let now = Instant::now();
    let result = verify_proof_circuit_kzg(&params, proof, &vk, strategy);

    let elapsed = now.elapsed();
    info!(
        "verify took {}.{}",
        elapsed.as_secs(),
        elapsed.subsec_millis()
    );
    info!("verified: {}", result.is_ok());
    result?;
    Ok(())
}

/// helper function to handle graphql errors
async fn parse_response(
    response: reqwest::Response,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    // Check if the response status is success
    if !response.status().is_success() {
        let status = response.status();
        let error_message = format!("Request failed with status code: {}", status);
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            error_message,
        )));
    }

    let response_body = response.json::<serde_json::Value>().await?;

    // Check if 'data' is null and 'errors' are present
    if response_body.get("data").is_none() || response_body.get("data").unwrap().is_null() {
        if let Some(errors) = response_body.get("errors") {
            let error_messages: Vec<String> = errors
                .as_array()
                .unwrap()
                .iter()
                .map(|error| error["message"].as_str().unwrap_or_default().to_string())
                .collect();

            let custom_error_message = format!("An error occurred: {}", error_messages.join(", "));
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                custom_error_message,
            )));
        } else {
            let error_message =
                "An error occurred: Response contains null data but no error details.";
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                error_message,
            )));
        }
    }

    Ok(response_body)
}

/// Retrieves the user's credentials from the hub
pub(crate) async fn get_hub_credentials(
    api_key: Option<&str>,
    url: Option<&str>,
    username: &str,
) -> Result<crate::hub::Organizations, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let request_body = serde_json::json!({
        "query": r#"
            query GetOrganizationId($username: String!) {
                organizations(name: $username) {
                    id
                    name
                }
            }
        "#,
        "variables": {
            "username": username,
        }
    });
    let url = url.unwrap_or("https://hub-staging.ezkl.xyz/graphql");
    let api_key = api_key.unwrap_or("ed896983-2ec3-4aaf-afa7-f01299f3d61f");

    let response = client
        .post(url)
        .header("API-Key", format!("{}", api_key))
        .json(&request_body)
        .send()
        .await?;

    // Using the parse_response helper function
    let response_body = parse_response(response).await?;

    // Extracting the organizations data
    let organizations: crate::hub::Organizations =
        serde_json::from_value(response_body["data"].clone())?;

    log::info!(
        "Organization ID : {}",
        organizations.as_json()?.to_colored_json_auto()?
    );
    Ok(organizations)
}

/// Deploy a model
pub(crate) async fn deploy_model(
    api_key: Option<&str>,
    url: Option<&str>,
    model: &Path,
    input: &Path,
    name: &str,
    organization_id: &str,
    args: &RunArgs,
    target: &CalibrationTarget,
) -> Result<crate::hub::Artifact, Box<dyn Error>> {
    let model_file = tokio::fs::File::open(model.canonicalize()?).await?;
    // read file body stream
    let stream = FramedRead::new(model_file, BytesCodec::new());
    let model_file_body = reqwest::Body::wrap_stream(stream);

    let model_file = reqwest::multipart::Part::stream(model_file_body).file_name("uncompiledModel");

    let input_file = tokio::fs::File::open(input.canonicalize()?).await?;
    // read file body stream
    let stream = FramedRead::new(input_file, BytesCodec::new());
    let input_file_body = reqwest::Body::wrap_stream(stream);

    //make form part of file
    let input_file = reqwest::multipart::Part::stream(input_file_body).file_name("input");

    // the graphql request map
    let map = r#"{
            "uncompiledModel": [
                "variables.uncompiledModel"
            ],
            "input": [
                "variables.input"
            ]
        }"#;

    let operations = serde_json::json!({
        "query": "mutation($uncompiledModel: Upload!, $input: Upload!, $organizationId: String!, $name: String!, $calibrationTarget: String!, $tolerance: Float!, $inputVisibility: String!, $outputVisibility: String!, $paramVisibility: String!) {
                generateArtifact(
                    name: $name,
                    description: $name,
                    uncompiledModel: $uncompiledModel,
                    input: $input,
                    organizationId: $organizationId, 
                    calibrationTarget: $calibrationTarget, 
                    tolerance: $tolerance, 
                    inputVisibility: $inputVisibility,
                    outputVisibility: $outputVisibility,
                    paramVisibility: $paramVisibility,
                ) {
                    id
                    name
                    status
                    errors
                }
            }",
        "variables": {
            "name": name,
            "uncompiledModel": null,
            "input": null,
            "organizationId": organization_id,
            "calibrationTarget": target.to_string(),
            "tolerance": args.tolerance.val,
            "inputVisibility": args.input_visibility.to_string(),
            "outputVisibility": args.output_visibility.to_string(),
            "paramVisibility": args.param_visibility.to_string(),
        }
    })
    .to_string();

    // now the form data
    let mut form = reqwest::multipart::Form::new();
    form = form
        .text("operations", operations)
        .text("map", map)
        .part("uncompiledModel", model_file)
        .part("input", input_file);

    let client = reqwest::Client::new();
    let url = url.unwrap_or("https://hub-staging.ezkl.xyz/graphql");
    let api_key = api_key.unwrap_or("ed896983-2ec3-4aaf-afa7-f01299f3d61f");
    //send request
    let response = client
        .post(url)
        .header("API-Key", format!("{}", api_key))
        .multipart(form)
        .send()
        .await?;
    let response_body = parse_response(response).await?;
    let artifact_data: crate::hub::Artifact =
        serde_json::from_value(response_body["data"]["generateArtifact"].clone())?;
    log::info!(
        "Artifact Data : {}",
        artifact_data.as_json()?.to_colored_json_auto()?
    );
    Ok(artifact_data)
}

/// Get the artifact from the hub
pub(crate) async fn get_deployed_model(
    api_key: Option<&str>,
    url: Option<&str>,
    id: &str,
) -> Result<crate::hub::Artifact, Box<dyn Error>> {
    let query = serde_json::json!({
        "query": "query getArtifact($id: String!){
            artifact(id: $id) {
                id
                name
                status
                errors
            }
        }",
        "variables": {
            "id": id,
        }
    });
    let client = reqwest::Client::new();
    let url = url.unwrap_or("https://hub-staging.ezkl.xyz/graphql");
    let api_key = api_key.unwrap_or("ed896983-2ec3-4aaf-afa7-f01299f3d61f");
    //send request
    let response = client
        .post(url)
        .header("API-Key", format!("{}", api_key))
        .json(&query)
        .send()
        .await?;
    let response_body = parse_response(response).await?;
    let artifact_data: crate::hub::Artifact =
        serde_json::from_value(response_body["data"]["artifact"].clone())?;
    log::info!(
        "Artifact Data : {}",
        artifact_data.as_json()?.to_colored_json_auto()?
    );
    Ok(artifact_data)
}

/// Generates proofs on the hub
pub async fn prove_hub(
    api_key: Option<&str>,
    url: Option<&str>,
    id: &str,
    input: &Path,
) -> Result<crate::hub::Proof, Box<dyn std::error::Error>> {
    let input_file = tokio::fs::File::open(input.canonicalize()?).await?;
    let stream = FramedRead::new(input_file, BytesCodec::new());
    let input_file_body = reqwest::Body::wrap_stream(stream);

    let input_file = reqwest::multipart::Part::stream(input_file_body).file_name("input");

    let map = r#"{
        "input": [
            "variables.input"
        ]
    }"#;

    let operations = serde_json::json!({
        "query": r#"
            mutation($input: Upload!, $id: String!) {
                initiateProof(input: $input, id: $id) {
                    id
                }
            }
        "#,
        "variables": {
            "input": null,
            "id": id,
        }
    })
    .to_string();

    let mut form = reqwest::multipart::Form::new();
    form = form
        .text("operations", operations)
        .text("map", map)
        .part("input", input_file);
    let url = url.unwrap_or("https://hub-staging.ezkl.xyz/graphql");
    let api_key = api_key.unwrap_or("ed896983-2ec3-4aaf-afa7-f01299f3d61f");
    let client = reqwest::Client::new();
    let response = client
        .post(url)
        .header("API-Key", format!("{}", api_key))
        .multipart(form)
        .send()
        .await?;

    let response_body = parse_response(response).await?;

    // Check if 'data' is null and 'errors' are present
    if response_body.get("data").is_none() || response_body.get("data").unwrap().is_null() {
        if let Some(errors) = response_body.get("errors") {
            let error_messages: Vec<String> = errors
                .as_array()
                .unwrap()
                .iter()
                .map(|error| error["message"].as_str().unwrap_or_default().to_string())
                .collect();

            let custom_error_message = format!("An error occurred: {}", error_messages.join(", "));
            log::error!("{}", custom_error_message);
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                custom_error_message,
            )));
        } else {
            let error_message =
                "An error occurred: Response contains null data but no error details.";
            log::error!("{}", error_message);
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                error_message,
            )));
        }
    }
    let proof_id: crate::hub::Proof =
        serde_json::from_value(response_body["data"]["initiateProof"].clone())?;
    log::info!("Proof ID : {}", proof_id.as_json()?.to_colored_json_auto()?);
    Ok(proof_id)
}

/// Fetches proofs from the hub
pub(crate) async fn get_hub_proof(
    api_key: Option<&str>,
    url: Option<&str>,
    id: &str,
) -> Result<crate::hub::Proof, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let request_body = serde_json::json!({
        "query": format!(r#"
            query {{
                getProof(id: "{}") {{
                    id
                    artifact {{ id name }}
                    status
                    proof
                    instances
                    transcriptType
                }}
            }}
        "#, id),
    });
    let url = url.unwrap_or("https://hub-staging.ezkl.xyz/graphql");
    let api_key = api_key.unwrap_or("ed896983-2ec3-4aaf-afa7-f01299f3d61f");

    let response = client
        .post(url)
        .header("API-Key", format!("{:?}", api_key))
        .json(&request_body)
        .send()
        .await?;
    let response_body = parse_response(response).await?;

    let proof: crate::hub::Proof =
        serde_json::from_value(response_body["data"]["getProof"].clone())?;

    log::info!("Proof : {}", proof.as_json()?.to_colored_json_auto()?);
    Ok(proof)
}

/// helper function for load_params
pub(crate) fn load_params_cmd(
    srs_path: PathBuf,
    logrows: u32,
) -> Result<ParamsKZG<Bn256>, Box<dyn Error>> {
    let mut params: ParamsKZG<Bn256> = load_srs::<KZGCommitmentScheme<Bn256>>(srs_path)?;
    info!("downsizing params to {} logrows", logrows);
    if logrows < params.k() {
        params.downsize(logrows);
    }
    Ok(params)
}
