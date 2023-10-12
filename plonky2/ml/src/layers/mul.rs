use std::{collections::HashMap, rc::Rc, vec};

use ndarray::{Array, IxDyn};

use plonky2::{
  field::extension::Extendable, hash::hash_types::RichField, iop::target::Target,
  plonk::circuit_builder::CircuitBuilder,
};

use crate::gadgets::gadget::GadgetConfig;

use super::super::layers::layer::{Layer, LayerConfig};

#[derive(Clone, Debug)]
pub struct MulCircuit {}

impl<F: RichField + Extendable<D>, const D: usize> Layer<F, D> for MulCircuit {
  fn make_circuit(
    &self,
    builder: &mut CircuitBuilder<F, D>,
    tensors: &Vec<Array<Rc<Target>, IxDyn>>,
    _constants: &HashMap<i64, Rc<F>>,
    _gadget_config: Rc<GadgetConfig>,
    _layer_config: &LayerConfig,
  ) -> Vec<Array<Rc<Target>, IxDyn>> {
    let inp = &tensors[0];
    let mul = &tensors[1];
    assert!(inp.shape()[3] == mul.shape()[0]);

    let flat_inp = Array::from_iter(inp.iter().cloned());

    let mut outp = vec![];

    for i in 0..flat_inp.len() {
      outp.push(Rc::new(builder.mul(*flat_inp[i], *mul[i % mul.len()])))
    }

    vec![Array::from_shape_vec(IxDyn(inp.shape()), outp).unwrap()]
  }
}
