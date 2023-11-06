use std::{collections::HashMap, marker::PhantomData, rc::Rc};

use crate::{
  gadgets::gadget::GadgetConfig,
  layers::{
    arithmetic::add::AddCircuit,
    arithmetic::mul::MulCircuit,
    avg_pool_2d::AvgPool2DCircuit,
    conv2d::Conv2DCircuit,
    layer::{Layer, LayerType}, shape::{concatenation::ConcatenationCircuit, reshape::ReshapeCircuit, transpose::TransposeCircuit, pack::PackCircuit, split::SplitCircuit, gather::GatherCircuit}, fully_connected::{FullyConnectedCircuit, FullyConnectedConfig}, noop::NoopCircuit, logistic::LogisticCircuit, batch_mat_mul::BatchMatMulCircuit,
  },
};
use ndarray::{Array, IxDyn};
use plonky2::{
  field::extension::Extendable, hash::hash_types::RichField, iop::target::Target,
  plonk::{circuit_builder::CircuitBuilder, config::GenericConfig},
};

use super::layer::LayerConfig;

#[derive(Clone, Debug, Default)]
pub struct DAGLayerConfig {
  pub ops: Vec<LayerConfig>,
  pub inp_idxes: Vec<Vec<usize>>,
  pub out_idxes: Vec<Vec<usize>>,
  pub final_out_idxes: Vec<usize>,
}

pub struct DAGLayerCircuit<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize> {
  dag_config: DAGLayerConfig,
  _marker: PhantomData<C>,
}

impl<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>  + 'static, const D: usize> DAGLayerCircuit<F, C, D> {
  pub fn construct(dag_config: DAGLayerConfig) -> Self {
    Self {
      dag_config,
      _marker: PhantomData,
    }
  }

  // IMPORTANT: Assumes input tensors are in order. Output tensors can be in any order.
  pub fn make_circuit(
    &self,
    builder: &mut CircuitBuilder<F, D>,
    tensors: &Vec<Array<Rc<Target>, IxDyn>>,
    constants: &HashMap<i64, Rc<F>>,
    gadget_config: Rc<GadgetConfig>,
    _layer_config: &LayerConfig,
  ) -> (Vec<Array<Rc<Target>, IxDyn>>, Vec<Target>) {
    // Tensor map
    let mut tensor_map = HashMap::new();
    for (idx, tensor) in tensors.iter().enumerate() {
      tensor_map.insert(idx, tensor.clone());
    }
    let mut rand_targets = vec![];

    // Compute the dag
    for (layer_idx, layer_config) in self.dag_config.ops.iter().enumerate() {
      let layer_type = &layer_config.layer_type;
      let inp_idxes = &self.dag_config.inp_idxes[layer_idx];
      let out_idxes = &self.dag_config.out_idxes[layer_idx];
      println!(
        "Processing layer {}, type: {:?}, inp_idxes: {:?}, out_idxes: {:?}, layer_params: {:?}",
        layer_idx, layer_type, inp_idxes, out_idxes, layer_config.layer_params
      );
      let vec_inps = inp_idxes
        .iter()
        .map(|idx| tensor_map.get(idx).unwrap().clone())
        .collect::<Vec<_>>();
      let out = match layer_type {
        LayerType::Add => {
          let add_circuit = AddCircuit {};
          add_circuit.make_circuit(
            builder,
            &vec_inps,
            constants,
            gadget_config.clone(),
            &layer_config,
            &mut rand_targets
          )
        }
        LayerType::AvgPool2D => {
          let avg_pool_2d_circuit = AvgPool2DCircuit::<F, C, D> {
            _marker: PhantomData,
          };
          avg_pool_2d_circuit.make_circuit(
            builder,
            &vec_inps,
            constants,
            gadget_config.clone(),
            &layer_config,
            &mut rand_targets
          )
        }
        LayerType::BatchMatMul => {
          let batch_mat_mul_circuit = BatchMatMulCircuit::<F, C, D> {
            _marker: PhantomData,
          };
          batch_mat_mul_circuit.make_circuit(
            builder,
            &vec_inps,
            constants,
            gadget_config.clone(),
            &layer_config,
            &mut rand_targets
          )
        }
        LayerType::Concatenation => {
          let concat_circuit = ConcatenationCircuit {};
          concat_circuit.make_circuit(
            builder,
            &vec_inps,
            constants,
            gadget_config.clone(),
            &layer_config,
            &mut rand_targets
          )
        }
        LayerType::Conv2D => {
          let conv_2d_circuit = Conv2DCircuit::<F, C, D> {
            config: layer_config.clone(),
            _marker: PhantomData,
          };
          conv_2d_circuit.make_circuit(
            builder,
            &vec_inps,
            constants,
            gadget_config.clone(),
            &layer_config,
            &mut rand_targets
          )
        }
        LayerType::FullyConnected => {
          let fc_circuit = FullyConnectedCircuit::<F, C, D> {
            config: FullyConnectedConfig::construct(true),
            _marker: PhantomData,
          };
          fc_circuit.make_circuit(
            builder,
            &vec_inps,
            constants,
            gadget_config.clone(),
            &layer_config,
            &mut rand_targets
          )
        }
        LayerType::Gather => {
          let gather_circuit = GatherCircuit {};
          gather_circuit.make_circuit(
            builder,
            &vec_inps,
            constants,
            gadget_config.clone(),
            &layer_config,
            &mut rand_targets
          )
        }
        LayerType::Logistic => {
          let logistic_circuit = LogisticCircuit {};
          logistic_circuit.make_circuit(
            builder,
            &vec_inps,
            constants,
            gadget_config.clone(),
            &layer_config,
            &mut rand_targets
          )
        }
        LayerType::Mul => {
          let mul_circuit = MulCircuit {};
          mul_circuit.make_circuit(
            builder,
            &vec_inps,
            constants,
            gadget_config.clone(),
            &layer_config,
            &mut rand_targets
          )
        }
        LayerType::Noop => {
          let noop_circuit = NoopCircuit {};
          noop_circuit.make_circuit(
            builder,
            &vec_inps,
            constants,
            gadget_config.clone(),
            &layer_config,
            &mut rand_targets
          )
        }
        LayerType::Pack => {
          let pack_circuit = PackCircuit {};
          pack_circuit.make_circuit(
            builder,
            &vec_inps,
            constants,
            gadget_config.clone(),
            &layer_config,
            &mut rand_targets
          )
        }
        LayerType::Reshape => {
          let reshape_circuit = ReshapeCircuit {};
          reshape_circuit.make_circuit(
            builder,
            &vec_inps,
            constants,
            gadget_config.clone(),
            &layer_config,
            &mut rand_targets
          )
        }
        LayerType::Split => {
          let split_circuit = SplitCircuit {};
          split_circuit.make_circuit(
            builder,
            &vec_inps,
            constants,
            gadget_config.clone(),
            &layer_config,
            &mut rand_targets
          )
        }
        LayerType::Transpose => {
          let transpose_circuit = TransposeCircuit {};
          transpose_circuit.make_circuit(
            builder,
            &vec_inps,
            constants,
            gadget_config.clone(),
            &layer_config,
            &mut rand_targets
          )
        }
      };

      for (idx, tensor_idx) in out_idxes.iter().enumerate() {
        println!("Out {} shape: {:?}", idx, out[idx].shape());
        tensor_map.insert(*tensor_idx, out[idx].clone());
      }
      println!();
    }

    let mut final_out = vec![];
    for idx in self.dag_config.final_out_idxes.iter() {
      final_out.push(tensor_map.get(idx).unwrap().clone());
    }

    (final_out, rand_targets)
  }
}
