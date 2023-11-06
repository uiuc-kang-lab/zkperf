use std::{collections::HashMap, rc::Rc};

use ndarray::{Array, IxDyn};
use plonky2::{plonk::circuit_builder::CircuitBuilder, hash::hash_types::RichField, iop::target::Target};
use plonky2_field::extension::Extendable;

use crate::{gadgets::gadget::GadgetConfig, layers::layer::GadgetConsumer};

use crate::layers::layer::{Layer, LayerConfig};
pub struct NoopCircuit {}

impl<F: RichField + Extendable<D>, const D: usize> Layer<F, D> for NoopCircuit {
  fn make_circuit(
    &self,
    _builder: &mut CircuitBuilder<F, D>,
    tensors: &Vec<Array<Rc<Target>, IxDyn>>,
    _constants: &HashMap<i64, Rc<F>>,
    _gadget_config: Rc<GadgetConfig>,
    layer_config: &LayerConfig,
    _rand_targets: &mut Vec<Target>
  ) -> Vec<Array<Rc<Target>, IxDyn>> {
    let ret_idx = layer_config.layer_params[0] as usize;
    vec![tensors[ret_idx].clone()]
  }
}

impl GadgetConsumer for NoopCircuit {
  fn used_gadgets(&self, _layer_params: Vec<i64>) -> Vec<crate::gadgets::gadget::GadgetType> {
    vec![]
  }
}
