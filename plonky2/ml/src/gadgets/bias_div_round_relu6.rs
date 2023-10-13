use std::rc::Rc;
use std::sync::Arc;

use crate::gates::bias_div_round::BiasDivRoundGate;

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
      (0..u16::MAX)
        .map(|d| {
          let val  = d.clamp(0, 6 * div_val as u16);
          (d, val)
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

    assert!(inps.len() % biases.len() == 0);

    let relu_table = gadget_config
      .tables
      .get(&GadgetType::BiasDivRoundRelu6)
      .unwrap()[0];

    let mut out_vec = vec![];
    let zero = builder.zero();
    for i in 0..inps.len() {
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

      let div_res = Target::wire(bdr_gate, BiasDivRoundGate::wire_div());
      // let x_pos = builder.sub(div_res, div_outp_min_val_target);
      let x_bits = builder.split_le(div_res, 64);
      let x_relu = builder.select(x_bits[63], zero, div_res);
      let outp = builder.add_lookup_from_index(x_relu, relu_table);

      // interleave div with relu and div without relu
      out_vec.push(outp);
      out_vec.push(div_res);
    }

    out_vec
  }
}
