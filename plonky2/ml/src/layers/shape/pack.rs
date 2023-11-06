use std::{collections::HashMap, rc::Rc};

use ndarray::{Array, IxDyn, concatenate, Axis};
use plonky2::{plonk::circuit_builder::CircuitBuilder, hash::hash_types::RichField, iop::target::Target};
use plonky2_field::extension::Extendable;

use crate::{gadgets::gadget::{GadgetConfig, GadgetType}, layers::layer::GadgetConsumer};

use super::super::layer::{Layer, LayerConfig};

pub struct PackCircuit {}

impl<F: RichField + Extendable<D>, const D: usize> Layer<F, D> for PackCircuit {
  fn make_circuit(
    &self,
    _builder: &mut CircuitBuilder<F, D>,
    tensors: &Vec<Array<Rc<Target>, IxDyn>>,
    _constants: &HashMap<i64, Rc<F>>,
    _gadget_config: Rc<GadgetConfig>,
    layer_config: &LayerConfig,
    _rand_targets: &mut Vec<Target>
  ) -> Vec<Array<Rc<Target>, IxDyn>> {
    let axis = layer_config.layer_params[0] as usize;
    if axis > 1 {
      panic!("Pack only supports axis=0 or axis=1");
    }

    let expanded = tensors
      .into_iter()
      .map(|x| x.clone().insert_axis(Axis(axis)))
      .collect::<Vec<_>>();
    let views = expanded.iter().map(|x| x.view()).collect::<Vec<_>>();

    // TODO: in some cases, the pack is unnecessary. Simply return the first tensor in this case
    let out = concatenate(Axis(axis), views.as_slice()).unwrap_or(tensors[0].clone());

    vec![out]
  }
}

impl GadgetConsumer for PackCircuit {
  fn used_gadgets(&self, _layer_params: Vec<i64>) -> Vec<GadgetType> {
    vec![]
  }
}
