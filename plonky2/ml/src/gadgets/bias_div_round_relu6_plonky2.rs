use std::rc::Rc;
use std::sync::Arc;

use crate::gadgets::div::DivisionGate;

use plonky2::field::extension::Extendable;
use plonky2::hash::hash_types::RichField;
use plonky2::iop::target::Target;
use plonky2::plonk::circuit_builder::CircuitBuilder;

use super::gadget::{GadgetConfig, Gadget};

const SHIFT_MIN_VAL: i64 = -(1 << 30);

fn make_relu6_table<F: RichField + Extendable<D>, const D: usize>(
  builder: &mut CircuitBuilder<F, D>,
  scale_factor: u64,
  min_val: i64,
  num_rows: u64,
) -> usize {

  println!("relu num_rows: {}", num_rows);
  let relu_table = Arc::new({
    let div_val = scale_factor;
    (0..num_rows)
      .map(|d: u64| {
        let shifted = d as i64 + min_val;
        (d as u16, shifted.clamp(0, 6 * div_val as i64) as u16)
      })
      .collect::<Vec<_>>()
  });
  builder.add_lookup_table_from_pairs(relu_table)
}

pub struct BiasDivRoundRelu6Circuit {}

impl<F: RichField + Extendable<D>, const D: usize> Gadget<F, D> for BiasDivRoundRelu6Circuit {
  fn make_circuit(
    &self,
    builder: &mut CircuitBuilder<F, D>,
    vec_inputs: &Vec<Vec<&Target>>,
    gadget_config: Rc<GadgetConfig>
  ) -> Vec<Target> {
    let inps = &vec_inputs[0];
    let biases = &vec_inputs[1];
    let div_outp_min_val_i64 = builder.constant(F::from_canonical_u64((-gadget_config.div_outp_min_val) as u64));
    let div_inp_min_val_pos_i64 = -SHIFT_MIN_VAL;
    let div_inp_min_val_pos = F::from_canonical_u64(div_inp_min_val_pos_i64 as u64);

    assert!(inps.len() % biases.len() == 0);

    // TODO for now use exactly one lut
    println!("min_val: {}", gadget_config.min_val);
    let relu_table = if builder.num_luts() == 0 {
      make_relu6_table(
        builder,
        gadget_config.scale_factor,
        gadget_config.min_val,
        (gadget_config.num_rows as i64).try_into().unwrap(),
        // u16::MAX as u64
      )
    } else {
      0
    };

    // let div_gate = builder.add_gate(
    //   DivisionGate {
    //     num_ops: gadget_config.num_cols,
    //   },
    //   vec![F::from_canonical_u64(gadget_config.scale_factor)],
    // );
    let mut out_vec = vec![];
    let zero = builder.zero();
    // let max = builder.constant(F::from_canonical_u32(u16::MAX as u32 * gadget_config.scale_factor as u32));
    for i in 0..inps.len() {
      let mut inp = inps[i].clone();
      // let ca_target = builder.mul(c_target, a_target);
      // let inp_bits = builder.split_le(inp, 64);
      // inp = builder.select(inp_bits[63], zero, inp);
      let x_pos = builder.add(inp, div_outp_min_val_i64);
      let c_target = x_pos;

      // let c_target = builder.add_lookup_from_index(inp, relu_table);

      let div_gate = builder.add_gate(
        DivisionGate {
          // num_ops: gadget_config.num_cols,
          num_ops: 1,
        },
        vec![F::from_canonical_u64(gadget_config.scale_factor)],
      );
      builder.connect(
        c_target,
        Target::wire(div_gate, DivisionGate::wire_input()),
        // Target::wire(div_gate, DivisionGate::wire_ith(i)),
      );
      let d_target = Target::wire(div_gate, DivisionGate::wire_output());

      // let d_bits = builder.split_le(d_target, 64);
      // for i in 16..63 {
      //   d_target = builder.select(d_bits[i], zero, d_target);
      // }
      let a_i_target = builder.add_lookup_from_index(d_target, relu_table);
      out_vec.push(builder.add(a_i_target, *biases[i % biases.len()]));
    }

    out_vec
  }
}
