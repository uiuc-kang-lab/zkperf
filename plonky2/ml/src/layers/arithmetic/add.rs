use std::{collections::HashMap, rc::Rc, vec};

use ndarray::{Array, IxDyn};

use plonky2::{
  field::extension::Extendable,
  hash::hash_types::RichField,
  iop::target::Target,
  plonk::circuit_builder::CircuitBuilder,
};

use crate::{gadgets::gadget::GadgetConfig, layers::layer::GadgetConsumer};

use super::super::super::layers::layer::{Layer, LayerConfig};

#[derive(Clone, Debug)]
pub struct AddCircuit {}

impl<F: RichField + Extendable<D>, const D: usize> Layer<F, D> for AddCircuit {
  fn make_circuit(
    &self,
    builder: &mut CircuitBuilder<F, D>,
    tensors: &Vec<Array<Rc<Target>, IxDyn>>,
    _constants: &HashMap<i64, Rc<F>>,
    _gadget_config: Rc<GadgetConfig>,
    _layer_config: &LayerConfig,
    _rand_targets: &mut Vec<Target>
  ) -> Vec<Array<Rc<Target>, IxDyn>> {
    let inp = &tensors[0];
    let add = &tensors[1];
    assert!(inp.shape()[3] == add.shape()[0]);

    let flat_inp = Array::from_iter(inp.iter().cloned());

    let mut outp = vec![];

    for i in 0..flat_inp.len() {
      outp.push(Rc::new(builder.add(*flat_inp[i], *add[i % add.len()])))
    }

    vec![Array::from_shape_vec(IxDyn(inp.shape()), outp).unwrap()]
  }
}

impl GadgetConsumer for AddCircuit {
  fn used_gadgets(&self, _layer_params: Vec<i64>) -> Vec<crate::gadgets::gadget::GadgetType> {
    vec![]
  }
}
