use std::{collections::HashMap, marker::PhantomData, rc::Rc};

use crate::{
  gadgets::gadget::GadgetConfig,
  layers::{
    add::AddCircuit,
    avg_pool_2d::AvgPool2DCircuit,
    conv2d::Conv2DCircuit,
    layer::{Layer, LayerType},
    mul::MulCircuit,
  },
};
use ndarray::{Array, IxDyn};
use plonky2::{
  field::extension::Extendable, hash::hash_types::RichField, iop::target::Target,
  plonk::circuit_builder::CircuitBuilder,
};

use super::layer::LayerConfig;

#[derive(Clone, Debug, Default)]
pub struct DAGLayerConfig {
  pub ops: Vec<LayerConfig>,
  pub inp_idxes: Vec<Vec<usize>>,
  pub out_idxes: Vec<Vec<usize>>,
  pub final_out_idxes: Vec<usize>,
}

pub struct DAGLayerCircuit<F: RichField + Extendable<D>, const D: usize> {
  dag_config: DAGLayerConfig,
  _marker: PhantomData<F>,
}

impl<F: RichField + Extendable<D>, const D: usize> DAGLayerCircuit<F, D> {
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
  ) -> Vec<Array<Rc<Target>, IxDyn>> {
    // Tensor map
    let mut tensor_map = HashMap::new();
    for (idx, tensor) in tensors.iter().enumerate() {
      tensor_map.insert(idx, tensor.clone());
    }

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
      for v in &vec_inps {
        println!("shape: {:?}", v.shape());
      }
      let out = match layer_type {
        LayerType::Add => {
          let add_circuit = AddCircuit {};
          add_circuit.make_circuit(
            builder,
            &vec_inps,
            constants,
            gadget_config.clone(),
            &layer_config,
          )
        }
        LayerType::AvgPool2D => {
          let avg_pool_2d_circuit = AvgPool2DCircuit {};
          avg_pool_2d_circuit.make_circuit(
            builder,
            &vec_inps,
            constants,
            gadget_config.clone(),
            &layer_config,
          )
        }
        LayerType::Conv2D => {
          let conv_2d_circuit = Conv2DCircuit {
            config: layer_config.clone(),
          };
          conv_2d_circuit.make_circuit(
            builder,
            &vec_inps,
            constants,
            gadget_config.clone(),
            &layer_config,
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

    final_out
  }
}
