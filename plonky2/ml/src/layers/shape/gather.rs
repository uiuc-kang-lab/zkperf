use std::{collections::HashMap, rc::Rc};

use ndarray::{Array, IxDyn, Axis};
use plonky2::{plonk::circuit_builder::CircuitBuilder, hash::hash_types::RichField, iop::target::Target};
use plonky2_field::extension::Extendable;

use crate::{gadgets::gadget::GadgetConfig, layers::layer::GadgetConsumer};

use super::super::layer::{Layer, LayerConfig};

pub struct GatherCircuit {}

impl<F: RichField + Extendable<D>, const D: usize> Layer<F, D> for GatherCircuit {
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
    let axis = layer_config.layer_params[0] as usize;
    let pos = &layer_config
      .layer_params[1..layer_config.layer_params.len()]
      .iter()
      .map(|x| *x as usize)
      .collect::<Vec<_>>()[..];

    // TODO not sure if this assumes that pos are unique
    let out = inp.select(Axis(axis), &pos);

    vec![out]
  }
}

impl GadgetConsumer for GatherCircuit {
  fn used_gadgets(&self, _layer_params: Vec<i64>) -> Vec<crate::gadgets::gadget::GadgetType> {
    vec![]
  }
}
