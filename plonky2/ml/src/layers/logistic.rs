use std::{collections::HashMap, rc::Rc};

use ndarray::{Array, IxDyn};
use plonky2::{plonk::circuit_builder::CircuitBuilder, hash::hash_types::RichField, iop::target::Target};
use plonky2_field::extension::Extendable;

use crate::gadgets::gadget::{Gadget, GadgetType};
use crate::gadgets::nonlinear::logistic::LogisticGadgetCircuit;
use crate::{gadgets::gadget::GadgetConfig, layers::layer::GadgetConsumer};

use crate::layers::layer::{Layer, LayerConfig};
#[derive(Clone, Debug)]
pub struct LogisticCircuit {}

impl<F: RichField + Extendable<D>, const D: usize> Layer<F, D> for LogisticCircuit {
  fn make_circuit(
    &self,
    builder: &mut CircuitBuilder<F, D>,
    tensors: &Vec<Array<Rc<Target>, IxDyn>>,
    _constants: &HashMap<i64, Rc<F>>,
    gadget_config: Rc<GadgetConfig>,
    _layer_config: &LayerConfig,
    _rand_targets: &mut Vec<Target>
  ) -> Vec<Array<Rc<Target>, IxDyn>> {    let inp = &tensors[0];
    let inp_vec = inp.iter().map(|x| x.as_ref()).collect::<Vec<_>>();

    let logistic_gadget = LogisticGadgetCircuit::construct(gadget_config.clone());
    let vec_inps = vec![inp_vec];
    let out = logistic_gadget.make_circuit(
      builder,
      &vec_inps,
      &vec![],
      gadget_config,
    );

    let out = out.into_iter().map(|x| Rc::new(x)).collect::<Vec<_>>();
    let out = Array::from_shape_vec(IxDyn(inp.shape()), out).unwrap();

    vec![out]
  }
}

impl GadgetConsumer for LogisticCircuit {
  fn used_gadgets(&self, _layer_params: Vec<i64>) -> Vec<crate::gadgets::gadget::GadgetType> {
    vec![GadgetType::Logistic]
  }
}
