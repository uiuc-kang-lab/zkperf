use std::{collections::HashMap, rc::Rc};

use ndarray::{Array, IxDyn};
use plonky2::{plonk::circuit_builder::CircuitBuilder, hash::hash_types::RichField, iop::target::Target};
use plonky2_field::extension::Extendable;

use crate::{gadgets::gadget::GadgetConfig, layers::layer::GadgetConsumer};

use super::super::layer::{Layer, LayerConfig};

pub struct TransposeCircuit {}

impl<F: RichField + Extendable<D>, const D: usize> Layer<F, D> for TransposeCircuit {
  fn make_circuit(
    &self,
    _builder: &mut CircuitBuilder<F, D>,
    tensors: &Vec<Array<Rc<Target>, IxDyn>>,
    _constants: &HashMap<i64, Rc<F>>,
    _gadget_config: Rc<GadgetConfig>,
    layer_config: &LayerConfig,
  ) -> Vec<Array<Rc<Target>, IxDyn>> {
    assert_eq!(layer_config.layer_params.len() % 2, 0);
    let ndim = layer_config.layer_params.len() / 2;
    let inp_shape = layer_config.layer_params[0..ndim]
      .to_vec()
      .iter()
      .map(|x| *x as usize)
      .collect::<Vec<_>>();
    let permutation = layer_config.layer_params[ndim..]
      .to_vec()
      .iter()
      .map(|x| *x as usize)
      .collect::<Vec<_>>();

    let inp = &tensors[0];
    // Required because of memory layout issues
    let inp_flat = inp.iter().cloned().collect::<Vec<_>>();
    let inp = Array::from_shape_vec(IxDyn(&inp_shape), inp_flat).unwrap();

    let inp = inp.permuted_axes(IxDyn(&permutation));

    vec![inp]
  }
}

impl GadgetConsumer for TransposeCircuit {
  fn used_gadgets(&self, _layer_params: Vec<i64>) -> Vec<crate::gadgets::gadget::GadgetType> {
    vec![]
  }
}
