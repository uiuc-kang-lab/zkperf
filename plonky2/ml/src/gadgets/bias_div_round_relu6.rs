use std::rc::Rc;
use std::sync::Arc;

use crate::gates::{bias_div_round::BiasDivRoundGate, var_div::DivRoundGate};

use plonky2::field::extension::Extendable;
use plonky2::hash::hash_types::RichField;
use plonky2::iop::target::Target;
use plonky2::plonk::circuit_builder::CircuitBuilder;

type BiasDivRoundRelu6Config = GadgetConfig;

use super::gadget::{Gadget, GadgetConfig, GadgetType};

pub struct BiasDivRoundRelu6Circuit {
  _config: Rc<BiasDivRoundRelu6Config>,
}

impl BiasDivRoundRelu6Circuit {
  pub fn construct(config: Rc<BiasDivRoundRelu6Config>) -> Self {
    Self { _config: config }
  }

  pub fn configure<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    gadget_config: GadgetConfig,
  ) -> GadgetConfig {
    let mut tables = gadget_config.tables;
    let relu_table = Arc::new({
      let div_val = gadget_config.scale_factor;
      (0..gadget_config.num_rows)
        .map(|d| {
          let shifted = d as i64 + gadget_config.min_val;
          (d as u16, shifted.clamp(0, 6 * div_val as i64) as u16)
        })
        .collect::<Vec<_>>()
    });
    let relu_idx = builder.add_lookup_table_from_pairs(relu_table);
    tables.insert(GadgetType::BiasDivRoundRelu6, vec![relu_idx]);

    GadgetConfig {
      tables,
      ..gadget_config
    }
  }
}

impl<F: RichField + Extendable<D>, const D: usize> Gadget<F, D> for BiasDivRoundRelu6Circuit {
  fn make_circuit(
    &self,
    builder: &mut CircuitBuilder<F, D>,
    vec_inputs: &Vec<Vec<&Target>>,
    gadget_config: Rc<GadgetConfig>,
  ) -> Vec<Target> {
    let inps = &vec_inputs[0];
    let biases = &vec_inputs[1];
    let div_outp_min_val = F::from_canonical_i64(gadget_config.div_outp_min_val);
    let shift_min_val = F::from_canonical_i64(gadget_config.shift_min_val);
    let div_outp_min_val_target = builder.constant(div_outp_min_val);
    // let sf_target = builder.constant(F::from_canonical_u64(gadget_config.scale_factor));
    // let two = builder.constant(F::from_canonical_u64(2));

    assert!(inps.len() % biases.len() == 0);

    // let div_table = gadget_config.tables.get(&GadgetType::InputLookup).unwrap()[0];
    let relu_table = gadget_config
      .tables
      .get(&GadgetType::BiasDivRoundRelu6)
      .unwrap()[0];

    let mut out_vec = vec![];
    for i in 0..inps.len() {
      // let bdr_gate = builder.add_gate(
      //   DivRoundGate { num_ops: 1 },
      //   vec![
      //     F::from_canonical_u64(gadget_config.scale_factor),
      //     shift_min_val,
      //     div_outp_min_val,
      //   ],
      // );
      // builder.connect(
      //   *inps[i],
      //   Target::wire(bdr_gate, DivRoundGate::wire_input()),
      // );

      let bdr_gate = builder.add_gate(
        BiasDivRoundGate { num_ops: 1 },
        vec![
          F::from_canonical_u64(gadget_config.scale_factor),
          shift_min_val,
          div_outp_min_val,
        ],
      );
      builder.connect(
        *inps[i],
        Target::wire(bdr_gate, BiasDivRoundGate::wire_input()),
      );
      builder.connect(
        *biases[i % biases.len()],
        Target::wire(bdr_gate, BiasDivRoundGate::wire_bias()),
      );

      // constrain division
      // let div = Target::wire(bdr_gate, BiasDivRoundGate::wire_div());
      // let div = builder.add(div, div_outp_min_val_target);
      // let div_lookup = builder.add_lookup_from_index(div, div_table);
      // builder.connect(
      //   div_lookup,
      //   Target::wire(bdr_gate, BiasDivRoundGate::wire_div_div_lookup()),
      // );

      // mod lookup should be in domain already
      // let mod_res = Target::wire(bdr_gate, BiasDivRoundGate::wire_mod());
      // let div_mod_diff = builder.mul_sub(two, sf_target, mod_res);
      // let mod_lookup = builder.add_lookup_from_index(div_mod_diff, div_table);
      // builder.connect(
      //   mod_lookup,
      //   Target::wire(bdr_gate, BiasDivRoundGate::wire_mod_div_lookup()),
      // );

      let div_res = Target::wire(bdr_gate, BiasDivRoundGate::wire_div());
      let x_pos = builder.sub(div_res, div_outp_min_val_target);
      let outp = builder.add_lookup_from_index(x_pos, relu_table);

      // let div_res = Target::wire(bdr_gate, DivRoundGate::wire_output());
      // let div_res = builder.add(div_res, *biases[i % biases.len()]);
      // let x_pos = builder.sub(div_res, div_outp_min_val_target);
      // let outp = builder.add_lookup_from_index(x_pos, relu_table);

      // interleave div with relu and div without relu
      out_vec.push(outp);
      out_vec.push(div_res);
    }

    out_vec
  }
}
