use clap::{Arg, App};
use plonky2::plonk::config::{GenericConfig, KeccakGoldilocksConfig};
use zkml::{model::ModelCircuit, utils::proving::time_circuit};

use jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn main() {
  env_logger::init();
  let matches = App::new("time_circuit")
    .arg(
      Arg::with_name("type")
        .index(1)
        .value_name("circuit_type")
        .help("circuit_name")
        .required(true),
    )
    .arg(
      Arg::with_name("config")
        .index(2)
        .value_name("config_fname")
        .help("config file path")
        .required(true),
    )
    .arg(
      Arg::with_name("input")
        .index(3)
        .value_name("inp_fname")
        .help("input file path")
        .required(true),
    )
    .arg(
      Arg::with_name("build_prove")
        .index(4)
        .value_name("build_prove")
        .help("build or prove")
        .required(true),
    )
    .arg(
      Arg::with_name("output")
        .index(5)
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
    .arg(
      Arg::with_name("no_lookups")
        .long("no-lookups")
        .value_name("no_lookups")
        .help("no lookups")
        .takes_value(false),
    )
    .get_matches();
  let circuit_type = matches.value_of("type").unwrap().to_string();
  let config_fname = matches.value_of("config").unwrap().to_string();
  let inp_fname = matches.value_of("input").unwrap().to_string();
  let build_prove = matches.value_of("build_prove").unwrap().to_string();
  let outp_json = matches.value_of("output").unwrap().to_string();

  let no_lookups = matches.is_present("no_lookups");
  if circuit_type != "mnist" && circuit_type != "dlrm" {
    panic!("Unsupported circuit type");
  }

  if build_prove != "build" && build_prove != "prove" {
    panic!("Must specify build or prove");
  }

  let col = if let Some(col) = matches.value_of("cols") {
    col.parse::<usize>().unwrap()
  } else {
    if circuit_type == "mnist" {
      60 as usize
    } else {
      110 as usize
    }
  };

  const D: usize = 2;
  type C = KeccakGoldilocksConfig;
  type F = <C as GenericConfig<D>>::F;
  let (circuit, builder, pw) =
    ModelCircuit::generate_from_file::<F, C, D>(&config_fname, &inp_fname, &col, no_lookups);
  time_circuit::<F, C, D>(circuit, builder, pw, circuit_type, build_prove, outp_json);
}
