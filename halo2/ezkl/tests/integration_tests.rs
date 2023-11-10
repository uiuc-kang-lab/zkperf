#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod native_tests {

    use core::panic;
    // use ezkl::circuit::table::RESERVED_BLINDING_ROWS_PAD;
    use ezkl::graph::input::{FileSource, FileSourceInner, GraphData};
    use ezkl::graph::{DataSource, GraphSettings, GraphWitness, Visibility};
    use lazy_static::lazy_static;
    use rand::Rng;
    use std::env::var;
    use std::io::{Read, Write};
    use std::process::{Child, Command};
    use std::sync::Once;
    static COMPILE: Once = Once::new();
    #[allow(dead_code)]
    static COMPILE_WASM: Once = Once::new();
    static ENV_SETUP: Once = Once::new();

    //Sure to run this once

    lazy_static! {
        static ref CARGO_TARGET_DIR: String =
            var("CARGO_TARGET_DIR").unwrap_or_else(|_| "./target".to_string());
        static ref ANVIL_URL: String = "http://localhost:3030".to_string();
        static ref LIMITLESS_ANVIL_URL: String = "http://localhost:8545".to_string();
        static ref ANVIL_DEFAULT_PRIVATE_KEY: String =
            "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string();
    }

    fn start_anvil(limitless: bool) -> Child {
        let mut args = vec!["-p"];
        if limitless {
            args.push("8545");
            args.push("--code-size-limit=41943040");
            args.push("--disable-block-gas-limit");
        } else {
            args.push("3030");
        }
        let child = Command::new("anvil")
            .args(args)
            // .stdout(Stdio::piped())
            .spawn()
            .expect("failed to start anvil process");

        std::thread::sleep(std::time::Duration::from_secs(3));
        child
    }

    fn init_binary() {
        COMPILE.call_once(|| {
            println!("using cargo target dir: {}", *CARGO_TARGET_DIR);
            build_ezkl();
        });
    }

    ///
    #[allow(dead_code)]
    pub fn init_wasm() {
        COMPILE_WASM.call_once(|| {
            build_wasm_ezkl();
        });
    }

    fn setup_py_env() {
        ENV_SETUP.call_once(|| {
            // supposes that you have a virtualenv called .env and have run the following
            // equivalent of python -m venv .env
            // source .env/bin/activate
            // pip install -r requirements.txt
            // maturin develop --release --features python-bindings

            // now install torch, pandas, numpy, seaborn, jupyter
            let status = Command::new("pip")
                .args(["install", "numpy", "onnxruntime", "onnx"])
                .stdout(std::process::Stdio::null())
                .status()
                .expect("failed to execute process");

            assert!(status.success());
        });
    }

    fn download_srs(test_dir: &str, logrows: u32) -> String {
        let srs_path = format!("{}/kzg{}.srs", test_dir, logrows);
        // if does not exist, download it
        if !std::path::Path::new(&srs_path).exists() {
            let status = Command::new("curl")
                .args([
                    "-o",
                    &format!(
                        "{}/kzg{}.srs",
                        test_dir,
                        logrows
                    ),
                    &format!(
                        "https://trusted-setup-halo2kzg.s3.eu-central-1.amazonaws.com/perpetual-powers-of-tau-raw-{}",
                        logrows
                    ),
                ])
                .status()
                .expect("failed to execute process");
            assert!(status.success());
        }
        srs_path
    }

    fn init_params(test_dir: &str, settings_path: std::path::PathBuf) -> String {
        println!("using settings path: {}", settings_path.to_str().unwrap());
        // read in settings json
        let settings =
            std::fs::read_to_string(settings_path).expect("failed to read settings file");
        // read in to GraphSettings object
        let settings: GraphSettings = serde_json::from_str(&settings).unwrap();
        let logrows = settings.run_args.logrows;

        download_srs(test_dir, logrows)
    }

    fn mv_test_(test_dir: &str, test: &str) {
        let path: std::path::PathBuf = format!("{}/{}", test_dir, test).into();
        if !path.exists() {
            let status = Command::new("cp")
                .args([
                    "-R",
                    &format!("./examples/onnx/{}", test),
                    &format!("{}/{}", test_dir, test),
                ])
                .status()
                .expect("failed to execute process");
            assert!(status.success());
        }
    }

    fn mk_data_batches_(test_dir: &str, test: &str, output_dir: &str, num_batches: usize) {
        let path: std::path::PathBuf = format!("{}/{}", test_dir, test).into();
        if !path.exists() {
            panic!("test_dir does not exist")
        } else {
            // copy the directory
            let status = Command::new("cp")
                .args([
                    "-R",
                    &format!("{}/{}", test_dir, test),
                    &format!("{}/{}", test_dir, output_dir),
                ])
                .status()
                .expect("failed to execute process");

            assert!(status.success());

            let data = GraphData::from_path(format!("{}/{}/input.json", test_dir, test).into())
                .expect("failed to load input data");

            let input_data = match data.input_data {
                DataSource::File(data) => data,
                _ => panic!("Only File data sources support batching"),
            };

            let duplicated_input_data: FileSource = input_data
                .iter()
                .map(|data| (0..num_batches).flat_map(|_| data.clone()).collect())
                .collect();

            let duplicated_data = GraphData::new(DataSource::File(duplicated_input_data));

            let res =
                duplicated_data.save(format!("{}/{}/input.json", test_dir, output_dir).into());

            assert!(res.is_ok());
        }
    }

    const PF_FAILURE: &str = "examples/test_failure.proof";

    const PF_FAILURE_AGGR: &str = "examples/test_failure_aggr.proof";

    const LARGE_TESTS: [&str; 5] = [
        "self_attention",
        "nanoGPT",
        "multihead_attention",
        "mobilenet",
        "mnist_gan",
    ];

    const ACCURACY_CAL_TESTS: [&str; 5] = [
        "accuracy",
        "1l_mlp",
        "4l_relu_conv_fc",
        "1l_elu",
        "1l_prelu",
    ];

    const TESTS: [&str; 64] = [
        "1l_mlp",
        "1l_slice",
        "1l_concat",
        "1l_flatten",
        // "1l_average",
        "1l_div",
        "1l_pad",
        "1l_reshape",
        "1l_eltwise_div",
        "1l_sigmoid",
        "1l_sqrt",
        "1l_softmax",
        // "1l_instance_norm",
        "1l_batch_norm",
        "1l_prelu",
        "1l_leakyrelu",
        "1l_gelu_noappx",
        // "1l_gelu_tanh_appx",
        "1l_relu",
        "1l_downsample",
        "1l_tanh",
        "2l_relu_sigmoid_small",
        "2l_relu_fc",
        "2l_relu_small",
        "2l_relu_sigmoid",
        "1l_conv",
        "2l_sigmoid_small",
        "2l_relu_sigmoid_conv",
        "3l_relu_conv_fc",
        "4l_relu_conv_fc",
        "1l_erf",
        "1l_var",
        "1l_elu", //30
        "min",
        "max",
        "1l_max_pool",
        "1l_conv_transpose",
        "1l_upsample", //35
        "1l_identity",
        "idolmodel",
        "trig",
        "prelu_gmm",
        "lstm", //40
        "rnn",
        "quantize_dequantize",
        "1l_where",
        "boolean",
        "boolean_identity",
        "decision_tree", // "variable_cnn",
        "random_forest",
        "gradient_boosted_trees",
        "1l_topk",
        "xgboost", //50
        "lightgbm",
        "hummingbird_decision_tree",
        "oh_decision_tree",
        "linear_svc",
        "gather_elements",
        "less",
        "xgboost_reg",
        "1l_powf",
        "scatter_elements",
        "1l_linear", //60
        "linear_regression",
        "sklearn_mlp",
        "1l_mean",
        "rounding_ops",
    ];

    const WASM_TESTS: [&str; 48] = [
        "1l_mlp",
        "1l_slice",
        "1l_concat",
        "1l_flatten",
        // "1l_average",
        "1l_div",
        "1l_pad",
        "1l_reshape",
        "1l_eltwise_div",
        "1l_sigmoid",
        "1l_sqrt",
        "1l_softmax",
        // "1l_instance_norm",
        "1l_batch_norm",
        "1l_prelu",
        "1l_leakyrelu",
        "1l_gelu_noappx",
        // "1l_gelu_tanh_appx",
        "1l_relu",
        "1l_downsample",
        "1l_tanh",
        "2l_relu_sigmoid_small",
        "2l_relu_fc",
        "2l_relu_small",
        "2l_relu_sigmoid",
        "1l_conv",
        "2l_sigmoid_small",
        "2l_relu_sigmoid_conv",
        "3l_relu_conv_fc",
        "4l_relu_conv_fc",
        "1l_erf",
        "1l_var",
        "1l_elu",
        "min",
        "max",
        "1l_max_pool",
        "1l_conv_transpose",
        "1l_upsample",
        "1l_identity",
        // "idolmodel",
        "trig",
        "prelu_gmm",
        "lstm",
        "rnn",
        "quantize_dequantize",
        "1l_where",
        "boolean",
        "boolean_identity",
        "decision_tree", // "variable_cnn",
        "random_forest",
        "gradient_boosted_trees",
        "1l_topk",
        // "xgboost",
        // "lightgbm",
        // "hummingbird_decision_tree",
    ];

    const TESTS_AGGR: [&str; 21] = [
        "1l_mlp",
        "1l_flatten",
        "1l_average",
        "1l_reshape",
        "1l_div",
        "1l_pad",
        "1l_sigmoid",
        "1l_gelu_noappx",
        "1l_sqrt",
        "1l_prelu",
        "1l_var",
        "1l_leakyrelu",
        "1l_relu",
        "1l_tanh",
        "2l_relu_fc",
        "2l_relu_sigmoid_small",
        "2l_relu_small",
        "1l_conv",
        "min",
        "max",
        "1l_max_pool",
    ];

    const NEG_TESTS: [(&str, &str); 2] = [
        ("2l_relu_sigmoid_small", "2l_relu_small"),
        ("2l_relu_small", "2l_relu_sigmoid_small"),
    ];

    const TESTS_EVM: [&str; 21] = [
        "1l_mlp",
        "1l_flatten",
        "1l_average",
        "1l_reshape",
        "1l_sigmoid",
        "1l_div",
        "1l_sqrt",
        "1l_prelu",
        "1l_var",
        "1l_leakyrelu",
        "1l_gelu_noappx",
        "1l_relu",
        "1l_tanh",
        "2l_relu_sigmoid_small",
        "2l_relu_small",
        "2l_relu_fc",
        "min",
        "max",
        "1l_max_pool",
        "idolmodel",
        "1l_identity",
    ];

    const TESTS_EVM_AGGR: [&str; 18] = [
        "1l_mlp",
        "1l_reshape",
        "1l_sigmoid",
        "1l_div",
        "1l_sqrt",
        "1l_prelu",
        "1l_var",
        "1l_leakyrelu",
        "1l_gelu_noappx",
        "1l_relu",
        "1l_tanh",
        "2l_relu_sigmoid_small",
        "2l_relu_small",
        "2l_relu_fc",
        "min",
        "max",
        "idolmodel",
        "1l_identity",
    ];

    const EXAMPLES: [&str; 2] = ["mlp_4d_einsum", "conv2d_mnist"];

    macro_rules! test_func_aggr {
    () => {
        #[cfg(test)]
        mod tests_aggr {
            use seq_macro::seq;
            use crate::native_tests::TESTS_AGGR;
            use test_case::test_case;
            use crate::native_tests::kzg_aggr_prove_and_verify;
            use crate::native_tests::kzg_aggr_mock_prove_and_verify;
            use tempdir::TempDir;

            seq!(N in 0..=20 {

            #(#[test_case(TESTS_AGGR[N])])*
            fn kzg_aggr_mock_prove_and_verify_(test: &str) {
                crate::native_tests::init_binary();
                let test_dir = TempDir::new(test).unwrap();
                let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                kzg_aggr_mock_prove_and_verify(path, test.to_string());
                test_dir.close().unwrap();
            }


            #(#[test_case(TESTS_AGGR[N])])*
            fn kzg_aggr_prove_and_verify_(test: &str) {
                crate::native_tests::init_binary();
                let test_dir = TempDir::new(test).unwrap();
                let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                kzg_aggr_prove_and_verify(path, test.to_string());
                test_dir.close().unwrap();
            }

            });
    }
    };
}

    macro_rules! test_func {
    () => {
        #[cfg(test)]
        mod tests {
            use seq_macro::seq;
            use crate::native_tests::TESTS;
            use crate::native_tests::WASM_TESTS;
            use crate::native_tests::ACCURACY_CAL_TESTS;
            use crate::native_tests::LARGE_TESTS;
            use test_case::test_case;
            use crate::native_tests::mock;
            use crate::native_tests::accuracy_measurement;
            use crate::native_tests::kzg_prove_and_verify;
            use crate::native_tests::run_js_tests;
            use crate::native_tests::kzg_fuzz;
            use crate::native_tests::render_circuit;
            use crate::native_tests::model_serialization;
            use crate::native_tests::model_serialization_different_binaries;
            use crate::native_tests::tutorial as run_tutorial;
            use tempdir::TempDir;

            #[test]
            fn model_serialization_different_binaries_() {
                let test = "1l_mlp";
                let test_dir = TempDir::new(test).unwrap();
                let path = test_dir.path().to_str().unwrap();
                crate::native_tests::mv_test_(path, test);
                // percent tolerance test
                model_serialization_different_binaries(path, test.to_string());
                test_dir.close().unwrap();
            }

            #[test]
            fn tutorial_() {
                let test_dir = TempDir::new("tutorial").unwrap();
                let path = test_dir.path().to_str().unwrap();
                crate::native_tests::mv_test_(path, "tutorial");
                // percent tolerance test
                run_tutorial(path, "1.0");
                test_dir.close().unwrap();
            }

            seq!(N in 0..=4 {
            #(#[test_case(ACCURACY_CAL_TESTS[N])])*
            fn mock_accuracy_cal_tests(test: &str) {
                crate::native_tests::init_binary();
                let test_dir = TempDir::new(test).unwrap();
                let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                mock(path, test.to_string(), "public", "public", "public", 1, "accuracy", None);
                test_dir.close().unwrap();
            }
        });




            seq!(N in 0..=63 {
            #(#[test_case(TESTS[N])])*
            fn model_serialization_(test: &str) {
                let test_dir = TempDir::new(test).unwrap();
                let path = test_dir.path().to_str().unwrap();
                crate::native_tests::mv_test_(path, test);
                // percent tolerance test
                model_serialization(path, test.to_string());
                test_dir.close().unwrap();
            }





            #(#[test_case(TESTS[N])])*
            #[ignore]
            fn render_circuit_(test: &str) {
                crate::native_tests::init_binary();
                let test_dir = TempDir::new(test).unwrap();
                let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                render_circuit(path, test.to_string());
                test_dir.close().unwrap();
            }

            #(#[test_case(TESTS[N])])*
            fn accuracy_measurement_public_outputs_(test: &str) {
                crate::native_tests::init_binary();
                crate::native_tests::setup_py_env();
                let test_dir = TempDir::new(test).unwrap();
                let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                accuracy_measurement(path, test.to_string(), "private", "private", "public", 1, "accuracy");
                test_dir.close().unwrap();
            }

            #(#[test_case(TESTS[N])])*
            fn accuracy_measurement_fixed_params_(test: &str) {
                crate::native_tests::init_binary();
                crate::native_tests::setup_py_env();
                let test_dir = TempDir::new(test).unwrap();
                let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                accuracy_measurement(path, test.to_string(), "private", "fixed", "private", 1, "accuracy");
                test_dir.close().unwrap();
            }

            #(#[test_case(TESTS[N])])*
            fn accuracy_measurement_public_inputs_(test: &str) {
                crate::native_tests::init_binary();
                crate::native_tests::setup_py_env();
                let test_dir = TempDir::new(test).unwrap();
                let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                accuracy_measurement(path, test.to_string(), "public", "private", "private", 1, "accuracy");
                test_dir.close().unwrap();
            }

            #(#[test_case(TESTS[N])])*
            fn mock_public_outputs_(test: &str) {
                crate::native_tests::init_binary();
                let test_dir = TempDir::new(test).unwrap();
                let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                mock(path, test.to_string(), "private", "private", "public", 1, "resources", None);
                test_dir.close().unwrap();
            }

            #(#[test_case(TESTS[N])])*
            fn mock_large_batch_public_outputs_(test: &str) {
                crate::native_tests::init_binary();
                let test_dir = TempDir::new(test).unwrap();
                let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                let large_batch_dir = &format!("large_batches_{}", test);
                crate::native_tests::mk_data_batches_(path, test, &large_batch_dir, 10);
                mock(path, large_batch_dir.to_string(), "private", "private", "public", 10, "resources", None);
                test_dir.close().unwrap();
            }

            #(#[test_case(TESTS[N])])*
            fn mock_public_inputs_(test: &str) {
                crate::native_tests::init_binary();
                let test_dir = TempDir::new(test).unwrap();
                let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                mock(path, test.to_string(), "public", "private", "private", 1, "resources", None);
                test_dir.close().unwrap();
            }

            #(#[test_case(TESTS[N])])*
            fn mock_fixed_inputs_(test: &str) {
                crate::native_tests::init_binary();
                let test_dir = TempDir::new(test).unwrap();
                let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                mock(path, test.to_string(), "fixed", "private", "private", 1, "resources", None);
                test_dir.close().unwrap();
            }

            #(#[test_case(TESTS[N])])*
            fn mock_fixed_outputs_(test: &str) {
                crate::native_tests::init_binary();
                let test_dir = TempDir::new(test).unwrap();
                let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                mock(path, test.to_string(), "private", "private", "fixed", 1, "resources", None);
                test_dir.close().unwrap();
            }

            #(#[test_case(TESTS[N])])*
            fn mock_fixed_params_(test: &str) {
                crate::native_tests::init_binary();
                let test_dir = TempDir::new(test).unwrap();
                let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                mock(path, test.to_string(), "private", "fixed", "private", 1, "resources", None);
                test_dir.close().unwrap();
            }

            #(#[test_case(TESTS[N])])*
            fn mock_hashed_input_(test: &str) {
                crate::native_tests::init_binary();
                let test_dir = TempDir::new(test).unwrap();
                let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                mock(path, test.to_string(), "hashed", "private", "public", 1, "resources", None);
                test_dir.close().unwrap();
            }

            #(#[test_case(TESTS[N])])*
            fn mock_kzg_input_(test: &str) {
                crate::native_tests::init_binary();
                let test_dir = TempDir::new(test).unwrap();
                let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                mock(path, test.to_string(), "kzgcommit", "private", "public", 1, "resources", None);
                test_dir.close().unwrap();
            }

            #(#[test_case(TESTS[N])])*
            fn mock_encrypted_input_(test: &str) {
                crate::native_tests::init_binary();
                let test_dir = TempDir::new(test).unwrap();
                let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                mock(path, test.to_string(), "encrypted", "private", "public", 1, "resources", None);
                test_dir.close().unwrap();
            }

            #(#[test_case(TESTS[N])])*
            fn mock_hashed_params_(test: &str) {
                crate::native_tests::init_binary();
                let test_dir = TempDir::new(test).unwrap();
                let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                mock(path, test.to_string(), "private", "hashed", "public", 1, "resources", None);
                test_dir.close().unwrap();
            }


            #(#[test_case(TESTS[N])])*
            fn mock_kzg_params_(test: &str) {
                crate::native_tests::init_binary();
                let test_dir = TempDir::new(test).unwrap();
                let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                mock(path, test.to_string(), "private", "kzgcommit", "public", 1, "resources", None);
                test_dir.close().unwrap();
            }

            #(#[test_case(TESTS[N])])*
            fn mock_encrypted_params_(test: &str) {
                crate::native_tests::init_binary();
                let test_dir = TempDir::new(test).unwrap();
                let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                mock(path, test.to_string(), "private", "hashed", "public", 1, "resources", None);
                test_dir.close().unwrap();
            }

            #(#[test_case(TESTS[N])])*
            fn mock_hashed_output_(test: &str) {
                crate::native_tests::init_binary();
                let test_dir = TempDir::new(test).unwrap();
                let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                mock(path, test.to_string(), "public", "private", "hashed", 1, "resources", None);
                test_dir.close().unwrap();
            }


            #(#[test_case(TESTS[N])])*
            fn mock_kzg_output_(test: &str) {
                crate::native_tests::init_binary();
                let test_dir = TempDir::new(test).unwrap();
                let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                mock(path, test.to_string(), "public", "private", "kzgcommit", 1, "resources", None);
                test_dir.close().unwrap();
            }

            #(#[test_case(TESTS[N])])*
            fn mock_hashed_output_fixed_params_(test: &str) {
                crate::native_tests::init_binary();
                let test_dir = TempDir::new(test).unwrap();
                let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                mock(path, test.to_string(), "public", "fixed", "hashed", 1, "resources", None);
                test_dir.close().unwrap();
            }


            #(#[test_case(TESTS[N])])*
            fn mock_hashed_output_kzg_params_(test: &str) {
                crate::native_tests::init_binary();
                let test_dir = TempDir::new(test).unwrap();
                let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                mock(path, test.to_string(), "encrypted", "kzgcommit", "hashed", 1, "resources", None);
                test_dir.close().unwrap();
            }

            #(#[test_case(TESTS[N])])*
            fn mock_encrypted_output_(test: &str) {
                crate::native_tests::init_binary();
                let test_dir = TempDir::new(test).unwrap();
                let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                mock(path, test.to_string(), "public", "private", "encrypted", 1, "resources", None);
                test_dir.close().unwrap();
            }

            #(#[test_case(TESTS[N])])*
            fn mock_encrypted_input_params_(test: &str) {
                crate::native_tests::init_binary();
                let test_dir = TempDir::new(test).unwrap();
                let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                mock(path, test.to_string(), "encrypted", "encrypted", "public", 1, "resources", None);
                test_dir.close().unwrap();
            }

            #(#[test_case(TESTS[N])])*
            fn mock_encrypted_all_(test: &str) {
                crate::native_tests::init_binary();
                let test_dir = TempDir::new(test).unwrap();
                let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                mock(path, test.to_string(), "encrypted", "encrypted", "encrypted", 1, "resources", None);
                test_dir.close().unwrap();
            }


            #(#[test_case(TESTS[N])])*
            fn mock_kzg_all_(test: &str) {
                crate::native_tests::init_binary();
                let test_dir = TempDir::new(test).unwrap();
                let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                mock(path, test.to_string(), "kzgcommit", "kzgcommit", "kzgcommit", 1, "resources", None);
                test_dir.close().unwrap();
            }



            #(#[test_case(TESTS[N])])*
            fn mock_encrypted_input_hashed_params_(test: &str) {
                crate::native_tests::init_binary();
                let test_dir = TempDir::new(test).unwrap();
                let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                mock(path, test.to_string(), "encrypted", "hashed", "public", 1, "resources", None);
                test_dir.close().unwrap();
            }

            #(#[test_case(TESTS[N])])*
            fn mock_hashed_input_output_(test: &str) {
                crate::native_tests::init_binary();
                let test_dir = TempDir::new(test).unwrap();
                let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                mock(path, test.to_string(), "hashed", "private", "hashed", 1, "resources", None);
                test_dir.close().unwrap();
            }

            #(#[test_case(TESTS[N])])*
            fn mock_hashed_input_params_(test: &str) {
                crate::native_tests::init_binary();
                let test_dir = TempDir::new(test).unwrap();
                let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                // needs an extra row for the large model
                mock(path, test.to_string(),"hashed", "hashed", "public", 1, "resources", None);
                test_dir.close().unwrap();
            }

            #(#[test_case(TESTS[N])])*
            fn mock_hashed_all_(test: &str) {
                crate::native_tests::init_binary();
                let test_dir = TempDir::new(test).unwrap();
                let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                // needs an extra row for the large model
                mock(path, test.to_string(),"hashed", "hashed", "hashed", 1, "resources", None);
                test_dir.close().unwrap();
            }

            #(#[test_case(TESTS[N])])*
            fn kzg_prove_and_verify_double_col(test: &str) {
                crate::native_tests::init_binary();
                let test_dir = TempDir::new(test).unwrap();
                let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
               kzg_prove_and_verify(path, test.to_string(), "safe", "private", "private", "public", 2, None, false);
               test_dir.close().unwrap();
            }

            #(#[test_case(TESTS[N])])*
            fn kzg_prove_and_verify_triple_col(test: &str) {
                crate::native_tests::init_binary();
                let test_dir = TempDir::new(test).unwrap();
                let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
               kzg_prove_and_verify(path, test.to_string(), "safe", "private", "private", "public", 3, None, false);
               test_dir.close().unwrap();
            }

            #(#[test_case(TESTS[N])])*
            fn kzg_prove_and_verify_quadruple_col(test: &str) {
                crate::native_tests::init_binary();
                let test_dir = TempDir::new(test).unwrap();
                let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
               kzg_prove_and_verify(path, test.to_string(), "safe", "private", "private", "public", 4, None, false);
               test_dir.close().unwrap();
            }

            #(#[test_case(TESTS[N])])*
            fn kzg_prove_and_verify_octuple_col(test: &str) {
                crate::native_tests::init_binary();
                let test_dir = TempDir::new(test).unwrap();
                let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
               kzg_prove_and_verify(path, test.to_string(), "safe", "private", "private", "public", 8, None, false);
               test_dir.close().unwrap();
            }

            #(#[test_case(TESTS[N])])*
            fn kzg_prove_and_verify_(test: &str) {
                crate::native_tests::init_binary();
                let test_dir = TempDir::new(test).unwrap();
                let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
               kzg_prove_and_verify(path, test.to_string(), "safe", "private", "private", "public", 1, None, false);
               test_dir.close().unwrap();
            }

            #(#[test_case(TESTS[N])])*
            fn kzg_prove_and_verify_public_input_(test: &str) {
                crate::native_tests::init_binary();
                let test_dir = TempDir::new(test).unwrap();
                let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
               kzg_prove_and_verify(path, test.to_string(), "safe", "public", "private", "public", 1, None, false);
               test_dir.close().unwrap();
            }

            #(#[test_case(TESTS[N])])*
            fn kzg_prove_and_verify_fixed_params_(test: &str) {
                crate::native_tests::init_binary();
                let test_dir = TempDir::new(test).unwrap();
                let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
               kzg_prove_and_verify(path, test.to_string(), "safe", "private", "fixed", "public", 1, None, false);
               test_dir.close().unwrap();
            }

            #(#[test_case(TESTS[N])])*
            fn kzg_prove_and_verify_hashed_output(test: &str) {
                crate::native_tests::init_binary();
                let test_dir = TempDir::new(test).unwrap();
                let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
               kzg_prove_and_verify(path, test.to_string(), "safe", "private", "private", "hashed", 1, None, false);
               test_dir.close().unwrap();
            }

            #(#[test_case(TESTS[N])])*
            fn kzg_prove_and_verify_kzg_output(test: &str) {
                crate::native_tests::init_binary();
                let test_dir = TempDir::new(test).unwrap();
                let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
               kzg_prove_and_verify(path, test.to_string(), "safe", "private", "private", "kzgcommit", 1, None, false);
               test_dir.close().unwrap();
            }

            #(#[test_case(TESTS[N])])*
            fn kzg_prove_and_verify_encrypted_output(test: &str) {
                crate::native_tests::init_binary();
                let test_dir = TempDir::new(test).unwrap();
                let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
               kzg_prove_and_verify(path, test.to_string(), "safe", "private", "private", "encrypted", 1, None, false);
               test_dir.close().unwrap();
            }

            #(#[test_case(TESTS[N])])*
            fn kzg_fuzz_(test: &str) {
                crate::native_tests::init_binary();
                let test_dir = TempDir::new(test).unwrap();
                let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                kzg_fuzz(path, test.to_string(), "evm");
                test_dir.close().unwrap();
            }

            });

            seq!(N in 0..=47 {

                #(#[test_case(WASM_TESTS[N])])*
                fn kzg_prove_and_verify_with_overflow_(test: &str) {
                    crate::native_tests::init_binary();
                    // crate::native_tests::init_wasm();
                    let test_dir = TempDir::new(test).unwrap();
                    env_logger::init();
                    let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                    kzg_prove_and_verify(path, test.to_string(), "safe", "private", "private", "public", 1, Some(vec![0,1]), true);
                    run_js_tests(path, test.to_string(), "testWasm");
                    test_dir.close().unwrap();
                }

                #(#[test_case(WASM_TESTS[N])])*
                fn kzg_prove_and_verify_with_overflow_fixed_params_(test: &str) {
                    crate::native_tests::init_binary();
                    // crate::native_tests::init_wasm();
                    let test_dir = TempDir::new(test).unwrap();
                    env_logger::init();
                    let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                    kzg_prove_and_verify(path, test.to_string(), "safe", "private", "fixed", "public", 1, Some(vec![0,1]), true);
                    run_js_tests(path, test.to_string(), "testWasm");
                    test_dir.close().unwrap();
                }

            });

            seq!(N in 0..=4 {

            #(#[test_case(LARGE_TESTS[N])])*
            #[ignore]
            fn large_kzg_prove_and_verify_(test: &str) {
                crate::native_tests::init_binary();
                let test_dir = TempDir::new(test).unwrap();
                let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                kzg_prove_and_verify(path, test.to_string(), "unsafe", "private", "fixed", "public", 1, Some(vec![0,6]), false);
                test_dir.close().unwrap();
            }

            #(#[test_case(LARGE_TESTS[N])])*
            #[ignore]
            fn large_mock_(test: &str) {
                crate::native_tests::init_binary();
                let test_dir = TempDir::new(test).unwrap();
                let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                mock(path, test.to_string(), "private", "fixed", "public", 1, "resources", Some(vec![0,6]));
                test_dir.close().unwrap();
            }
        });
    }
    };
}

    macro_rules! test_func_evm {
    () => {
        #[cfg(test)]
        mod tests_evm {
            use seq_macro::seq;
            use crate::native_tests::TESTS_EVM;
            use crate::native_tests::TESTS_EVM_AGGR;
            use test_case::test_case;
            use crate::native_tests::kzg_evm_prove_and_verify;
            use crate::native_tests::run_js_tests;
            use crate::native_tests::kzg_evm_on_chain_input_prove_and_verify;
            use crate::native_tests::kzg_evm_aggr_prove_and_verify;
            use crate::native_tests::kzg_fuzz;
            use tempdir::TempDir;

            /// Currently only on chain inputs that return a non-negative value are supported.
            const TESTS_ON_CHAIN_INPUT: [&str; 17] = [
                "1l_mlp",
                "1l_average",
                "1l_reshape",
                "1l_sigmoid",
                "1l_div",
                "1l_sqrt",
                "1l_prelu",
                "1l_var",
                "1l_leakyrelu",
                "1l_gelu_noappx",
                "1l_relu",
                "1l_tanh",
                "2l_relu_sigmoid_small",
                "2l_relu_small",
                "2l_relu_fc",
                "min",
                "max"
            ];

            seq!(N in 0..=16 {
                #(#[test_case(TESTS_ON_CHAIN_INPUT[N])])*
                fn kzg_evm_on_chain_input_prove_and_verify_(test: &str) {
                    crate::native_tests::init_binary();
                    let test_dir = TempDir::new(test).unwrap();
                    let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                    let _anvil_child = crate::native_tests::start_anvil(true);
                    kzg_evm_on_chain_input_prove_and_verify(path, test.to_string(), "on-chain", "file", "public", "private");
                    test_dir.close().unwrap();
                }

                #(#[test_case(TESTS_ON_CHAIN_INPUT[N])])*
                fn kzg_evm_on_chain_output_prove_and_verify_(test: &str) {
                    crate::native_tests::init_binary();
                    let test_dir = TempDir::new(test).unwrap();
                    let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                    let _anvil_child = crate::native_tests::start_anvil(true);
                    kzg_evm_on_chain_input_prove_and_verify(path, test.to_string(), "file", "on-chain", "private", "public");
                    test_dir.close().unwrap();
                }

                #(#[test_case(TESTS_ON_CHAIN_INPUT[N])])*
                fn kzg_evm_on_chain_input_output_prove_and_verify_(test: &str) {
                    crate::native_tests::init_binary();
                    let test_dir = TempDir::new(test).unwrap();
                    let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                    let _anvil_child = crate::native_tests::start_anvil(true);
                    kzg_evm_on_chain_input_prove_and_verify(path, test.to_string(), "on-chain", "on-chain", "public", "public");
                    test_dir.close().unwrap();
                }

                #(#[test_case(TESTS_ON_CHAIN_INPUT[N])])*
                fn kzg_evm_on_chain_input_output_hashed_prove_and_verify_(test: &str) {
                    crate::native_tests::init_binary();
                    let test_dir = TempDir::new(test).unwrap();
                    let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                    let _anvil_child = crate::native_tests::start_anvil(true);
                    kzg_evm_on_chain_input_prove_and_verify(path, test.to_string(), "on-chain", "on-chain", "hashed", "hashed");
                    test_dir.close().unwrap();
                }
            });


            seq!(N in 0..= 17 {
                // these take a particularly long time to run
                #(#[test_case(TESTS_EVM_AGGR[N])])*
                #[ignore]
                fn kzg_evm_aggr_prove_and_verify_(test: &str) {
                    crate::native_tests::init_binary();
                    let test_dir = TempDir::new(test).unwrap();
                    let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                    let _anvil_child = crate::native_tests::start_anvil(false);
                    kzg_evm_aggr_prove_and_verify(path, test.to_string(), "private", "private", "public");
                    test_dir.close().unwrap();
                }

                // these take a particularly long time to run
                #[test]
                #[ignore]
                fn kzg_evm_aggr_prove_and_verify_encrypted_input_() {
                    let test = "1l_mlp";
                    crate::native_tests::init_binary();
                    let test_dir = TempDir::new(test).unwrap();
                    let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                    let _anvil_child = crate::native_tests::start_anvil(false);
                    kzg_evm_aggr_prove_and_verify(path, test.to_string(), "encrypted", "private", "public");
                    test_dir.close().unwrap();
                }
            });


            seq!(N in 0..= 20 {

                #(#[test_case(TESTS_EVM[N])])*
                fn kzg_evm_prove_and_verify_(test: &str) {
                    crate::native_tests::init_binary();
                    let test_dir = TempDir::new(test).unwrap();
                    let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                    let _anvil_child = crate::native_tests::start_anvil(false);
                    kzg_evm_prove_and_verify(path, test.to_string(), "private", "private", "public");
                    run_js_tests(path, test.to_string(), "testBrowserEvmVerify");
                    test_dir.close().unwrap();

                }

                #[test]
                #[ignore]
                fn kzg_evm_prove_and_verify_encrypted_input_() {
                    let test = "1l_mlp";
                    crate::native_tests::init_binary();
                    let test_dir = TempDir::new(test).unwrap();
                    let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                    let _anvil_child = crate::native_tests::start_anvil(false);
                    kzg_evm_prove_and_verify(path, test.to_string(), "encrypted", "private", "public");
                    run_js_tests(path, test.to_string(), "testBrowserEvmVerify");
                    test_dir.close().unwrap();
                }

                #(#[test_case(TESTS_EVM[N])])*
                fn kzg_evm_hashed_input_prove_and_verify_(test: &str) {
                    crate::native_tests::init_binary();
                    let test_dir = TempDir::new(test).unwrap();
                    let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                    let mut _anvil_child = crate::native_tests::start_anvil(false);
                    kzg_evm_prove_and_verify(path, test.to_string(), "hashed", "private", "private");
                    run_js_tests(path, test.to_string(), "testBrowserEvmVerify");
                    test_dir.close().unwrap();
                }


                #(#[test_case(TESTS_EVM[N])])*
                fn kzg_evm_kzg_input_prove_and_verify_(test: &str) {
                    crate::native_tests::init_binary();
                    let test_dir = TempDir::new(test).unwrap();
                    let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                    let mut _anvil_child = crate::native_tests::start_anvil(false);
                    kzg_evm_prove_and_verify(path, test.to_string(), "kzgcommit", "private", "public");
                    run_js_tests(path, test.to_string(), "testBrowserEvmVerify");
                    test_dir.close().unwrap();
                }


                #(#[test_case(TESTS_EVM[N])])*
                fn kzg_evm_hashed_params_prove_and_verify_(test: &str) {
                    crate::native_tests::init_binary();
                    let test_dir = TempDir::new(test).unwrap();
                    let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                    let _anvil_child = crate::native_tests::start_anvil(false);
                    kzg_evm_prove_and_verify(path, test.to_string(), "private", "hashed", "public");
                    run_js_tests(path, test.to_string(), "testBrowserEvmVerify");
                    test_dir.close().unwrap();

                }

                #(#[test_case(TESTS_EVM[N])])*
                fn kzg_evm_hashed_output_prove_and_verify_(test: &str) {
                    crate::native_tests::init_binary();
                    let test_dir = TempDir::new(test).unwrap();
                    let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                    let _anvil_child = crate::native_tests::start_anvil(false);
                    kzg_evm_prove_and_verify(path, test.to_string(), "private", "private", "hashed");
                    run_js_tests(path, test.to_string(), "testBrowserEvmVerify");
                    test_dir.close().unwrap();
                }


                #(#[test_case(TESTS_EVM[N])])*
                fn kzg_evm_kzg_params_prove_and_verify_(test: &str) {
                    crate::native_tests::init_binary();
                    let test_dir = TempDir::new(test).unwrap();
                    let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                    let _anvil_child = crate::native_tests::start_anvil(false);
                    kzg_evm_prove_and_verify(path, test.to_string(), "private", "kzgcommit", "public");
                    run_js_tests(path, test.to_string(), "testBrowserEvmVerify");
                    test_dir.close().unwrap();
                }


                #(#[test_case(TESTS_EVM[N])])*
                fn kzg_evm_kzg_output_prove_and_verify_(test: &str) {
                    crate::native_tests::init_binary();
                    let test_dir = TempDir::new(test).unwrap();
                    let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                    let _anvil_child = crate::native_tests::start_anvil(false);
                    kzg_evm_prove_and_verify(path, test.to_string(), "private", "private", "kzgcommit");
                    run_js_tests(path, test.to_string(), "testBrowserEvmVerify");
                    test_dir.close().unwrap();
                }



                #(#[test_case(TESTS_EVM[N])])*
                fn kzg_evm_fuzz_(test: &str) {
                    crate::native_tests::init_binary();
                    let test_dir = TempDir::new(test).unwrap();
                    let path = test_dir.path().to_str().unwrap(); crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test);
                    let _anvil_child = crate::native_tests::start_anvil(false);
                    kzg_fuzz(path, test.to_string(), "evm");
                    test_dir.close().unwrap();

                }
            });
    }
    };
}

    macro_rules! test_func_examples {
    () => {
        #[cfg(test)]
        mod tests_examples {
            use seq_macro::seq;
            use crate::native_tests::EXAMPLES;
            use test_case::test_case;
            use crate::native_tests::run_example as run;
            seq!(N in 0..=1 {
            #(#[test_case(EXAMPLES[N])])*
            fn example_(test: &str) {
                run(test.to_string());
            }
            });
    }
    };
}

    macro_rules! test_neg_examples {
    () => {
        #[cfg(test)]
        mod neg_tests {
            use seq_macro::seq;
            use crate::native_tests::NEG_TESTS;
            use test_case::test_case;
            use crate::native_tests::neg_mock as run;
            use tempdir::TempDir;

            seq!(N in 0..=1 {
            #(#[test_case(NEG_TESTS[N])])*
            fn neg_examples_(test: (&str, &str)) {
                crate::native_tests::init_binary();
                let test_dir = TempDir::new(test.0).unwrap();
                let path = test_dir.path().to_str().unwrap();
                crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test.0);
                crate::native_tests::mv_test_(test_dir.path().to_str().unwrap(), test.1);
                run(path, test.0.to_string(), test.1.to_string());
                test_dir.close().unwrap();
            }

            });
    }
    };
}

    test_func!();
    test_func_aggr!();
    test_func_evm!();
    test_func_examples!();
    test_neg_examples!();

    fn model_serialization(test_dir: &str, example_name: String) {
        let model_path = format!("{}/{}/network.onnx", test_dir, example_name);
        let serialization_path = format!("{}/{}/network.ezkl", test_dir, example_name);
        let run_args = ezkl::RunArgs {
            param_visibility: Visibility::Public,
            variables: vec![("batch_size".to_string(), 1)],
            ..Default::default()
        };

        let model =
            ezkl::graph::Model::new(&mut std::fs::File::open(model_path).unwrap(), &run_args)
                .unwrap();

        model.save(serialization_path.clone().into()).unwrap();

        let loaded_model = ezkl::graph::Model::load(serialization_path.into()).unwrap();
        assert_eq!(model, loaded_model)
    }

    fn model_serialization_different_binaries(test_dir: &str, example_name: String) {
        let status = Command::new("cargo")
            .args([
                "run",
                "--bin",
                "ezkl",
                "--",
                "gen-settings",
                "-M",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                &format!(
                    "--settings-path={}/{}/settings.json",
                    test_dir, example_name
                ),
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let status = Command::new("cargo")
            .args([
                "run",
                "--bin",
                "ezkl",
                "--",
                "compile-circuit",
                "-M",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                "--compiled-circuit",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                &format!(
                    "--settings-path={}/{}/settings.json",
                    test_dir, example_name
                ),
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        // now alter binary slightly
        // create new temp cargo.toml with a different version
        // cpy old cargo.toml to cargo.toml.bak
        let status = Command::new("cp")
            .args(["Cargo.toml", "Cargo.toml.bak"])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let mut cargo_toml = std::fs::File::open("Cargo.toml").unwrap();
        let mut cargo_toml_contents = String::new();
        cargo_toml.read_to_string(&mut cargo_toml_contents).unwrap();
        let mut cargo_toml_contents = cargo_toml_contents.split('\n').collect::<Vec<_>>();

        // draw a random version number from 0.0.0 to 0.100.100
        let mut rng = rand::thread_rng();
        let version = &format!(
            "version = \"0.{}.{}-test\"",
            rng.gen_range(0..100),
            rng.gen_range(0..100)
        );
        let cargo_toml_contents = cargo_toml_contents
            .iter_mut()
            .map(|line| {
                if line.starts_with("version") {
                    *line = version;
                }
                *line
            })
            .collect::<Vec<_>>();
        let mut cargo_toml = std::fs::File::create("Cargo.toml").unwrap();
        cargo_toml
            .write_all(cargo_toml_contents.join("\n").as_bytes())
            .unwrap();

        let status = Command::new("cargo")
            .args([
                "run",
                "--bin",
                "ezkl",
                "--",
                "gen-witness",
                "-D",
                &format!("{}/{}/input.json", test_dir, example_name),
                "-M",
                &format!("{}/{}/network.onnx", test_dir, example_name),
                "-O",
                &format!("{}/{}/witness.json", test_dir, example_name),
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        // now delete cargo.toml and move cargo.toml.bak to cargo.toml
        let status = Command::new("rm")
            .args(["Cargo.toml"])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let status = Command::new("mv")
            .args(["Cargo.toml.bak", "Cargo.toml"])
            .status()
            .expect("failed to execute process");
        assert!(status.success());
    }

    // Mock prove (fast, but does not cover some potential issues)
    fn neg_mock(test_dir: &str, example_name: String, counter_example: String) {
        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "gen-settings",
                "-M",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                &format!(
                    "--settings-path={}/{}/settings.json",
                    test_dir, example_name
                ),
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "compile-circuit",
                "-M",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                "--compiled-circuit",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                &format!(
                    "--settings-path={}/{}/settings.json",
                    test_dir, example_name
                ),
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "gen-witness",
                "-D",
                &format!("{}/{}/input.json", test_dir, example_name),
                "-M",
                &format!("{}/{}/network.onnx", test_dir, example_name),
                "-O",
                &format!("{}/{}/witness.json", test_dir, example_name),
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "mock",
                "-W",
                format!("{}/{}/witness.json", test_dir, counter_example).as_str(),
                "-M",
                format!("{}/{}/network.compiled", test_dir, example_name).as_str(),
            ])
            .status()
            .expect("failed to execute process");
        assert!(!status.success());
    }

    // Mock prove (fast, but does not cover some potential issues)
    fn run_example(example_name: String) {
        let status = Command::new("cargo")
            .args(["run", "--release", "--example", example_name.as_str()])
            .status()
            .expect("failed to execute process");
        assert!(status.success());
    }

    // Mock prove (fast, but does not cover some potential issues)
    #[allow(clippy::too_many_arguments)]
    fn mock(
        test_dir: &str,
        example_name: String,
        input_visibility: &str,
        param_visibility: &str,
        output_visibility: &str,
        batch_size: usize,
        cal_target: &str,
        scales_to_use: Option<Vec<u32>>,
    ) {
        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "gen-settings",
                "-M",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                &format!(
                    "--settings-path={}/{}/settings.json",
                    test_dir, example_name
                ),
                &format!("--variables=batch_size={}", batch_size),
                &format!("--input-visibility={}", input_visibility),
                &format!("--param-visibility={}", param_visibility),
                &format!("--output-visibility={}", output_visibility),
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let mut calibrate_args = vec![
            "calibrate-settings".to_string(),
            "--data".to_string(),
            format!("{}/{}/input.json", test_dir, example_name),
            "-M".to_string(),
            format!("{}/{}/network.onnx", test_dir, example_name),
            format!(
                "--settings-path={}/{}/settings.json",
                test_dir, example_name
            ),
            format!("--target={}", cal_target),
        ];

        if let Some(scales) = scales_to_use {
            let scales = scales
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join(",");
            calibrate_args.push("--scales".to_string());
            calibrate_args.push(scales);
        }

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args(calibrate_args)
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "compile-circuit",
                "-M",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                "--compiled-circuit",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                &format!(
                    "--settings-path={}/{}/settings.json",
                    test_dir, example_name
                ),
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "gen-witness",
                "-D",
                &format!("{}/{}/input.json", test_dir, example_name),
                "-M",
                &format!("{}/{}/network.onnx", test_dir, example_name),
                "-O",
                &format!("{}/{}/witness_mock.json", test_dir, example_name),
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "mock",
                "-W",
                format!("{}/{}/witness_mock.json", test_dir, example_name).as_str(),
                "-M",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());
    }

    // Mock prove (fast, but does not cover some potential issues)
    fn accuracy_measurement(
        test_dir: &str,
        example_name: String,
        input_visibility: &str,
        param_visibility: &str,
        output_visibility: &str,
        batch_size: usize,
        cal_target: &str,
    ) {
        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "gen-settings",
                "-M",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                &format!(
                    "--settings-path={}/{}/settings.json",
                    test_dir, example_name
                ),
                &format!("--variables=batch_size={}", batch_size),
                &format!("--input-visibility={}", input_visibility),
                &format!("--param-visibility={}", param_visibility),
                &format!("--output-visibility={}", output_visibility),
            ])
            .stdout(std::process::Stdio::null())
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "calibrate-settings",
                "--data",
                format!("{}/{}/input.json", test_dir, example_name).as_str(),
                "-M",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                &format!(
                    "--settings-path={}/{}/settings.json",
                    test_dir, example_name
                ),
                &format!("--target={}", cal_target),
            ])
            .stdout(std::process::Stdio::null())
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "compile-circuit",
                "-M",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                "--compiled-circuit",
                format!("{}/{}/network.compiled", test_dir, example_name).as_str(),
                &format!(
                    "--settings-path={}/{}/settings.json",
                    test_dir, example_name
                ),
            ])
            .stdout(std::process::Stdio::null())
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "gen-witness",
                "-D",
                &format!("{}/{}/input.json", test_dir, example_name),
                "-M",
                &format!("{}/{}/network.compiled", test_dir, example_name),
                "-O",
                &format!("{}/{}/witness.json", test_dir, example_name),
            ])
            .stdout(std::process::Stdio::null())
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        println!(
            " ------------ running accuracy measurement for {}",
            example_name
        );
        // run python ./output_comparison.py in the test dir
        let status = Command::new("python")
            .args([
                "tests/output_comparison.py",
                &format!("{}/{}/network.onnx", test_dir, example_name),
                &format!("{}/{}/input.json", test_dir, example_name),
                &format!("{}/{}/witness.json", test_dir, example_name),
                &format!("{}/{}/settings.json", test_dir, example_name),
            ])
            .status()
            .expect("failed to execute process");

        assert!(status.success());
    }

    // Mock prove (fast, but does not cover some potential issues)
    fn render_circuit(test_dir: &str, example_name: String) {
        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "render-circuit",
                "-M",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                "-O",
                format!("{}/{}/render.png", test_dir, example_name).as_str(),
                "--lookup-range=(-32768,32768)",
                "-K=17",
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());
    }

    // Mock prove (fast, but does not cover some potential issues)
    fn tutorial(test_dir: &str, tolerance: &str) {
        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "gen-settings",
                "-M",
                format!("{}/tutorial/network.onnx", test_dir).as_str(),
                &format!("--settings-path={}/tutorial/settings.json", test_dir),
                "--lookup-range=(-32768,32768)",
                "--logrows=17",
                "--input-scale=4",
                "--param-scale=4",
                &format!("--tolerance={}", tolerance),
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "compile-circuit",
                "-M",
                format!("{}/tutorial/network.onnx", test_dir).as_str(),
                "--compiled-circuit",
                format!("{}/tutorial/network.onnx", test_dir).as_str(),
                &format!("--settings-path={}/tutorial/settings.json", test_dir),
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "gen-witness",
                "-D",
                &format!("{}/tutorial/input.json", test_dir),
                "-M",
                &format!("{}/tutorial/network.onnx", test_dir),
                "-O",
                &format!("{}/tutorial/witness_tutorial.json", test_dir),
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "mock",
                "-W",
                format!("{}/tutorial/witness_tutorial.json", test_dir).as_str(),
                "-M",
                format!("{}/tutorial/network.onnx", test_dir).as_str(),
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());
    }

    // prove-serialize-verify, the usual full path
    fn kzg_aggr_mock_prove_and_verify(test_dir: &str, example_name: String) {
        let settings_path = format!("{}/{}/settings.json", test_dir, example_name);

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "gen-settings",
                "-M",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                "-O",
                &settings_path,
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "calibrate-settings",
                "--data",
                format!("{}/{}/input.json", test_dir, example_name).as_str(),
                "-M",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                "-O",
                &settings_path,
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        // now shrink the logrows by 1 to test for overflow
        let mut settings: GraphSettings =
            GraphSettings::load(&settings_path.clone().into()).unwrap();
        // anything smaller and the circuit will not compile
        if settings.run_args.logrows > 4 {
            settings.run_args.logrows -= 1;
        }
        settings.save(&settings_path.clone().into()).unwrap();

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "compile-circuit",
                "-M",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                "--compiled-circuit",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                "-S",
                &settings_path,
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "gen-witness",
                "-D",
                format!("{}/{}/input.json", test_dir, example_name).as_str(),
                "-M",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                "-O",
                format!("{}/{}/input.json", test_dir, example_name).as_str(),
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let srs_path = download_srs(test_dir, 17);
        let srs_path = format!("--srs-path={}", srs_path);

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "setup",
                "-M",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                "--pk-path",
                &format!("{}/{}/key.pk", test_dir, example_name),
                "--vk-path",
                &format!("{}/{}/key.vk", test_dir, example_name),
                &srs_path,
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());
        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "prove",
                "-W",
                format!("{}/{}/input.json", test_dir, example_name).as_str(),
                "-M",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                "--proof-path",
                &format!("{}/{}/proof.pf", test_dir, example_name),
                "--pk-path",
                &format!("{}/{}/key.pk", test_dir, example_name),
                &srs_path,
                "--proof-type=for-aggr",
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());
        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "mock-aggregate",
                "--logrows=23",
                "--aggregation-snarks",
                &format!("{}/{}/proof.pf", test_dir, example_name),
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());
    }

    // prove-serialize-verify, the usual full path
    fn kzg_aggr_prove_and_verify(test_dir: &str, example_name: String) {
        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "gen-settings",
                "-M",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                &format!(
                    "--settings-path={}/{}/settings.json",
                    test_dir, example_name
                ),
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "calibrate-settings",
                "--data",
                format!("{}/{}/input.json", test_dir, example_name).as_str(),
                "-M",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                &format!(
                    "--settings-path={}/{}/settings.json",
                    test_dir, example_name
                ),
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "compile-circuit",
                "-M",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                "--compiled-circuit",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                &format!(
                    "--settings-path={}/{}/settings.json",
                    test_dir, example_name
                ),
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "gen-witness",
                "-D",
                format!("{}/{}/input.json", test_dir, example_name).as_str(),
                "-M",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                "-O",
                format!("{}/{}/input.json", test_dir, example_name).as_str(),
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let srs_path = download_srs(test_dir, 23);
        let srs_path = format!("--srs-path={}", srs_path);

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "setup",
                "-M",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                "--pk-path",
                &format!("{}/{}/key.pk", test_dir, example_name),
                "--vk-path",
                &format!("{}/{}/key.vk", test_dir, example_name),
                &srs_path,
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());
        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "prove",
                "-W",
                format!("{}/{}/input.json", test_dir, example_name).as_str(),
                "-M",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                "--proof-path",
                &format!("{}/{}/proof.pf", test_dir, example_name),
                "--pk-path",
                &format!("{}/{}/key.pk", test_dir, example_name),
                &srs_path,
                "--proof-type=for-aggr",
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        // now setup-aggregate
        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "setup-aggregate",
                "--sample-snarks",
                &format!("{}/{}/proof.pf", test_dir, example_name),
                "--logrows=23",
                "--vk-path",
                &format!("{}/{}/aggr.vk", test_dir, example_name),
                "--pk-path",
                &format!("{}/{}/aggr.pk", test_dir, example_name),
                &srs_path,
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "aggregate",
                "--logrows=23",
                "--aggregation-snarks",
                &format!("{}/{}/proof.pf", test_dir, example_name),
                "--proof-path",
                &format!("{}/{}/aggr.pf", test_dir, example_name),
                "--pk-path",
                &format!("{}/{}/aggr.pk", test_dir, example_name),
                &srs_path,
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());
        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "verify-aggr",
                "--logrows=23",
                "--proof-path",
                &format!("{}/{}/aggr.pf", test_dir, example_name),
                "--vk-path",
                &format!("{}/{}/aggr.vk", test_dir, example_name),
                &srs_path,
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());
    }

    // prove-serialize-verify, the usual full path
    fn kzg_evm_aggr_prove_and_verify(
        test_dir: &str,
        example_name: String,
        input_visibility: &str,
        param_visibility: &str,
        output_visibility: &str,
    ) {
        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "gen-settings",
                "-M",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                &format!(
                    "--settings-path={}/{}/settings.json",
                    test_dir, example_name
                ),
                &format!("--input-visibility={}", input_visibility),
                &format!("--param-visibility={}", param_visibility),
                &format!("--output-visibility={}", output_visibility),
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "calibrate-settings",
                "--data",
                format!("{}/{}/input.json", test_dir, example_name).as_str(),
                "-M",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                &format!(
                    "--settings-path={}/{}/settings.json",
                    test_dir, example_name
                ),
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "compile-circuit",
                "-M",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                "--compiled-circuit",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                &format!(
                    "--settings-path={}/{}/settings.json",
                    test_dir, example_name
                ),
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "gen-witness",
                "-D",
                format!("{}/{}/input.json", test_dir, example_name).as_str(),
                "-M",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                "-O",
                format!("{}/{}/input.json", test_dir, example_name).as_str(),
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let srs_path = download_srs(test_dir, 23);
        let srs_path = format!("--srs-path={}", srs_path);

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "setup",
                "-M",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                "--vk-path",
                &format!("{}/{}/evm.vk", test_dir, example_name),
                "--pk-path",
                &format!("{}/{}/evm.pk", test_dir, example_name),
                &srs_path,
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "prove",
                "-W",
                format!("{}/{}/input.json", test_dir, example_name).as_str(),
                "-M",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                "--proof-path",
                &format!("{}/{}/evm.pf", test_dir, example_name),
                "--pk-path",
                &format!("{}/{}/evm.pk", test_dir, example_name),
                &srs_path,
                "--proof-type=for-aggr",
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        // now setup-aggregate
        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "setup-aggregate",
                "--sample-snarks",
                &format!("{}/{}/evm.pf", test_dir, example_name),
                "--logrows=23",
                "--vk-path",
                &format!("{}/{}/evm_aggr.vk", test_dir, example_name),
                "--pk-path",
                &format!("{}/{}/evm_aggr.pk", test_dir, example_name),
                &srs_path,
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "aggregate",
                "--logrows=23",
                "--aggregation-snarks",
                &format!("{}/{}/evm.pf", test_dir, example_name),
                "--proof-path",
                &format!("{}/{}/evm_aggr.pf", test_dir, example_name),
                "--pk-path",
                &format!("{}/{}/evm_aggr.pk", test_dir, example_name),
                &srs_path,
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let vk_arg = format!("{}/{}/evm_aggr.vk", test_dir, example_name);

        fn build_args<'a>(base_args: Vec<&'a str>, sol_arg: &'a str) -> Vec<&'a str> {
            let mut args = base_args;

            args.push("--sol-code-path");
            args.push(sol_arg);
            args
        }

        let sol_arg = format!("{}/{}/kzg_aggr.sol", test_dir, example_name);
        let addr_path_arg = format!("--addr-path={}/{}/addr.txt", test_dir, example_name);
        let rpc_arg = format!("--rpc-url={}", *ANVIL_URL);
        let settings_arg = format!("{}/{}/settings.json", test_dir, example_name);
        let private_key = format!("--private-key={}", *ANVIL_DEFAULT_PRIVATE_KEY);

        let base_args = vec![
            "create-evm-verifier-aggr",
            srs_path.as_str(),
            "--vk-path",
            vk_arg.as_str(),
            "--aggregation-settings",
            settings_arg.as_str(),
        ];

        let args = build_args(base_args, &sol_arg);

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args(args)
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        // deploy the verifier
        let args = vec![
            "deploy-evm-verifier",
            rpc_arg.as_str(),
            addr_path_arg.as_str(),
            "--sol-code-path",
            sol_arg.as_str(),
            private_key.as_str(),
        ];

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args(&args)
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        // read in the address
        let addr = std::fs::read_to_string(format!("{}/{}/addr.txt", test_dir, example_name))
            .expect("failed to read address file");

        let deployed_addr_arg = format!("--addr-verifier={}", addr);

        let pf_arg = format!("{}/{}/evm_aggr.pf", test_dir, example_name);

        let mut base_args = vec![
            "verify-evm",
            "--proof-path",
            pf_arg.as_str(),
            deployed_addr_arg.as_str(),
            rpc_arg.as_str(),
        ];

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args(&base_args)
            .status()
            .expect("failed to execute process");
        assert!(status.success());
        // As sanity check, add example that should fail.
        base_args[2] = PF_FAILURE_AGGR;
        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args(base_args)
            .status()
            .expect("failed to execute process");
        assert!(!status.success());
    }

    // prove-serialize-verify, the usual full path
    #[allow(clippy::too_many_arguments)]
    fn kzg_prove_and_verify(
        test_dir: &str,
        example_name: String,
        checkmode: &str,
        input_visibility: &str,
        param_visibility: &str,
        output_visibility: &str,
        num_inner_columns: usize,
        scales_to_use: Option<Vec<u32>>,
        overflow: bool,
    ) {
        let settings_path = format!("{}/{}/settings.json", test_dir, example_name);

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "gen-settings",
                "-M",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                format!("--settings-path={}", settings_path).as_str(),
                &format!("--input-visibility={}", input_visibility),
                &format!("--param-visibility={}", param_visibility),
                &format!("--output-visibility={}", output_visibility),
                &format!("--num-inner-cols={}", num_inner_columns),
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let target_str = if overflow {
            "--target=resources/col-overflow"
        } else {
            "--target=resources"
        };

        let mut calibrate_args = vec![
            "calibrate-settings".to_string(),
            "--data".to_string(),
            format!("{}/{}/input.json", test_dir, example_name),
            "-M".to_string(),
            format!("{}/{}/network.onnx", test_dir, example_name),
            format!(
                "--settings-path={}/{}/settings.json",
                test_dir, example_name
            ),
            target_str.into(),
        ];

        if let Some(scales) = scales_to_use {
            let scales = scales
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join(",");
            calibrate_args.push("--scales".to_string());
            calibrate_args.push(scales);
        }

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args(calibrate_args)
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "compile-circuit",
                "-M",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                "--compiled-circuit",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                &format!(
                    "--settings-path={}/{}/settings.json",
                    test_dir, example_name
                ),
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let srs_path = init_params(test_dir, settings_path.clone().into());
        let srs_path = format!("--srs-path={}", srs_path);

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "setup",
                "-M",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                "--pk-path",
                &format!("{}/{}/key.pk", test_dir, example_name),
                "--vk-path",
                &format!("{}/{}/key.vk", test_dir, example_name),
                &srs_path,
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "gen-witness",
                "-D",
                format!("{}/{}/input.json", test_dir, example_name).as_str(),
                "-M",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                "-O",
                format!("{}/{}/input.json", test_dir, example_name).as_str(),
                "--vk-path",
                &format!("{}/{}/key.vk", test_dir, example_name),
                &srs_path,
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "prove",
                "-W",
                format!("{}/{}/input.json", test_dir, example_name).as_str(),
                "-M",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                "--proof-path",
                &format!("{}/{}/proof.pf", test_dir, example_name),
                "--pk-path",
                &format!("{}/{}/key.pk", test_dir, example_name),
                &srs_path,
                &format!("--check-mode={}", checkmode),
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "swap-proof-commitments",
                "--proof-path",
                &format!("{}/{}/proof.pf", test_dir, example_name),
                "--witness-path",
                format!("{}/{}/input.json", test_dir, example_name).as_str(),
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "verify",
                format!("--settings-path={}", settings_path).as_str(),
                "--proof-path",
                &format!("{}/{}/proof.pf", test_dir, example_name),
                "--vk-path",
                &format!("{}/{}/key.vk", test_dir, example_name),
                &srs_path,
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());
    }

    // prove-serialize-verify, the usual full path
    fn kzg_fuzz(test_dir: &str, example_name: String, transcript: &str) {
        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "gen-settings",
                "-M",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                "-O",
                format!("{}/{}/settings_fuzz.json", test_dir, example_name).as_str(),
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "calibrate-settings",
                "--data",
                format!("{}/{}/input.json", test_dir, example_name).as_str(),
                "-M",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                format!(
                    "--settings-path={}/{}/settings_fuzz.json",
                    test_dir, example_name
                )
                .as_str(),
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "compile-circuit",
                "-M",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                "--compiled-circuit",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                &format!(
                    "--settings-path={}/{}/settings_fuzz.json",
                    test_dir, example_name
                ),
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "gen-witness",
                "-D",
                format!("{}/{}/input.json", test_dir, example_name).as_str(),
                "-M",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                "-O",
                format!("{}/{}/witness_fuzz.json", test_dir, example_name).as_str(),
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "fuzz",
                "-W",
                format!("{}/{}/witness_fuzz.json", test_dir, example_name).as_str(),
                "-M",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                &format!("--num-runs={}", 5),
                &format!("--transcript={}", transcript),
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());
    }

    // prove-serialize-verify, the usual full path
    fn kzg_evm_prove_and_verify(
        test_dir: &str,
        example_name: String,
        input_visibility: &str,
        param_visibility: &str,
        output_visibility: &str,
    ) {
        let anvil_url = ANVIL_URL.as_str();

        let settings_path = format!("{}/{}/settings.json", test_dir, example_name);

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "gen-settings",
                "-M",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                format!("--settings-path={}", settings_path).as_str(),
                &format!("--input-visibility={}", input_visibility),
                &format!("--param-visibility={}", param_visibility),
                &format!("--output-visibility={}", output_visibility),
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "calibrate-settings",
                "--data",
                format!("{}/{}/input.json", test_dir, example_name).as_str(),
                "-M",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                format!("--settings-path={}", settings_path).as_str(),
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "compile-circuit",
                "-M",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                "--compiled-circuit",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                (format!("--settings-path={}", settings_path).as_str()),
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let srs_path = init_params(test_dir, settings_path.clone().into());
        let srs_path = format!("--srs-path={}", srs_path);

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "setup",
                "-M",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                "--pk-path",
                &format!("{}/{}/key.pk", test_dir, example_name),
                "--vk-path",
                &format!("{}/{}/key.vk", test_dir, example_name),
                &srs_path,
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "gen-witness",
                "-D",
                format!("{}/{}/input.json", test_dir, example_name).as_str(),
                "-M",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                "-O",
                format!("{}/{}/input.json", test_dir, example_name).as_str(),
                "--vk-path",
                &format!("{}/{}/key.vk", test_dir, example_name),
                &srs_path,
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "prove",
                "-W",
                format!("{}/{}/input.json", test_dir, example_name).as_str(),
                "-M",
                format!("{}/{}/network.onnx", test_dir, example_name).as_str(),
                "--proof-path",
                &format!("{}/{}/proof.pf", test_dir, example_name),
                "--pk-path",
                &format!("{}/{}/key.pk", test_dir, example_name),
                &srs_path,
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let vk_arg = format!("{}/{}/key.vk", test_dir, example_name);
        let rpc_arg = format!("--rpc-url={}", anvil_url);
        let addr_path_arg = format!("--addr-path={}/{}/addr.txt", test_dir, example_name);
        let settings_arg = format!("--settings-path={}", settings_path);

        // create the verifier
        let mut args = vec![
            "create-evm-verifier",
            &srs_path,
            "--vk-path",
            &vk_arg,
            &settings_arg,
        ];

        let sol_arg = format!("{}/{}/kzg.sol", test_dir, example_name);

        // create everything to test the pipeline
        args.push("--sol-code-path");
        args.push(sol_arg.as_str());

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args(&args)
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        // deploy the verifier
        let mut args = vec![
            "deploy-evm-verifier",
            rpc_arg.as_str(),
            addr_path_arg.as_str(),
        ];

        args.push("--sol-code-path");
        args.push(sol_arg.as_str());

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args(&args)
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        // read in the address
        let addr = std::fs::read_to_string(format!("{}/{}/addr.txt", test_dir, example_name))
            .expect("failed to read address file");

        let deployed_addr_arg = format!("--addr-verifier={}", addr);

        // now verify the proof
        let pf_arg = format!("{}/{}/proof.pf", test_dir, example_name);
        let mut args = vec![
            "verify-evm",
            "--proof-path",
            pf_arg.as_str(),
            rpc_arg.as_str(),
            deployed_addr_arg.as_str(),
        ];

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args(&args)
            .status()
            .expect("failed to execute process");
        assert!(status.success());
        // As sanity check, add example that should fail.
        args[2] = PF_FAILURE;
        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args(args)
            .status()
            .expect("failed to execute process");
        assert!(!status.success());
    }

    // run js browser evm verify tests for a given example
    fn run_js_tests(test_dir: &str, example_name: String, js_test: &str) {
        let status = Command::new("pnpm")
            .args([
                "run",
                "test",
                js_test,
                &format!("--example={}", example_name),
                &format!("--dir={}", test_dir),
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());
    }

    fn kzg_evm_on_chain_input_prove_and_verify(
        test_dir: &str,
        example_name: String,
        input_source: &str,
        output_source: &str,
        input_visbility: &str,
        output_visbility: &str,
    ) {
        let model_path = format!("{}/{}/network.onnx", test_dir, example_name);
        let settings_path = format!("{}/{}/settings.json", test_dir, example_name);

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "gen-settings",
                "-M",
                &model_path,
                format!("--settings-path={}", settings_path).as_str(),
                &format!("--input-visibility={}", input_visbility),
                &format!("--output-visibility={}", output_visbility),
                "--param-visibility=private",
                "--lookup-range=(-32768,32768)",
                "-K=17",
                "--num-inner-cols=1"
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let srs_path = download_srs(test_dir, 17);
        let srs_path = format!("--srs-path={}", srs_path);

        let data_path = format!("{}/{}/input.json", test_dir, example_name);
        let witness_path = format!("{}/{}/witness.json", test_dir, example_name);
        let test_on_chain_data_path = format!("{}/{}/on_chain_input.json", test_dir, example_name);
        let rpc_arg = format!("--rpc-url={}", LIMITLESS_ANVIL_URL.as_str());
        let private_key = format!("--private-key={}", *ANVIL_DEFAULT_PRIVATE_KEY);

        let test_input_source = format!("--input-source={}", input_source);
        let test_output_source = format!("--output-source={}", output_source);

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "compile-circuit",
                "-M",
                &model_path,
                "--compiled-circuit",
                &model_path,
                &format!(
                    "--settings-path={}/{}/settings.json",
                    test_dir, example_name
                ),
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "gen-witness",
                "-D",
                data_path.as_str(),
                "-M",
                &model_path,
                "-O",
                &witness_path,
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        // load witness
        let witness: GraphWitness = GraphWitness::from_path(witness_path.clone().into()).unwrap();
        let mut input: GraphData = GraphData::from_path(data_path.clone().into()).unwrap();

        if input_visbility == "hashed" {
            let hashes = witness.processed_inputs.unwrap().poseidon_hash.unwrap();
            input.input_data = DataSource::File(
                hashes
                    .iter()
                    .map(|h| vec![FileSourceInner::Field(*h)])
                    .collect(),
            );
        }
        if output_visbility == "hashed" {
            let hashes = witness.processed_outputs.unwrap().poseidon_hash.unwrap();
            input.output_data = Some(DataSource::File(
                hashes
                    .iter()
                    .map(|h| vec![FileSourceInner::Field(*h)])
                    .collect(),
            ));
        }

        println!("input is {:?}", input);

        input.save(data_path.clone().into()).unwrap();

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "setup-test-evm-data",
                "-D",
                data_path.as_str(),
                "-M",
                &model_path,
                "--test-data",
                test_on_chain_data_path.as_str(),
                rpc_arg.as_str(),
                test_input_source.as_str(),
                test_output_source.as_str(),
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "setup",
                "-M",
                &model_path,
                "--pk-path",
                &format!("{}/{}/key.pk", test_dir, example_name),
                "--vk-path",
                &format!("{}/{}/key.vk", test_dir, example_name),
                &srs_path,
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "prove",
                "-W",
                &witness_path,
                "-M",
                &model_path,
                "--proof-path",
                &format!("{}/{}/proof.pf", test_dir, example_name),
                "--pk-path",
                &format!("{}/{}/key.pk", test_dir, example_name),
                &srs_path,
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let vk_arg = format!("{}/{}/key.vk", test_dir, example_name);

        let settings_arg = format!("--settings-path={}", settings_path);

        // create the verifier
        let mut args = vec![
            "create-evm-verifier",
            &srs_path,
            "--vk-path",
            &vk_arg,
            &settings_arg,
        ];

        let sol_arg = format!("{}/{}/kzg.sol", test_dir, example_name);

        args.push("--sol-code-path");
        args.push(sol_arg.as_str());

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args(&args)
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let addr_path_verifier_arg = format!(
            "--addr-path={}/{}/addr_verifier.txt",
            test_dir, example_name
        );

        // deploy the verifier
        let mut args = vec![
            "deploy-evm-verifier",
            rpc_arg.as_str(),
            addr_path_verifier_arg.as_str(),
        ];

        args.push("--sol-code-path");
        args.push(sol_arg.as_str());

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args(&args)
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let sol_arg = format!("{}/{}/kzg.sol", test_dir, example_name);

        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "create-evm-da",
                &settings_arg,
                "--sol-code-path",
                sol_arg.as_str(),
                &srs_path,
                "--vk-path",
                &vk_arg,
                "-D",
                test_on_chain_data_path.as_str(),
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let addr_path_da_arg = format!("--addr-path={}/{}/addr_da.txt", test_dir, example_name);
        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "deploy-evm-da",
                format!("--settings-path={}", settings_path).as_str(),
                "-D",
                test_on_chain_data_path.as_str(),
                "--sol-code-path",
                sol_arg.as_str(),
                rpc_arg.as_str(),
                addr_path_da_arg.as_str(),
                private_key.as_str(),
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let pf_arg = format!("{}/{}/proof.pf", test_dir, example_name);
        // read in the verifier address
        let addr_verifier =
            std::fs::read_to_string(format!("{}/{}/addr_verifier.txt", test_dir, example_name))
                .expect("failed to read address file");

        let deployed_addr_verifier_arg = format!("--addr-verifier={}", addr_verifier);

        // read in the da address
        let addr_da = std::fs::read_to_string(format!("{}/{}/addr_da.txt", test_dir, example_name))
            .expect("failed to read address file");

        let deployed_addr_da_arg = format!("--addr-da={}", addr_da);

        let args = vec![
            "verify-evm",
            "--proof-path",
            pf_arg.as_str(),
            deployed_addr_verifier_arg.as_str(),
            deployed_addr_da_arg.as_str(),
            rpc_arg.as_str(),
        ];
        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args(&args)
            .status()
            .expect("failed to execute process");
        assert!(status.success());
        // Create a new set of test on chain data
        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args([
                "setup-test-evm-data",
                "-D",
                data_path.as_str(),
                "-M",
                &model_path,
                "--test-data",
                test_on_chain_data_path.as_str(),
                rpc_arg.as_str(),
                test_input_source.as_str(),
                test_output_source.as_str(),
            ])
            .status()
            .expect("failed to execute process");

        assert!(status.success());

        let deployed_addr_arg = format!("--addr={}", addr_da);

        let args = vec![
            "test-update-account-calls",
            deployed_addr_arg.as_str(),
            "-D",
            test_on_chain_data_path.as_str(),
            rpc_arg.as_str(),
        ];
        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args(&args)
            .status()
            .expect("failed to execute process");

        assert!(status.success());
        // As sanity check, add example that should fail.
        let args = vec![
            "verify-evm",
            "--proof-path",
            PF_FAILURE,
            deployed_addr_verifier_arg.as_str(),
            deployed_addr_da_arg.as_str(),
            rpc_arg.as_str(),
        ];
        let status = Command::new(format!("{}/release/ezkl", *CARGO_TARGET_DIR))
            .args(args)
            .status()
            .expect("failed to execute process");
        assert!(!status.success());
    }

    fn build_ezkl() {
        let status = Command::new("cargo")
            .args(["build", "--release", "--bin", "ezkl"])
            .status()
            .expect("failed to execute process");
        assert!(status.success());
    }

    #[allow(dead_code)]
    fn build_wasm_ezkl() {
        // wasm-pack build --target nodejs --out-dir ./tests/wasm/nodejs . -- -Z build-std="panic_abort,std"
        let status = Command::new("wasm-pack")
            .args([
                "build",
                "--release",
                "--target",
                "nodejs",
                "--out-dir",
                "./tests/wasm/nodejs",
                ".",
                "--",
                "-Z",
                "build-std=panic_abort,std",
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());
        // fix the memory size
        //   sed -i "3s|.*|imports['env'] = {memory: new WebAssembly.Memory({initial:20,maximum:65536,shared:true})}|" tests/wasm/nodejs/ezkl.js
        let status = Command::new("sed")
            .args([
                "-i",
                // is required on macos
                // "\".js\"",
                "3s|.*|imports['env'] = {memory: new WebAssembly.Memory({initial:20,maximum:65536,shared:true})}|",
                "./tests/wasm/nodejs/ezkl.js",
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());
    }
}
