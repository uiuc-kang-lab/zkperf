use std::rc::Rc;

use crate::gadgets::div::DivisionGate;

use plonky2::field::extension::Extendable;
use plonky2::hash::hash_types::RichField;
use plonky2::iop::target::Target;
use plonky2::plonk::circuit_builder::CircuitBuilder;

use super::gadget::{GadgetConfig, Gadget};

pub struct BiasDivRoundCircuit {}

impl<F: RichField + Extendable<D>, const D: usize> Gadget<F, D> for BiasDivRoundCircuit {
  fn make_circuit(
    &self,
    builder: &mut CircuitBuilder<F, D>,
    vec_inputs: &Vec<Vec<&Target>>,
    gadget_config: Rc<GadgetConfig>
  ) -> Vec<Target> {
    let inps = &vec_inputs[0];
    let biases = &vec_inputs[1];
    assert!(inps.len() % biases.len() == 0);

    let mut out_vec = vec![];
    // let zero = builder.zero();
    for i in 0..inps.len() {
      let inp = inps[i];
      let div_gate = builder.add_gate(
        DivisionGate {
          num_ops: 1,
        },
        vec![F::from_canonical_u64(gadget_config.scale_factor)],
      );
      builder.connect(
        *inp,
        Target::wire(div_gate, DivisionGate::wire_input()),
      );
      let d_target = Target::wire(div_gate, DivisionGate::wire_output());

      // let d_bits = builder.split_le(d_target, 64);
      // for i in 16..63 {
      //   d_target = builder.select(d_bits[i], zero, d_target);
      // }
      // this is not added to the division gate to reduce gate wires
      out_vec.push(builder.add(d_target, *biases[i % biases.len()]));
    }

    out_vec
  }
}
