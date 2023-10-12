use std::{collections::HashMap, rc::Rc};

use ndarray::{Array, IxDyn};

use plonky2::{
  field::extension::Extendable, hash::hash_types::RichField, iop::target::Target,
  plonk::circuit_builder::CircuitBuilder,
};

use crate::{
  gadgets::gadget::GadgetConfig,
  gates::var_div::DivRoundGate,
  layers::{conv2d::Conv2DCircuit, conv2d::PaddingEnum, layer::LayerConfig},
};

use super::layer::Layer;

pub struct AvgPool2DCircuit {}

impl<F: RichField + Extendable<D>, const D: usize> Layer<F, D> for AvgPool2DCircuit {
  fn make_circuit(
    &self,
    builder: &mut CircuitBuilder<F, D>,
    tensors: &Vec<Array<Rc<Target>, IxDyn>>,
    _constants: &HashMap<i64, Rc<F>>,
    gadget_config: Rc<GadgetConfig>,
    layer_config: &LayerConfig,
  ) -> Vec<Array<Rc<Target>, IxDyn>> {
    // assert!(weight_height == weight_width);

    let inp = &tensors[0];
    assert_eq!(inp.shape().len(), 4);
    // Don't support batch size > 1 yet
    assert_eq!(inp.shape()[0], 1);

    let h = inp.shape()[1];
    let w = inp.shape()[2];
    let c = inp.shape()[3];

    let (fx, fy) = (
      layer_config.layer_params[0] as usize,
      layer_config.layer_params[1] as usize,
    );
    let div = fx * fy;
    let (sh, sw) = (
      layer_config.layer_params[2] as usize,
      layer_config.layer_params[3] as usize,
    );

    let (oh, ow) = Conv2DCircuit::out_hw(h, w, sh, sw, fx, fy, PaddingEnum::Valid);
    let batch_size = inp.shape()[0];

    let mut sum_outp = vec![];
    for x in 0..oh {
      for y in 0..ow {
        for chan in 0..c {
          let pool_targets = (0..sh - fx + 1)
            .map(|i| (0..sw - fy + 1).map(move |j| inp[[0, x + i, y + j, chan]].as_ref()))
            .flatten().collect::<Vec<_>>();
          sum_outp.push(builder.add_many(pool_targets));
        }
      }
    }

    let mut div_gates = vec![];
    let div_outp_min_val = F::from_canonical_u64(gadget_config.div_outp_min_val as u64);
    let shift_min_val = F::from_canonical_u64(gadget_config.shift_min_val as u64);
    for i in 0..sum_outp.len() {
      let div_gate = builder.add_gate(
        DivRoundGate {
          num_ops: 1,
        },
        vec![
          F::from_canonical_u64(div as u64),
          shift_min_val,
          div_outp_min_val
        ],
      );
      div_gates.push(div_gate);
      // println!("div_gate: {}, outp: {}", div_gate, i);
      builder.connect(
        sum_outp[i],
        Target::wire(div_gate, DivRoundGate::wire_input()),
      );
    }

    let outp = (0..sum_outp.len())
      .map(|i| Rc::new(Target::wire(div_gates[i], DivRoundGate::wire_output())))
      .collect::<Vec<_>>();

    vec![Array::from_shape_vec(IxDyn(&vec![batch_size, oh, ow, c]), outp).unwrap()]
  }
}
