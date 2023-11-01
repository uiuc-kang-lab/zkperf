use std::rc::Rc;

use plonky2::field::extension::Extendable;
use plonky2::hash::hash_types::RichField;
use plonky2::iop::target::Target;
use plonky2::plonk::circuit_builder::CircuitBuilder;

use crate::gates::var_div::DivRoundGate;

use super::gadget::{Gadget, GadgetConfig, GadgetType};

type DivRoundConfig = GadgetConfig;

pub struct DivRoundCircuit {
  _config: Rc<DivRoundConfig>,
}

impl DivRoundCircuit {
  pub fn construct(config: Rc<DivRoundConfig>) -> Self {
    Self { _config: config }
  }

  pub fn configure<F: RichField + Extendable<D>, const D: usize>(
    _builder: &mut CircuitBuilder<F, D>,
    gadget_config: GadgetConfig,
  ) -> GadgetConfig {
    gadget_config
  }
}

impl<F: RichField + Extendable<D>, const D: usize> Gadget<F, D> for DivRoundCircuit {
  fn load_lookups(_builder: &mut CircuitBuilder<F, D>, _config: GadgetConfig) -> Option<usize> {
    None
  }

  fn make_circuit(
    &self,
    builder: &mut CircuitBuilder<F, D>,
    vec_inputs: &Vec<Vec<&Target>>,
    single_inputs: &Vec<F>,
    gadget_config: Rc<GadgetConfig>,
  ) -> Vec<Target> {
    let inp = &vec_inputs[0];
    let mut div_gates = vec![];
    let div_outp_min_val = F::from_canonical_u64(gadget_config.div_outp_min_val as u64);
    let shift_min_val = F::from_canonical_u64(gadget_config.shift_min_val as u64);
    // println!("vdiv div_outp_min_val: {}, vdiv shift_min_val: {}", div_outp_min_val, shift_min_val);
    let div = single_inputs[0];

    // r is already constrained within the gate
    // check a stronger constraint than necessary 2 * div \in [0, 2^N)
    let div_lookup = gadget_config.tables.get(&GadgetType::InputLookup).unwrap()[0];
    let two_div = builder.constant(F::from_canonical_u16(2) * div);
    builder.add_lookup_from_index(two_div, div_lookup);

    for i in 0..inp.len() {
      let div_gate = builder.add_gate(
        DivRoundGate { num_ops: 1 },
        vec![div, shift_min_val, div_outp_min_val],
      );
      div_gates.push(div_gate);
      builder.connect(*inp[i], Target::wire(div_gate, DivRoundGate::wire_input()));
    }

    (0..inp.len())
      .map(|i| Target::wire(div_gates[i], DivRoundGate::wire_output()))
      .collect::<Vec<_>>()
  }
}
