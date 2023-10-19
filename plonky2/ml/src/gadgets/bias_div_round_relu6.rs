use std::sync::Arc;
use std::{collections::HashMap, rc::Rc};

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

  pub fn generate_map(scale_factor: u64, min_val: i64, num_rows: i64) -> HashMap<i64, i64> {
    let div_val = scale_factor;

    let mut map = HashMap::new();
    for i in 0..num_rows {
      // let shifted = i;
      let shifted = i + min_val;
      let val = shifted.clamp(0, 6 * div_val as i64);
      map.insert(i as i64, val);
    }
    map
  }

  pub fn configure<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    gadget_config: GadgetConfig,
  ) -> GadgetConfig {
    let cloned_gadget = gadget_config.clone();
    let mut maps = gadget_config.maps;
    let relu6_map = Self::generate_map(
      gadget_config.scale_factor,
      gadget_config.min_val,
      gadget_config.num_rows as i64,
    );
    maps.insert(GadgetType::BiasDivRoundRelu6, vec![relu6_map]);
    let map_config = GadgetConfig {
      maps,
      ..cloned_gadget
    };
    let cloned_map_config = map_config.clone();
    let lookup = Self::load_lookups(builder, map_config).unwrap();
    let mut tables = gadget_config.tables;
    tables.insert(GadgetType::BiasDivRoundRelu6, vec![lookup]);
    println!("biasdivround lookup: {}", lookup);

    GadgetConfig {
      tables,
      ..cloned_map_config
    }
  }
}

impl<F: RichField + Extendable<D>, const D: usize> Gadget<F, D> for BiasDivRoundRelu6Circuit {
  fn load_lookups(builder: &mut CircuitBuilder<F, D>, config: GadgetConfig) -> Option<usize> {
    println!("load lookups");
    let map = &config.maps[&GadgetType::BiasDivRoundRelu6][0];

    println!("num_rows: {}", config.num_rows);
    let relu_table = Arc::new({
      (0..config.num_rows)
        .map(|d| {
          let i = d as i64;
          let val = map.get(&i).unwrap();
          (i as u32, *val as u32)
        })
        .collect::<Vec<_>>()
    });
    Some(builder.add_lookup_table_from_pairs(relu_table))
  }

  fn make_circuit(
    &self,
    builder: &mut CircuitBuilder<F, D>,
    vec_inputs: &Vec<Vec<&Target>>,
    _single_inputs: &Vec<F>,
    gadget_config: Rc<GadgetConfig>,
  ) -> Vec<Target> {
    let inps = &vec_inputs[0];
    let biases = &vec_inputs[1];

    // values > 0xFFFFFFFF00000001 cannot be safely converted with from_canonical...
    let zero = builder.zero();
    let neg_div_outp_min_val_t = builder.constant(F::from_canonical_i64(-gadget_config.div_outp_min_val));
    let div_outp_min_val_t = builder.sub(zero, neg_div_outp_min_val_t);

    let div_outp_min_val = F::from_canonical_i64(gadget_config.div_outp_min_val);
    let shift_min_val = F::from_canonical_i64(gadget_config.shift_min_val);

    assert!(inps.len() % biases.len() == 0);

    let relu_table = gadget_config
      .tables
      .get(&GadgetType::BiasDivRoundRelu6)
      .unwrap()[0];

    let mut out_vec = vec![];
    // let div_outp_min_val_t = builder.constant(div_outp_min_val);
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
      // div_res + div_outp_min_val in lookup

      // let x_bits = builder.split_le(div_res, 64);
      // let x_relu = builder.select(x_bits[63], zero, div_res);

      // serves as a constraint for both relu input and output
      let relu_pos = builder.sub(div_res, div_outp_min_val_t);
      let outp = builder.add_lookup_from_index(relu_pos, relu_table);

      // interleave div with relu and div without relu
      out_vec.push(outp);
      out_vec.push(div_res);
    }

    out_vec
  }
}
