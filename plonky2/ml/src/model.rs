use std::{
  collections::{BTreeMap, BTreeSet, HashMap},
  rc::Rc,
  sync::{Arc, Mutex}, marker::PhantomData,
};

use lazy_static::lazy_static;
use ndarray::{Array, IxDyn};

use plonky2::iop::witness::PartialWitness;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::{field::extension::Extendable, plonk::circuit_data::CircuitConfig};
use plonky2::{hash::hash_types::RichField, iop::witness::WitnessWrite};
use plonky2::{iop::target::Target, plonk::config::GenericConfig};

use crate::{
  gadgets::{
    bias_div_round_relu6::BiasDivRoundRelu6Circuit,
    dot_prod::DotProductCircuit,
    gadget::{GadgetConfig, GadgetType},
    input_lookup::InputLookupCircuit,
    nonlinear::{logistic::LogisticGadgetCircuit, relu::ReluCircuit},
    var_div::DivRoundCircuit,
  },
  layers::{
    arithmetic::{add::AddCircuit, mul::MulCircuit},
    avg_pool_2d::AvgPool2DCircuit,
    batch_mat_mul::BatchMatMulCircuit,
    conv2d::Conv2DCircuit,
    dag::{DAGLayerCircuit, DAGLayerConfig},
    fully_connected::{FullyConnectedCircuit, FullyConnectedConfig},
    layer::{GadgetConsumer, LayerConfig, LayerType},
    logistic::LogisticCircuit,
    noop::NoopCircuit,
    shape::{
      concatenation::ConcatenationCircuit, gather::GatherCircuit, pack::PackCircuit,
      reshape::ReshapeCircuit, split::SplitCircuit, transpose::TransposeCircuit,
    },
  },
  utils::loader::{load_model_msgpack, ModelMsgpack},
};

lazy_static! {
  pub static ref GADGET_CONFIG: Mutex<GadgetConfig> = Mutex::new(GadgetConfig::default());
}
#[derive(Clone, Debug, Default)]
pub struct ModelCircuit {
  pub used_gadgets: Arc<BTreeSet<GadgetType>>,
  pub dag_config: DAGLayerConfig,
  pub tensors: BTreeMap<i64, Array<Rc<Target>, IxDyn>>,
  pub k: usize,
  pub bits_per_elem: usize,
  pub inp_idxes: Vec<i64>,
}

impl ModelCircuit {
  pub fn tensor_map_to_vec(
    &self,
    tensor_map: &BTreeMap<i64, Array<Rc<Target>, IxDyn>>,
  ) -> Vec<Array<Rc<Target>, IxDyn>> {
    let smallest_tensor = tensor_map
      .iter()
      .min_by_key(|(_, tensor)| tensor.len())
      .unwrap()
      .1;
    let max_tensor_key = tensor_map
      .iter()
      .max_by_key(|(key, _)| *key)
      .unwrap()
      .0
      .clone();
    let mut tensors = vec![];
    for i in 0..max_tensor_key + 1 {
      let tensor = tensor_map.get(&i).unwrap_or(smallest_tensor);
      tensors.push(tensor.clone());
    }

    tensors
  }

  pub fn assign_constants<F: RichField + Extendable<D>, const D: usize>(
    &self,
    gadget_config: Rc<GadgetConfig>,
  ) -> HashMap<i64, Rc<F>> {
    let sf = gadget_config.scale_factor;
    let min_val = gadget_config.min_val;
    let max_val = gadget_config.max_val;

    let mut constants: HashMap<i64, Rc<F>> = HashMap::new();

    let vals = vec![0 as i64, 1, sf as i64, min_val, max_val];
    let shift_val_i64 = -min_val * 2; // FIXME
    let shift_val_f = F::from_canonical_u64(shift_val_i64 as u64);
    for val in vals {
      constants.insert(
        val,
        Rc::new(F::from_canonical_u64((val + shift_val_i64) as u64) - shift_val_f),
      );
    }
    constants
  }

  pub fn generate_from_file<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize>(
    config_file: &str,
    inp_file: &str,
    sweep_variable: &usize,
  ) -> (ModelCircuit, CircuitBuilder<F, D>, PartialWitness<F>) {
    let config = load_model_msgpack(config_file, inp_file);
    Self::generate_from_msgpack::<F, C, D>(config, true, sweep_variable)
  }

