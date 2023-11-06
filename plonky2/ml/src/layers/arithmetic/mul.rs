use std::{collections::HashMap, rc::Rc, vec};

use ndarray::{Array, IxDyn};

use plonky2::{
  field::extension::Extendable, hash::hash_types::RichField, iop::target::Target,
  plonk::circuit_builder::CircuitBuilder,
};

use crate::{gadgets::{gadget::{GadgetConfig, Gadget, GadgetType}, var_div::DivRoundCircuit}, layers::layer::GadgetConsumer};

use super::super::super::layers::layer::{Layer, LayerConfig};

#[derive(Clone, Debug)]
pub struct MulCircuit {}

impl<F: RichField + Extendable<D>, const D: usize> Layer<F, D> for MulCircuit {
  fn make_circuit(
    &self,
    builder: &mut CircuitBuilder<F, D>,
    tensors: &Vec<Array<Rc<Target>, IxDyn>>,
    _constants: &HashMap<i64, Rc<F>>,
    gadget_config: Rc<GadgetConfig>,
    _layer_config: &LayerConfig,
    _rand_targets: &mut Vec<Target>
  ) -> Vec<Array<Rc<Target>, IxDyn>> {
    let inp = &tensors[0];
    let mul = &tensors[1];
    assert!(inp.shape()[3] == mul.shape()[0]);

    let flat_inp = Array::from_iter(inp.iter().cloned());

    let mut mul_outp = vec![];

    for i in 0..flat_inp.len() {
      mul_outp.push(builder.mul(*flat_inp[i], *mul[i % mul.len()]))
    }

    let mul_ref = mul_outp.iter().collect::<Vec<_>>();

    let div_gadget = DivRoundCircuit::construct(gadget_config.clone());
    let div_outp = div_gadget.make_circuit(
      builder,
      &vec![mul_ref],
      &vec![F::from_canonical_u64(gadget_config.scale_factor)],
      gadget_config,
    );

    let outp = div_outp.iter().map(|t| Rc::new(*t)).collect::<Vec<_>>();

    vec![Array::from_shape_vec(IxDyn(inp.shape()), outp).unwrap()]
  }
}

impl GadgetConsumer for MulCircuit {
  fn used_gadgets(&self, _layer_params: Vec<i64>) -> Vec<crate::gadgets::gadget::GadgetType> {
    vec![GadgetType::DivRound]
  }
}
