use std::{rc::Rc, sync::Arc};

use plonky2::field::extension::Extendable;
use plonky2::hash::hash_types::RichField;
use plonky2::iop::target::Target;
use plonky2::plonk::circuit_builder::CircuitBuilder;

use super::gadget::{Gadget, GadgetConfig, GadgetType};

pub struct InputLookupCircuit {
  _config: Rc<GadgetConfig>,
}

impl InputLookupCircuit {
  pub fn construct(config: Rc<GadgetConfig>) -> Self {
    Self { _config: config }
  }

  pub fn configure<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    gadget_config: GadgetConfig,
  ) -> GadgetConfig {
    let lookup = Self::load_lookups(builder, gadget_config.clone()).unwrap();
    let mut tables = gadget_config.tables;
    tables.insert(GadgetType::InputLookup, vec![lookup]);

    GadgetConfig {
      tables,
      ..gadget_config
    }
  }
}

impl<F: RichField + Extendable<D>, const D: usize> Gadget<F, D> for InputLookupCircuit {
  fn load_lookups(builder: &mut CircuitBuilder<F, D>, config: GadgetConfig) -> Option<usize> {
    let table = Arc::new({
      (0..config.num_rows)
        .map(|d| {
          (d as u32, d as u32)
        })
        .collect::<Vec<_>>()
    });
    Some(builder.add_lookup_table_from_pairs(table))
  }

  fn make_circuit(
    &self,
    _builder: &mut CircuitBuilder<F, D>,
    _vec_inputs: &Vec<Vec<&Target>>,
    _single_inputs: &Vec<F>,
    _gadget_config: Rc<GadgetConfig>,
  ) -> Vec<Target> {
    panic!("InputLookupCircuit should not be called directly")
  }
}
