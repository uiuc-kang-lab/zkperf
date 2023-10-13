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

pub fn convert_to_u64<F: RichField + Extendable<D>, const D: usize>(x: &F) -> u64 {
  let big = x.to_canonical_biguint();
  let big_digits = big.to_u64_digits();
  if big_digits.len() > 2 {
    println!("big_digits: {:?}", big_digits);
  }
  if big_digits.len() == 1 {
    big_digits[0] as u64
  } else if big_digits.len() == 0 {
    0
  } else {
    panic!();
  }
}

pub trait Gadget<F: RichField + Extendable<D>, const D: usize> {
  fn make_circuit(
    &self,
    _builder: &mut CircuitBuilder<F, D>,
    _vec_inputs: &Vec<Vec<&Target>>,
    _gadget_config: Rc<GadgetConfig>
  ) -> Vec<Target> { return vec![] }
}
