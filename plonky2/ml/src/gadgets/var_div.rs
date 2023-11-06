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
    let div_outp_min_val = F::from_canonical_u64(gadget_config.div_outp_min_val as u64);
    let shift_min_val = F::from_canonical_u64(gadget_config.shift_min_val as u64);
    // println!("vdiv div_outp_min_val: {}, vdiv shift_min_val: {}", div_outp_min_val, shift_min_val);
    let div = single_inputs[0];

    // r is already constrained within the gate
    // check a stronger constraint than necessary 2 * div \in [0, 2^N)
    let div_lookup = gadget_config.tables.get(&GadgetType::InputLookup).unwrap()[0];
    let two_div = builder.constant(F::from_canonical_u16(2) * div);
    builder.add_lookup_from_index(two_div, div_lookup);

    let num_ops = DivRoundGate::num_ops(&builder.config);
    let mut div_gates = vec![];
    for i in 0..inp.len() {
      let wire_idx = i % num_ops;
      if wire_idx == 0 {
        let dr_gate = builder.add_gate(
          DivRoundGate::new_from_config(&builder.config),
          vec![div, shift_min_val, div_outp_min_val],
        );
        div_gates.push(dr_gate);
      }
      let gate_idx = i / num_ops;
      builder.connect(
        *inp[i],
        Target::wire(div_gates[gate_idx], DivRoundGate::wire_ith_input(wire_idx)),
      );
    }

    let zero = builder.zero();
    if inp.len() % num_ops != 0 {
      for i in (inp.len() % num_ops)..num_ops {
        println!("i: {}", i);
        builder.connect(
          zero,
          Target::wire(
            div_gates[div_gates.len() - 1],
            DivRoundGate::wire_ith_input(i),
          ),
        );
      }
    }

    (0..inp.len())
      .map(|i| {
        Target::wire(
          div_gates[i / num_ops],
          DivRoundGate::wire_ith_output(i % num_ops),
        )
      })
      .collect::<Vec<_>>()
  }
}
