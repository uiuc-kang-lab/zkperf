use std::sync::Arc;
use std::{collections::HashMap, rc::Rc};

use plonky2::field::extension::Extendable;
use plonky2::hash::hash_types::RichField;
use plonky2::iop::target::Target;
use plonky2::plonk::circuit_builder::CircuitBuilder;

use super::super::gadget::Gadget;
use super::super::gadget::{GadgetConfig, GadgetType};

pub trait NonLinearGadget<F: RichField + Extendable<D>, const D: usize>: Gadget<F, D> {
  fn generate_map(scale_factor: u64, min_val: i64, num_rows: i64) -> HashMap<i64, i64>;

  fn get_map(&self) -> &HashMap<i64, i64>;

  fn configure(
    builder: &mut CircuitBuilder<F, D>,
    gadget_config: GadgetConfig,
    gadget_type: GadgetType,
  ) -> GadgetConfig {
    let cloned_gadget = gadget_config.clone();
    let mut tables = gadget_config.tables;

    let mut maps = gadget_config.maps;
    let non_linear_map = Self::generate_map(
      gadget_config.scale_factor,
      gadget_config.min_val,
      gadget_config.num_rows as i64,
    );
    maps.insert(gadget_type, vec![non_linear_map]);
    let map_config = GadgetConfig {
      maps,
      ..cloned_gadget
    };
    let cloned_map_config = map_config.clone();
    let outp_lookup = <Self as NonLinearGadget<F, D>>::load_lookups(builder, Rc::new(map_config), gadget_type).unwrap();

    let inp_lookup = tables.get(&GadgetType::InputLookup).unwrap()[0];
    tables.insert(gadget_type, vec![inp_lookup, outp_lookup]);

    GadgetConfig {
      tables,
      ..cloned_map_config
    }
  }

  fn load_lookups(builder: &mut CircuitBuilder<F, D>, config: Rc<GadgetConfig>, gadget_type: GadgetType) -> Option<usize> {
    // TODO may need to refactor, currently each of the nonlinearities have exactly one map
    println!("nl load lookups, num_rows: {}", config.num_rows);
    let map = &config.maps.get(&gadget_type).unwrap()[0];

    let shift_pos_i64 = -config.shift_min_val;

    let nl_table = Arc::new({
      (0..config.num_rows)
        .map(|i| {
          let i = i as i64;
          // FIXME: refactor this
          let tmp = *map.get(&i).unwrap();
          let val = if i == 0 {
            0
          } else {
            if tmp >= 0 {
              tmp as u64
            } else {
              let tmp = tmp + shift_pos_i64;
              tmp as u64 - shift_pos_i64 as u64
            }
          };
          (i as u32, val as u32)
        })
        .collect::<Vec<_>>()
    });
    Some(builder.add_lookup_table_from_pairs(nl_table))
  }

  fn make_circuit(
    &self,
    builder: &mut CircuitBuilder<F, D>,
    vec_inputs: &Vec<Vec<&Target>>,
    gadget_config: Rc<GadgetConfig>,
    gadget_type: GadgetType,
  ) -> Vec<Target> {
    let inps = &vec_inputs[0];
    let nl_table = gadget_config.tables.get(&gadget_type).unwrap()[1];

    // let shift_val_pos_i64 = -gadget_config.shift_min_val;
    // println!("shift val pos: {}", shift_val_pos_i64);
    // let shift_val_pos = F::from_canonical_u64(shift_val_pos_i64 as u64);
    let min_val = gadget_config.min_val;
    println!("min_val: {}", min_val);
    let zero = builder.zero();
    let neg_min_val = F::from_canonical_i64(-min_val);
    println!("neg min_val: {}", neg_min_val);
    let neg_min_val_t = builder.constant(neg_min_val);
    let min_val_t = builder.sub(zero, neg_min_val_t);

    // let mut shifted_inps = vec![];
    // let shift_val_pos_t = builder.constant(shift_val_pos);

    let shifted_inps = inps.iter().map(
      |x| {
        // let x = builder.sub(**x, shift_val_pos_t);
        // let pos = convert_to_u128(&(*x + shift_val_pos)) as i128 - shift_val_pos_i64 as i128;
        // let x = pos as i64 - min_val;
        builder.sub(**x, min_val_t)
        // let val = *map.get(&x).unwrap();
        // if x == 0 {
        //   F::ZERO
        // } else {
        //   if val >= 0 {
        //     F::from(val as u64)
        //   } else {
        //     let val_pos = val + shift_val_pos_i64;
        //     F::from(val_pos as u64) - F::from(shift_val_pos_i64 as u64)
        //   }
        // }
      }).collect::<Vec<_>>();

    let mut outps = vec![];

    for i in 0..inps.len() {
      // let outp = builder.add_lookup_from_index(*inps[i], nl_table);
      let outp = builder.add_lookup_from_index(shifted_inps[i], nl_table);
      outps.push(outp);
    }

    outps
  }
}
