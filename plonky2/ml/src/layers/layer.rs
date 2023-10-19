use std::{collections::HashMap, rc::Rc};

use ndarray::{Array, IxDyn};

use plonky2::{
  field::extension::Extendable, hash::hash_types::RichField, iop::target::Target,
  plonk::circuit_builder::CircuitBuilder,
};

use crate::gadgets::gadget::{GadgetConfig, GadgetType};

#[derive(Clone, Copy, Debug, Default, Hash, Eq, PartialEq)]
pub enum LayerType {
  // TODO hopefully this default never triggers
  Add,
  AvgPool2D,
  Concatenation,
  Conv2D,
  FullyConnected,
  Logistic,
  Gather,
  Mul,
  #[default]
  Noop,
  Pack,
  Reshape,
  Split,
  Transpose,
}

// NOTE: This is the same order as the TFLite schema
// Must not be changed
#[derive(Clone, Debug, Default, Hash, Eq, PartialEq)]
pub enum ActivationType {
  #[default]
  None,
  Relu6,
  Relu,
}

#[derive(Clone, Debug, Default)]
pub struct LayerConfig {
  pub layer_type: LayerType,
  pub layer_params: Vec<i64>, // This is turned into layer specific configurations at runtime
  pub inp_shapes: Vec<Vec<usize>>,
  pub out_shapes: Vec<Vec<usize>>,
  pub mask: Vec<i64>,
}

// General issue with rust: I'm not sure how to pass named arguments to a trait...
// Currently, the caller must be aware of the order of the tensors and results
pub trait Layer<F: RichField + Extendable<D>, const D: usize> {
  fn make_circuit(
    &self,
    builder: &mut CircuitBuilder<F, D>,
    tensors: &Vec<Array<Rc<Target>, IxDyn>>,
    constants: &HashMap<i64, Rc<F>>,
    gadget_config: Rc<GadgetConfig>,
    layer_config: &LayerConfig,
  ) -> Vec<Array<Rc<Target>, IxDyn>>;
}

pub trait GadgetConsumer {
  fn used_gadgets(&self, layer_params: Vec<i64>) -> Vec<GadgetType>;
}
