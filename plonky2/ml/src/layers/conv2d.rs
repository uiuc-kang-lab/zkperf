use std::{collections::HashMap, rc::Rc};

use crate::{
  gadgets::{
    bias_div_round_relu6::BiasDivRoundRelu6Circuit,
    gadget::{Gadget, GadgetConfig},
  },
  gates::dot_prod::{DotProductGate, DOTPROD_SIZE},
};
use ndarray::{Array, IxDyn};
use plonky2::{
  field::extension::Extendable, hash::hash_types::RichField, iop::target::Target,
  plonk::circuit_builder::CircuitBuilder,
};

use super::layer::{ActivationType, Layer, LayerConfig};

#[derive(Default, Clone, Copy, Eq, PartialEq)]
pub enum PaddingEnum {
  Same,
  #[default]
  Valid,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ConvLayerEnum {
  #[default]
  Conv2D,
  DepthwiseConv2D,
}

pub struct Conv2DConfig {
  pub conv_type: ConvLayerEnum,
  pub padding: PaddingEnum,
  pub activation: ActivationType,
  pub stride: (usize, usize),
}

pub struct Conv2DCircuit {
  pub config: LayerConfig,
}

impl Conv2DCircuit {
  pub fn param_vec_to_config(layer_params: Vec<i64>) -> Conv2DConfig {
    let conv_type = match layer_params[0] {
      0 => ConvLayerEnum::Conv2D,
      1 => panic!("DepthwiseConv2D is unimplemented"),
      _ => panic!("Invalid conv type"),
    };
    let padding = match layer_params[1] {
      0 => panic!("Same is unimplemented"),
      1 => PaddingEnum::Valid,
      _ => panic!("Invalid padding"),
    };
    let activation = match layer_params[2] {
      0 => ActivationType::None,
      1 => panic!("Relu is unimplemented"),
      3 => ActivationType::Relu6,
      _ => panic!("Invalid activation type"),
    };
    let stride = (layer_params[3] as usize, layer_params[4] as usize);
    Conv2DConfig {
      conv_type,
      padding,
      activation,
      stride,
    }
  }

  pub fn out_hw(
    h: usize,
    w: usize,
    si: usize,
    sj: usize,
    ch: usize,
    cw: usize,
    padding: PaddingEnum,
  ) -> (usize, usize) {
    /*
    println!(
      "H: {}, W: {}, SI: {}, SJ: {}, CH: {}, CW: {}",
      h, w, si, sj, ch, cw
    );
    */
    // https://iq.opengenus.org/same-and-valid-padding/
    match padding {
      PaddingEnum::Same => ((h + si - 1) / si, (w + sj - 1) / sj),
      // TODO: the above is probably correct, but we always have valid paddings
      // PaddingEnum::Same => (h / si, w / sj),
      PaddingEnum::Valid => ((h - ch) / si + 1, (w - cw) / sj + 1),
    }
  }
}

impl<F: RichField + Extendable<D>, const D: usize> Layer<F, D> for Conv2DCircuit {
  fn make_circuit(
    &self,
    builder: &mut CircuitBuilder<F, D>,
    tensors: &Vec<Array<Rc<Target>, IxDyn>>,
    constants: &HashMap<i64, Rc<F>>,
    gadget_config: Rc<GadgetConfig>,
    _layer_config: &LayerConfig,
  ) -> Vec<Array<Rc<Target>, IxDyn>> {
    let conv_config = &Self::param_vec_to_config(self.config.layer_params.clone());
    let zero_target = builder.zero();
    let zero = constants.get(&0).unwrap();
    let z_target = builder.constant(**zero);

    let inp = &tensors[0];
    let weights = &tensors[1];
    let h = inp.shape()[1];
    let w = inp.shape()[2];
    assert!(h == w);
    // TODO implement adder for h > DOTPROD_SIZE

    let ch = weights.shape()[1];
    let cw = weights.shape()[2];
    let oc = weights.shape()[0];
    let ic = inp.shape()[3];
    assert!(ch <= DOTPROD_SIZE);
    assert!(ch == cw);

    let (oh, ow) = Self::out_hw(
      h,
      w,
      conv_config.stride.0,
      conv_config.stride.1,
      ch,
      cw,
      conv_config.padding,
    );

    let batch_size = inp.shape()[0];

    let gate = DotProductGate::construct();

    let flat_rows = (0..batch_size * oh * ow * oc * ic)
      .map(|_| builder.add_gate(gate.clone(), vec![**zero]))
      .collect::<Vec<_>>();
    let rows = Array::from_shape_vec(IxDyn(&[batch_size, oh, ow, oc, ic]), flat_rows).unwrap();

    // TODO: assumes padding 0, stride 1
    let mut conv_outp = vec![];
    for batch in 0..batch_size {
      for x in 0..oh {
        for y in 0..ow {
          for chan_out in 0..oc {
            for chan_in in 0..ic {
              let row = rows[[batch, x, y, chan_out, chan_in]];
              for i in 0..DOTPROD_SIZE {
                for j in 0..DOTPROD_SIZE {
                  if i < ch && j < cw {
                    builder.connect(
                      *inp[[batch, x + i, y + j, chan_in]],
                      Target::wire(row, DotProductGate::wire_ijth_input(i, j)),
                    );
                    builder.connect(
                      *weights[[chan_out, i, j, chan_in]],
                      Target::wire(row, DotProductGate::wire_ijth_weight(i, j)),
                    );
                  } else {
                    builder.connect(
                      z_target,
                      Target::wire(row, DotProductGate::wire_ijth_input(i, j)),
                    );
                    builder.connect(
                      zero_target,
                      Target::wire(row, DotProductGate::wire_ijth_weight(i, j)),
                    );
                  }
                }
              }
            }
            conv_outp.push(builder.add_many((0..ic).map(|i| {
              // if chan_out == 7 && x == 23 && y == 17 {
              //   println!("outp: {}, {}, {}, {}, {}", batch, chan_out, x, y, i);
              // }
              Target::wire(
                rows[[batch, x, y, chan_out, i]],
                DotProductGate::wire_output(),
              )
            })));
          }
        }
      }
    }

    let flat_biases = Array::from_iter(tensors[2].iter()).to_vec();
    let mut biases = vec![];
    for bias in flat_biases.iter() {
      biases.push(bias.as_ref());
    }

    // bdr outputs interleaved [(relu'd, div'd), (relu'd, div'd), ...]
    // Uninterleave depending on whether or not we're doing the relu
    let bdr_circuit = BiasDivRoundRelu6Circuit::construct(gadget_config.clone());
    let outp_flat = conv_outp.iter().map(|x| x).collect::<Vec<_>>();
    let outp = bdr_circuit.make_circuit(builder, &vec![outp_flat, biases], gadget_config);
    let outp = if conv_config.activation == ActivationType::Relu6 {
      outp
        .into_iter()
        .step_by(2)
        .map(|x| Rc::new(x))
        .collect::<Vec<_>>()
    } else if conv_config.activation == ActivationType::None {
      outp
        .into_iter()
        .skip(1)
        .step_by(2)
        .map(|x| Rc::new(x))
        .collect::<Vec<_>>()
    } else {
      panic!("Unsupported activation type");
    };

    let outp = outp.into_iter().map(|x| x).collect::<Vec<_>>();
    vec![Array::from_shape_vec(IxDyn(&vec![batch_size, oh, ow, oc]), outp).unwrap()]
  }
}