  pub fn generate_from_msgpack<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize>(
    config: ModelMsgpack,
    panic_empty_tensor: bool,
    sweep_variable: &usize
  ) -> (ModelCircuit, CircuitBuilder<F, D>, PartialWitness<F>) {
    let to_field = |x: i64| {
      let bias = 1 << 31;
      let x_pos = x + bias;
      F::from_canonical_u64(x_pos as u64) - F::from_canonical_u64(bias as u64)
    };

    let match_layer = |x: &str| match x {
      "Add" => LayerType::Add,
      "AveragePool2D" => LayerType::AvgPool2D,
      "BatchMatMul" => LayerType::BatchMatMul,
      "Concatenation" => LayerType::Concatenation,
      "Conv2D" => LayerType::Conv2D,
      "FullyConnected" => LayerType::FullyConnected,
      "Gather" => LayerType::Gather,
      "Logistic" => LayerType::Logistic,
      "Mul" => LayerType::Mul,
      "Noop" => LayerType::Noop,
      "Pack" => LayerType::Pack,
      "Reshape" => LayerType::Reshape,
      "Split" => LayerType::Split,
      "Transpose" => LayerType::Transpose,
      _ => panic!("unknown op: {}", x),
    };

    if *sweep_variable < 25{
      panic!("Invalid Sweep Variable")
    }

    let mnist_config: CircuitConfig = CircuitConfig {
      num_constants: 3,
      num_routed_wires: *sweep_variable,
      ..CircuitConfig::standard_recursion_zk_config()
    };
    let mut builder = CircuitBuilder::<F, D>::new(mnist_config);
    let mut pw = PartialWitness::<F>::new();

    let mut tensors = BTreeMap::new();
    for flat in config.tensors {
      let value_flat = flat.data.iter().map(|x| to_field(*x)).collect::<Vec<_>>();
      let target_flat = builder.add_virtual_targets(flat.data.len());
      let inp_idxes = config.inp_idxes.clone();
      let is_public = inp_idxes
        .into_iter()
        .find(|&x| x == flat.idx as i64)
        .is_some();
      for j in 0..target_flat.len() {
        // for now set inp idxs to be public
        pw.set_target(target_flat[j], value_flat[j]);
        if is_public {
          builder.register_public_input(target_flat[j]);
        }
      }
      let target_flat = target_flat
        .iter()
        .map(|t| Rc::new(t.clone()))
        .collect::<Vec<_>>();
      let shape = flat.shape.iter().map(|x| *x as usize).collect::<Vec<_>>();
      let num_el: usize = shape.iter().product();
      if panic_empty_tensor && num_el != value_flat.len() {
        panic!("tensor shape and data length mismatch");
      }
      if num_el == target_flat.len() {
        let tensor = Array::from_shape_vec(IxDyn(&shape), target_flat).unwrap();
        tensors.insert(flat.idx, tensor);
      } else {
        // Do nothing here since we're loading the config
      };
    }

    let i64_to_usize = |x: &Vec<i64>| x.iter().map(|x| *x as usize).collect::<Vec<_>>();

    let mut used_gadgets = BTreeSet::new();

    let dag_config = {
      let ops = config
        .layers
        .iter()
        .map(|layer| {
          let layer_type = match_layer(&layer.layer_type);
          let layer_gadgets = match layer_type {
            LayerType::Add => Box::new(AddCircuit {}) as Box<dyn GadgetConsumer>,
            LayerType::AvgPool2D => Box::new(AvgPool2DCircuit::<F, C, D> {
              _marker: PhantomData,
            }) as Box<dyn GadgetConsumer>,
            LayerType::BatchMatMul => Box::new(BatchMatMulCircuit::<F, C, D> {
              _marker: PhantomData,
            }) as Box<dyn GadgetConsumer>,
            LayerType::Concatenation => {
              Box::new(ConcatenationCircuit {}) as Box<dyn GadgetConsumer>
            }
            LayerType::Conv2D => Box::new(Conv2DCircuit::<F, C, D> {
              config: LayerConfig::default(),
              _marker: PhantomData,
            }) as Box<dyn GadgetConsumer>,
            LayerType::FullyConnected => Box::new(FullyConnectedCircuit::<F, C, D> {
              config: FullyConnectedConfig { normalize: true },
              _marker: PhantomData,
            }) as Box<dyn GadgetConsumer>,
            LayerType::Gather => Box::new(GatherCircuit {}) as Box<dyn GadgetConsumer>,
            LayerType::Logistic => Box::new(LogisticCircuit {}) as Box<dyn GadgetConsumer>,
            LayerType::Mul => Box::new(MulCircuit {}) as Box<dyn GadgetConsumer>,
            LayerType::Noop => Box::new(NoopCircuit {}) as Box<dyn GadgetConsumer>,
            LayerType::Pack => Box::new(PackCircuit {}) as Box<dyn GadgetConsumer>,
            LayerType::Reshape => Box::new(ReshapeCircuit {}) as Box<dyn GadgetConsumer>,
            LayerType::Split => Box::new(SplitCircuit {}) as Box<dyn GadgetConsumer>,
            LayerType::Transpose => Box::new(TransposeCircuit {}) as Box<dyn GadgetConsumer>,
          }
          .used_gadgets(layer.params.clone());
          for gadget in layer_gadgets {
            used_gadgets.insert(gadget);
          }
          LayerConfig {
            layer_type,
            layer_params: layer.params.clone(),
            inp_shapes: layer.inp_shapes.iter().map(|x| i64_to_usize(x)).collect(),
            out_shapes: layer.out_shapes.iter().map(|x| i64_to_usize(x)).collect(),
            mask: layer.mask.clone(),
          }
        })
        .collect::<Vec<_>>();
      let inp_idxes = config
        .layers
        .iter()
        .map(|layer| i64_to_usize(&layer.inp_idxes))
        .collect::<Vec<_>>();
      let out_idxes = config
        .layers
        .iter()
        .map(|layer| i64_to_usize(&layer.out_idxes))
        .collect::<Vec<_>>();
      let final_out_idxes = config
        .out_idxes
        .iter()
        .map(|x| *x as usize)
        .collect::<Vec<_>>();
      DAGLayerConfig {
        inp_idxes,
        out_idxes,
        ops,
        final_out_idxes,
      }
    };

    used_gadgets.insert(GadgetType::InputLookup);

    // Make luts
    let og_gadget_config = crate::model::GADGET_CONFIG.lock().unwrap().clone();
    let gadget = &GADGET_CONFIG;
    *gadget.lock().unwrap() = GadgetConfig {
      scale_factor: config.global_sf as u64,
      shift_min_val: -(config.global_sf * config.global_sf * (1 << 17)),
      div_outp_min_val: -(1 << (config.k - 1)),
      min_val: -(1 << (config.k - 1)),
      max_val: (1 << (config.k - 1)) - 10,
      k: config.k as usize,
      num_rows: (1 << config.k) - 10 + 1,
      num_cols: config.num_cols as usize,
      num_bits_per_elem: config.bits_per_elem.unwrap_or(config.k),
      ..og_gadget_config
    };
    let mut gadget_config = crate::model::GADGET_CONFIG.lock().unwrap().clone();
    gadget_config = InputLookupCircuit::configure(&mut builder, gadget_config);
    for gadget_type in used_gadgets.iter() {
      gadget_config = match gadget_type {
        GadgetType::BiasDivRoundRelu6 => {
          BiasDivRoundRelu6Circuit::configure(&mut builder, gadget_config)
        }
        GadgetType::DivRound => DivRoundCircuit::configure(&mut builder, gadget_config),
        GadgetType::DotProduct => DotProductCircuit::configure(&mut builder, gadget_config),
        GadgetType::Logistic => LogisticGadgetCircuit::configure(&mut builder, gadget_config),
        GadgetType::Relu => ReluCircuit::configure(&mut builder, gadget_config),
        GadgetType::InputLookup => gadget_config, // This is always loaded
      };
    }

    let used_gadgets = Arc::new(used_gadgets);
    let gadget = &GADGET_CONFIG;
    *gadget.lock().unwrap() = GadgetConfig {
      used_gadgets: used_gadgets.clone(),
      ..gadget_config
    };

    (
      ModelCircuit {
        used_gadgets,
        tensors,
        dag_config,
        k: config.k as usize,
        bits_per_elem: config.bits_per_elem.unwrap_or(config.k) as usize,
        inp_idxes: config.inp_idxes.clone(),
      },
      builder,
      pw,
    )
  }

  pub fn construct<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>  + 'static, const D: usize>(
    &self,
    builder: &mut CircuitBuilder<F, D>,
  ) -> (Vec<Array<Rc<Target>, IxDyn>>, Vec<Target>) {
    let gadget = &GADGET_CONFIG;
    let cloned_gadget = gadget.lock().unwrap().clone();
    let constants = self.assign_constants(Rc::new(cloned_gadget.clone()));

    // make the circuit
    let tensors_vec = self.tensor_map_to_vec(&self.tensors);
    let dag_circuit = DAGLayerCircuit::<F, C, D>::construct(self.dag_config.clone());
    dag_circuit.make_circuit(
      builder,
      &tensors_vec,
      &constants,
      Rc::new(cloned_gadget),
      &LayerConfig::default(),
    )
  }
}
