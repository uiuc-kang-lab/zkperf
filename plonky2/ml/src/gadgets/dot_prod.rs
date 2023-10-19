use std::rc::Rc;

use plonky2::field::extension::Extendable;
use plonky2::hash::hash_types::RichField;
use plonky2::iop::target::Target;
use plonky2::plonk::circuit_builder::CircuitBuilder;

use crate::gates::{dot_prod::{DOTPROD_SIZE, DotProductGate}, var_div::DivRoundGate};

use super::gadget::{Gadget, GadgetConfig, GadgetType};

type DotProductConfig = GadgetConfig;

pub struct DotProductCircuit {
  pub _config: Rc<DotProductConfig>,
}

impl DotProductCircuit {
  pub fn construct(config: Rc<DotProductConfig>) -> Self {
    Self { _config: config }
  }

  pub fn configure<F: RichField + Extendable<D>, const D: usize>(
    _builder: &mut CircuitBuilder<F, D>,
    gadget_config: GadgetConfig,
  ) -> GadgetConfig {
    gadget_config
  }
}

impl<F: RichField + Extendable<D>, const D: usize> Gadget<F, D> for DotProductCircuit {
  fn load_lookups(_builder: &mut CircuitBuilder<F, D>, _config: GadgetConfig) -> Option<usize> {
    None
  }

  fn make_circuit(
    &self,
    builder: &mut CircuitBuilder<F, D>,
    vec_inputs: &Vec<Vec<&Target>>,
    single_inputs: &Vec<F>,
    _gadget_config: Rc<GadgetConfig>,
  ) -> Vec<Target> {
    let dp_size = DOTPROD_SIZE;
    let mut inputs = vec_inputs[0].clone();
    let mut weights = vec_inputs[1].clone();
    let zero = single_inputs[0];
    let zero_t = builder.constant(zero);
    assert_eq!(inputs.len(), weights.len());

    while inputs.len() % dp_size != 0 {
      inputs.push(&zero_t);
      weights.push(&zero_t);
    }

    let mut dp_gates = vec![];

    let mut dp_gate = 0;
    for i in 0..inputs.len() {
      if i % dp_size == 0 {
        dp_gate = builder.add_gate(
          DotProductGate {},
          vec![single_inputs[0]],
        );
        dp_gates.push(dp_gate);
      }
      let wire_idx = i % dp_size;
      builder.connect(*inputs[i], Target::wire(dp_gate, DotProductGate::wire_ith_input(wire_idx)));
      builder.connect(*weights[i], Target::wire(dp_gate, DotProductGate::wire_ith_input(wire_idx)));
    }

    let outp = builder.add_many(dp_gates.iter().map(|row| Target::wire(*row, DotProductGate::wire_output())));
    vec![outp]
  }
}
