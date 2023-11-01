use std::{collections::HashMap, rc::Rc};

use crate::gadgets::{
    bias_div_round_relu6::BiasDivRoundRelu6Circuit,
    dot_prod::DotProductCircuit,
    gadget::{Gadget, GadgetConfig, GadgetType},
  };
use ndarray::{Array, Axis, IxDyn, Slice};
use plonky2::{
  field::extension::Extendable, hash::hash_types::RichField, iop::target::Target,
  plonk::circuit_builder::CircuitBuilder,
};

use super::layer::{ActivationType, GadgetConsumer, Layer, LayerConfig};

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

  pub fn get_padding(
    h: usize,
    w: usize,
    si: usize,
    sj: usize,
    ci: usize,
    cj: usize,
  ) -> ((usize, usize), (usize, usize)) {
    let ph = if h % si == 0 {
      (ci as i64 - sj as i64).max(0)
    } else {
      (ci as i64 - (h % si) as i64).max(0)
    } as usize;
    let pw = if w % sj == 0 {
      (cj as i64 - sj as i64).max(0)
    } else {
      (cj as i64 - (w % sj) as i64).max(0)
    } as usize;
    ((ph / 2, ph - ph / 2), (pw / 2, pw - pw / 2))
  }

  pub fn pad<G: Clone>(
    input: &Array<Rc<G>, IxDyn>,
    padding: Vec<[usize; 2]>,
    pad_val: &Rc<G>,
  ) -> Array<Rc<G>, IxDyn> {
    let tmp = input.iter().collect();
    let input = Array::from_shape_vec(input.raw_dim(), tmp).unwrap();
    assert_eq!(input.ndim(), padding.len());
    let mut padded_shape = input.raw_dim();
    for (ax, (&ax_len, &[pad_lo, pad_hi])) in input.shape().iter().zip(&padding).enumerate() {
      padded_shape[ax] = ax_len + pad_lo + pad_hi;
    }

    let mut padded = Array::from_elem(padded_shape, pad_val);
    let padded_dim = padded.raw_dim();
    {
      // Select portion of padded array that needs to be copied from the
      // original array.
      let mut orig_portion = padded.view_mut();
      for (ax, &[pad_lo, pad_hi]) in padding.iter().enumerate() {
        orig_portion.slice_axis_inplace(
          Axis(ax),
          Slice::from(pad_lo as isize..padded_dim[ax] as isize - (pad_hi as isize)),
        );
      }
      // Copy the data from the original array.
      orig_portion.assign(&input.view());
    }

    let dim = padded.raw_dim();
    let tmp = padded.into_iter().map(|x| x.clone()).collect();
    let padded = Array::from_shape_vec(dim, tmp).unwrap();

    padded
  }

  pub fn splat<G: Clone>(
    &self,
    tensors: &Vec<Array<Rc<G>, IxDyn>>,
    zero: Rc<G>,
  ) -> (Vec<Vec<Rc<G>>>, Vec<Vec<Rc<G>>>, Vec<Rc<G>>) {
    let input = &tensors[0];
    let weights = &tensors[1];
    let biases = &tensors[2];

    assert_eq!(tensors.len(), 3);
    assert_eq!(input.shape().len(), 4);
    assert_eq!(weights.shape().len(), 4);
    assert_eq!(input.shape()[0], 1);

    let conv_config = &Self::param_vec_to_config(self.config.layer_params.clone());
    let strides = conv_config.stride;

    let h: usize = input.shape()[1];
    let w: usize = input.shape()[2];
    let ch: usize = weights.shape()[1];
    let cw: usize = weights.shape()[2];
    let (si, sj) = conv_config.stride;
    let (oh, ow) = Self::out_hw(h, w, si, sj, ch, cw, conv_config.padding);

    let (ph, pw) = if conv_config.padding == PaddingEnum::Same {
      Self::get_padding(h, w, si, sj, ch, cw)
    } else {
      ((0, 0), (0, 0))
    };

    let padding = vec![[0, 0], [ph.0, ph.1], [pw.0, pw.1], [0, 0]];

    let inp_pad = Self::pad(&input, padding, &zero);

    let mut inp_cells = vec![];
    let mut weight_cells = vec![];
    let mut biases_cells = vec![];
    let mut row_idx = 0;

    for i in 0..oh {
      for j in 0..ow {
        for chan_out in 0..weights.shape()[0] {
          inp_cells.push(vec![]);
          weight_cells.push(vec![]);
          biases_cells.push(biases[[chan_out]].clone());
          for chan_in in 0..weights.shape()[3] {
            for ci in 0..weights.shape()[1] {
              for cj in 0..weights.shape()[2] {
                let idx_i = i * strides.0 + ci;
                let idx_j = j * strides.1 + cj;

                // println!("i: {}, {}, {}, {}", 0, idx_i, idx_j, chan_in);
                // println!("w: {}, {}, {}, {}", chan_out, ci, cj, chan_in);
                inp_cells[row_idx].push(inp_pad[[0, idx_i, idx_j, chan_in]].clone());
                weight_cells[row_idx].push(weights[[chan_out, ci, cj, chan_in]].clone());
              }
            }
          }
          row_idx += 1;
        }
      }
    }
    println!(
      "len: {}, prod: {}",
      inp_cells.len(),
      oh * ow * weights.shape()[0] * weights.shape()[3]
    );

    (inp_cells, weight_cells, biases_cells)
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
    let zero_t = builder.zero();
    let zero = constants.get(&0).unwrap();
    // let z_target = builder.constant(**zero);

    let inp = &tensors[0];
    let weights = &tensors[1];
    let h = inp.shape()[1];
    let w = inp.shape()[2];
    assert!(h == w);
    // TODO implement adder for h > DOTPROD_SIZE

    let ch = weights.shape()[1];
    let cw = weights.shape()[2];
    let oc = weights.shape()[0];
    // let ic = inp.shape()[3];
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

    let (splat_inp, splat_weights, splat_biases) = match conv_config.conv_type {
      ConvLayerEnum::Conv2D => self.splat(tensors, Rc::new(zero_t)),
      ConvLayerEnum::DepthwiseConv2D => panic!("DepthwiseConv2D is unimplemented"),
    };

    let outp_flat = match conv_config.conv_type {
      ConvLayerEnum::Conv2D => {
        let dot_prod_circuit = DotProductCircuit::construct(gadget_config.clone());
        let mut outp_flat = vec![];
        for (inp_vec, weight_vec) in splat_inp.iter().zip(splat_weights.iter()) {
          let inp_vec = inp_vec.iter().map(|x| x.as_ref()).collect::<Vec<_>>();
          let weight_vec = weight_vec.iter().map(|x| x.as_ref()).collect::<Vec<_>>();
          let vec_inputs = vec![inp_vec, weight_vec];
          let constants = vec![**zero];
          let outp =
            dot_prod_circuit.make_circuit(builder, &vec_inputs, &constants, gadget_config.clone());
          outp_flat.push(outp[0].clone());
        }
        println!("outp_flat: {:?}", outp_flat.len());

        outp_flat
      }
      ConvLayerEnum::DepthwiseConv2D => panic!("DepthwiseConv2D is unimplemented"),
    };

    // TODO: assumes padding 0, stride 1
    // let mut conv_outp = vec![];
    // for batch in 0..batch_size {
    //   for x in 0..oh {
    //     for y in 0..ow {
    //       for chan_out in 0..oc {
    //         for chan_in in 0..ic {
    //           let row = rows[[batch, x, y, chan_out, chan_in]];
    //           for i in 0..DOTPROD_SIZE {
    //             for j in 0..DOTPROD_SIZE {
    //               if i < ch && j < cw {
    //                 builder.connect(
    //                   *inp[[batch, x + i, y + j, chan_in]],
    //                   Target::wire(row, DotProductGate::wire_ijth_input(i, j)),
    //                 );
    //                 builder.connect(
    //                   *weights[[chan_out, i, j, chan_in]],
    //                   Target::wire(row, DotProductGate::wire_ijth_weight(i, j)),
    //                 );
    //               } else {
    //                 builder.connect(
    //                   z_target,
    //                   Target::wire(row, DotProductGate::wire_ijth_input(i, j)),
    //                 );
    //                 builder.connect(
    //                   zero_target,
    //                   Target::wire(row, DotProductGate::wire_ijth_weight(i, j)),
    //                 );
    //               }
    //             }
    //           }
    //         }
    //         conv_outp.push(builder.add_many((0..ic).map(|i| {
    //           Target::wire(
    //             rows[[batch, x, y, chan_out, i]],
    //             DotProductGate::wire_output(),
    //           )
    //         })));
    //       }
    //     }
    //   }
    // }

    let mut biases = vec![];
    for bias in splat_biases.iter() {
      biases.push(bias.as_ref());
    }

    // bdr outputs interleaved [(relu'd, div'd), (relu'd, div'd), ...]
    // Uninterleave depending on whether or not we're doing the relu
    let bdr_circuit = BiasDivRoundRelu6Circuit::construct(gadget_config.clone());
    let outp_flat = outp_flat.iter().map(|x| x).collect::<Vec<_>>();
    let outp = bdr_circuit.make_circuit(builder, &vec![outp_flat, biases], &vec![], gadget_config);
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
    println!("outp: {}", outp.len());
    vec![Array::from_shape_vec(IxDyn(&vec![batch_size, oh, ow, oc]), outp).unwrap()]
  }
}

impl GadgetConsumer for Conv2DCircuit {
  fn used_gadgets(&self, _layer_params: Vec<i64>) -> Vec<crate::gadgets::gadget::GadgetType> {
    vec![GadgetType::BiasDivRoundRelu6, GadgetType::InputLookup]
  }
}
