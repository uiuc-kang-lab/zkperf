use std::{collections::HashMap, rc::Rc, vec};

use ndarray::{Array, IxDyn};

use plonky2::{
  field::extension::Extendable, hash::hash_types::RichField, iop::target::Target,
  plonk::circuit_builder::CircuitBuilder,
};

use crate::{gadgets::gadget::GadgetConfig, gates::var_div::DivRoundGate};

use super::super::layers::layer::{Layer, LayerConfig};

#[derive(Clone, Debug)]
pub struct MulCircuit {}

impl<F: RichField + Extendable<D>, const D: usize> Layer<F, D> for MulCircuit {
  fn make_circuit(
    &self,
    builder: &mut CircuitBuilder<F, D>,
    tensors: &Vec<Array<Rc<Target>, IxDyn>>,
    _constants: &HashMap<i64, Rc<F>>,
    gadget_config: Rc<GadgetConfig>,
    _layer_config: &LayerConfig,
  ) -> Vec<Array<Rc<Target>, IxDyn>> {
    let inp = &tensors[0];
    let mul = &tensors[1];
    assert!(inp.shape()[3] == mul.shape()[0]);

    let flat_inp = Array::from_iter(inp.iter().cloned());

    let mut mul_outp = vec![];

    for i in 0..flat_inp.len() {
      mul_outp.push(Rc::new(builder.mul(*flat_inp[i], *mul[i % mul.len()])))
    }

    let mut div_gates = vec![];
    let div_outp_min_val = F::from_canonical_u64(gadget_config.div_outp_min_val as u64);
    let shift_min_val = F::from_canonical_u64(gadget_config.shift_min_val as u64);
    for i in 0..mul_outp.len() {
      let div_gate = builder.add_gate(
        DivRoundGate { num_ops: 1 },
        vec![
          F::from_canonical_u64(gadget_config.scale_factor as u64),
          shift_min_val,
          div_outp_min_val,
        ],
      );
      div_gates.push(div_gate);
      builder.connect(
        *mul_outp[i],
        Target::wire(div_gate, DivRoundGate::wire_input()),
      );
    }

    let outp = (0..mul_outp.len())
      .map(|i| Rc::new(Target::wire(div_gates[i], DivRoundGate::wire_output())))
      .collect::<Vec<_>>();

    vec![Array::from_shape_vec(IxDyn(inp.shape()), outp).unwrap()]
  }
}
