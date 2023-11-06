use std::{collections::HashMap, rc::Rc};

use ndarray::{Array, IxDyn};
use plonky2::{plonk::circuit_builder::CircuitBuilder, hash::hash_types::RichField, iop::target::Target};
use plonky2_field::extension::Extendable;

use crate::{gadgets::gadget::GadgetConfig, layers::layer::GadgetConsumer};

use super::super::layer::{Layer, LayerConfig};

pub struct ReshapeCircuit {}

impl<F: RichField + Extendable<D>, const D: usize> Layer<F, D> for ReshapeCircuit {
  fn make_circuit(
    &self,
    _builder: &mut CircuitBuilder<F, D>,
    tensors: &Vec<Array<Rc<Target>, IxDyn>>,
    _constants: &HashMap<i64, Rc<F>>,
    _gadget_config: Rc<GadgetConfig>,
    layer_config: &LayerConfig,
    _rand_targets: &mut Vec<Target>
  ) -> Vec<Array<Rc<Target>, IxDyn>> {
    let inp = &tensors[0];
    let shape = layer_config.out_shapes[0].clone();

    println!("Reshape: {:?} -> {:?}", inp.shape(), shape);
    let flat = inp.iter().map(|x| x.clone()).collect();
    let out = Array::from_shape_vec(shape, flat).unwrap();
    vec![out]
  }
}

impl GadgetConsumer for ReshapeCircuit {
  fn used_gadgets(&self, _layer_params: Vec<i64>) -> Vec<crate::gadgets::gadget::GadgetType> {
    vec![]
  }
}
