use std::{rc::Rc, collections::HashMap};

use plonky2::{
  field::extension::Extendable,
  hash::hash_types::RichField,
  iop::target::Target,
  plonk::circuit_builder::CircuitBuilder,
};

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum GadgetType {
  Adder,
  BiasDivRoundRelu6,
  InputLookup, // Dummy placeholder for the input lookup
}

#[derive(Clone, Debug, Default)]
pub struct GadgetConfig {
  // pub used_gadgets: Arc<BTreeSet<GadgetType>>,
  pub tables: HashMap<GadgetType, Vec<usize>>,
  pub scale_factor: u64,
  pub shift_min_val: i64, // MUST be divisible by 2 * scale_factor
  pub num_rows: usize,
  pub num_cols: usize,
  pub k: usize,
  pub min_val: i64,
  pub max_val: i64,
  pub div_outp_min_val: i64,
  pub num_bits_per_elem: i64,
}

pub trait Gadget<F: RichField + Extendable<D>, const D: usize> {
  fn make_circuit(
    &self,
    _builder: &mut CircuitBuilder<F, D>,
    _vec_inputs: &Vec<Vec<&Target>>,
    _gadget_config: Rc<GadgetConfig>
  ) -> Vec<Target> { return vec![] }
}
