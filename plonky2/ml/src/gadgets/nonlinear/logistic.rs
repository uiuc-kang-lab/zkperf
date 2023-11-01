use std::{rc::Rc, collections::HashMap};

use plonky2::field::extension::Extendable;
use plonky2::hash::hash_types::RichField;
use plonky2::iop::target::Target;
use plonky2::plonk::circuit_builder::CircuitBuilder;

type LogisticConfig = GadgetConfig;

use super::{
  super::gadget::{Gadget, GadgetConfig, GadgetType},
  non_linearity::NonLinearGadget,
};

pub struct LogisticGadgetCircuit {
  config: Rc<LogisticConfig>,
}

impl LogisticGadgetCircuit {
  pub fn construct(config: Rc<LogisticConfig>) -> Self {
    Self { config }
  }

  pub fn configure<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    gadget_config: GadgetConfig,
  ) -> GadgetConfig {
    <LogisticGadgetCircuit as NonLinearGadget::<F, D>>::configure(builder, gadget_config, GadgetType::Logistic)
  }
}

impl<F: RichField + Extendable<D>, const D: usize> NonLinearGadget<F, D> for LogisticGadgetCircuit {
  fn generate_map(scale_factor: u64, min_val: i64, num_rows: i64) -> HashMap<i64, i64> {
    let mut map = HashMap::new();
    for i in 0..num_rows {
      let shifted = i + min_val;
      let x = (shifted as f64) / (scale_factor as f64);
      let logistic = 1. / (1. + (-x).exp());
      let logistic = (logistic * ((scale_factor) as f64)).round() as i64;
      map.insert(i as i64, logistic);
    }

    map
  }

  fn get_map(&self) -> &HashMap<i64, i64> {
    &self.config.maps.get(&GadgetType::Logistic).unwrap()[0]
  }
}

impl<F: RichField + Extendable<D>, const D: usize> Gadget<F, D> for LogisticGadgetCircuit {
  fn load_lookups(builder: &mut CircuitBuilder<F, D>, config: GadgetConfig) -> Option<usize> {
    <LogisticGadgetCircuit as NonLinearGadget::<F, D>>::load_lookups(builder, Rc::new(config.clone()), GadgetType::Logistic)
  }

  fn make_circuit(
    &self,
    builder: &mut CircuitBuilder<F, D>,
    vec_inputs: &Vec<Vec<&Target>>,
    _single_inputs: &Vec<F>,
    gadget_config: Rc<GadgetConfig>,
  ) -> Vec<Target> {
    NonLinearGadget::make_circuit(self, builder, vec_inputs, gadget_config, GadgetType::Logistic)
  }
}
