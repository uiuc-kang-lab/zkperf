use std::{rc::Rc, sync::Arc};

use plonky2::{hash::hash_types::RichField, plonk::circuit_builder::CircuitBuilder};
use plonky2_field::extension::Extendable;

use super::gadget::{GadgetConfig, GadgetType};

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
    let mut tables = gadget_config.tables;
    let inp_table = Arc::new({
      (0..gadget_config.num_rows)
        .map(|i| (i as u16, i as u16))
        .collect::<Vec<_>>()
    });
    let inp_idx = builder.add_lookup_table_from_pairs(inp_table);
    tables.insert(GadgetType::InputLookup, vec![inp_idx]);

    GadgetConfig {
        tables,
        ..gadget_config
    }
  }
}
