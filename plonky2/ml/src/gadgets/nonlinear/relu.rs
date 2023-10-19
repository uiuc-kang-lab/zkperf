use std::{rc::Rc, collections::HashMap};

use plonky2::field::extension::Extendable;
use plonky2::hash::hash_types::RichField;
use plonky2::iop::target::Target;
use plonky2::plonk::circuit_builder::CircuitBuilder;

type ReluConfig = GadgetConfig;

use super::{
  super::gadget::{Gadget, GadgetConfig, GadgetType},
  non_linearity::NonLinearGadget,
};

pub struct ReluCircuit {
  config: Rc<ReluConfig>,
}

impl ReluCircuit {
  pub fn construct(config: Rc<ReluConfig>) -> Self {
    Self { config }
  }

  pub fn configure<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    gadget_config: GadgetConfig,
  ) -> GadgetConfig {
    <ReluCircuit as NonLinearGadget::<F, D>>::configure(builder, gadget_config, GadgetType::Relu)
  }
}

impl<F: RichField + Extendable<D>, const D: usize> NonLinearGadget<F, D> for ReluCircuit {
  fn generate_map(_scale_factor: u64, min_val: i64, num_rows: i64) -> HashMap<i64, i64> {
    let mut map = HashMap::new();
    for i in 0..num_rows {
      let shifted = i + min_val;
      let relu = shifted.max(0);
      map.insert(i as i64, relu);
      // map.insert(i as i64, i as i64);
    }

    map
  }

  fn get_map(&self) -> &HashMap<i64, i64> {
    &self.config.maps.get(&GadgetType::Relu).unwrap()[0]
  }
}

impl<F: RichField + Extendable<D>, const D: usize> Gadget<F, D> for ReluCircuit {
  fn load_lookups(builder: &mut CircuitBuilder<F, D>, config: GadgetConfig) -> Option<usize> {
    println!("relu load lookups");
    <ReluCircuit as NonLinearGadget::<F, D>>::load_lookups(builder, Rc::new(config.clone()), GadgetType::Relu)
  }

  fn make_circuit(
    &self,
    builder: &mut CircuitBuilder<F, D>,
    vec_inputs: &Vec<Vec<&Target>>,
    _single_inputs: &Vec<F>,
    gadget_config: Rc<GadgetConfig>,
  ) -> Vec<Target> {
    NonLinearGadget::make_circuit(self, builder, &vec_inputs, gadget_config, GadgetType::Relu)
  }
}
