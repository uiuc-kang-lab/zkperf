use std::{collections::HashMap, rc::Rc};

use ndarray::{Array, IxDyn, Slice, Axis};
use plonky2::{plonk::circuit_builder::CircuitBuilder, hash::hash_types::RichField, iop::target::Target};
use plonky2_field::extension::Extendable;

use crate::gadgets::gadget::GadgetType;
use crate::{gadgets::gadget::GadgetConfig, layers::layer::GadgetConsumer};

use super::super::layer::{Layer, LayerConfig};

pub struct SplitCircuit {}

impl<F: RichField + Extendable<D>, const D: usize> Layer<F, D> for SplitCircuit {
  fn make_circuit(
    &self,
    _builder: &mut CircuitBuilder<F, D>,
    tensors: &Vec<Array<Rc<Target>, IxDyn>>,
    _constants: &HashMap<i64, Rc<F>>,
    _gadget_config: Rc<GadgetConfig>,
    layer_config: &LayerConfig,
  ) -> Vec<Array<Rc<Target>, IxDyn>> {
    let axis = layer_config.layer_params[0] as usize;
    let num_splits = layer_config.layer_params[1] as usize;
    let inp = &tensors[1];

    let mut out = vec![];
    let split_len = inp.shape()[axis] / num_splits;
    for i in 0..num_splits {
      let slice = inp
        .slice_axis(
          Axis(axis),
          Slice::from((i * split_len)..((i + 1) * split_len)),
        )
        .to_owned();
      out.push(slice.to_owned());
    }
    out
  }
}

impl GadgetConsumer for SplitCircuit {
  fn used_gadgets(&self, _layer_params: Vec<i64>) -> Vec<GadgetType> {
    vec![]
  }
}
