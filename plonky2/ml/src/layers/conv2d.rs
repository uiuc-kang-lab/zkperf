use std::{collections::HashMap, rc::Rc, marker::PhantomData};

use crate::{
  gadgets::{
    bias_div_round_relu6::BiasDivRoundRelu6Circuit,
    gadget::{Gadget, GadgetConfig, GadgetType},
  },
  layers::fully_connected::{FullyConnectedCircuit, FullyConnectedConfig},
};
use ndarray::{Array, Axis, IxDyn, Slice};
use plonky2::{
  field::extension::Extendable, hash::hash_types::RichField, iop::target::Target,
  plonk::{circuit_builder::CircuitBuilder, config::GenericConfig},
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

pub struct Conv2DCircuit<
F: RichField + Extendable<D>,
C: GenericConfig<D, F = F>,
const D: usize> {
  pub config: LayerConfig,
  pub(crate) _marker: PhantomData<C>,
}

impl<
F: RichField + Extendable<D>,
C: GenericConfig<D, F = F>,
const D: usize> Conv2DCircuit<F, C, D> {
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
    // assert_eq!(tensors.len(), 3);
    assert!(tensors.len() <= 3);

    let conv_config = &Self::param_vec_to_config(self.config.layer_params.clone());

    let inp = &tensors[0];
    let weights = &tensors[1];
    let zero_arr = Array::from_elem(IxDyn(&vec![1]), zero.clone());
    let biases = if tensors.len() == 3 {
      &tensors[2]
    } else {
      &zero_arr
    };

    let h: usize = inp.shape()[1];
    let w: usize = inp.shape()[2];

    let ch: usize = weights.shape()[1];
    let cw: usize = weights.shape()[2];

    let (si, sj) = conv_config.stride;

    // B, H, W, C
    assert_eq!(inp.shape().len(), 4);

    let (ph, pw) = if conv_config.padding == PaddingEnum::Same {
      Self::get_padding(h, w, si, sj, ch, cw)
    } else {
      ((0, 0), (0, 0))
    };
    // println!("Padding: {:?}", (ph, pw));
    let padding = vec![[0, 0], [ph.0, ph.1], [pw.0, pw.1], [0, 0]];

    let inp_pad = Self::pad(&inp, padding, &zero);

    let (oh, ow) = Self::out_hw(h, w, si, sj, ch, cw, conv_config.padding);

    let mut inp_cells = vec![];
    let mut weights_cells = vec![];
    let mut biases_cells = vec![];
    let mut input_row_idx = 0;
    let mut weight_row_idx = 0;

    // (output_channels x inp_channels * C_H * C_W)
    for chan_out in 0..weights.shape()[0] {
      weights_cells.push(vec![]);
      for ci in 0..weights.shape()[1] {
        for cj in 0..weights.shape()[2] {
          for ck in 0..weights.shape()[3] {
            weights_cells[weight_row_idx].push(weights[[chan_out, ci, cj, ck]].clone());
          }
        }
      }
      weight_row_idx += 1;
    }

    // (O_H * O_W x inp_channels * C_H * C_W)
    for batch in 0..inp.shape()[0] {
      for i in 0..oh {
        for j in 0..ow {
          inp_cells.push(vec![]);
          for ci in 0..weights.shape()[1] {
            for cj in 0..weights.shape()[2] {
              for ck in 0..weights.shape()[3] {
                let idx_i = i * si + ci;
                let idx_j = j * sj + cj;
                inp_cells[input_row_idx].push(inp_pad[[batch, idx_i, idx_j, ck]].clone());
              }
            }
          }
          input_row_idx += 1;
        }
      }
    }

    for _batch in 0..inp.shape()[0] {
      for _ in 0..oh {
        for _ in 0..ow {
          for chan_out in 0..weights.shape()[0] {
            if tensors.len() == 3 {
              biases_cells.push(biases[chan_out].clone());
            } else {
              biases_cells.push(zero.clone());
            }
          }
        }
      }
    }

    (inp_cells, weights_cells, biases_cells)
  }
}

impl<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>  + 'static, const D: usize> Layer<F, D> for Conv2DCircuit<F, C, D> {
  fn make_circuit(
    &self,
    builder: &mut CircuitBuilder<F, D>,
    tensors: &Vec<Array<Rc<Target>, IxDyn>>,
    constants: &HashMap<i64, Rc<F>>,
    gadget_config: Rc<GadgetConfig>,
    layer_config: &LayerConfig,
    rand_targets: &mut Vec<Target>,
  ) -> Vec<Array<Rc<Target>, IxDyn>> {
    let conv_config = &Self::param_vec_to_config(self.config.layer_params.clone());
    let zero_t = builder.zero();
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

    // let gate = DotProductGate::construct();

    // let flat_rows = (0..batch_size * oh * ow * oc * ic)
    //   .map(|_| builder.add_gate(gate.clone(), vec![**zero]))
    //   .collect::<Vec<_>>();
    // let rows = Array::from_shape_vec(IxDyn(&[batch_size, oh, ow, oc, ic]), flat_rows).unwrap();

    let outp_flat = match conv_config.conv_type {
      ConvLayerEnum::Conv2D => {
        let fc_circuit = FullyConnectedCircuit::<F, C, D> {
          config: FullyConnectedConfig::construct(false),
            _marker: std::marker::PhantomData,
        };

        let conv_size = splat_inp[0].len();

        let flattened_inp: Vec<_> = splat_inp.into_iter().flat_map(|x| x.into_iter()).collect();
        let flattened_weights = splat_weights
          .into_iter()
          .flat_map(|x| x.into_iter())
          .collect::<Vec<_>>();

        let out_channels = weights.shape()[0];
        let inp_array =
          Array::from_shape_vec(IxDyn(&vec![batch_size * oh * ow, conv_size]), flattened_inp)
            .unwrap();
        let weights_array =
          Array::from_shape_vec(IxDyn(&vec![out_channels, conv_size]), flattened_weights).unwrap();

        let outp_slice = fc_circuit.make_circuit(
          builder,
          &vec![weights_array, inp_array],
          constants,
          gadget_config.clone(),
          layer_config,
          rand_targets
        );

        let outp_flat = outp_slice[0]
          .t()
          .into_iter()
          .map(|x| (**x).clone())
          .collect::<Vec<_>>();
        // let outp_flat : Vec<Target>= dps.into_iter().flat_map(|x| x.into_iter()).collect();
        outp_flat
      }
      ConvLayerEnum::DepthwiseConv2D => panic!("DepthwiseConv2D is unimplemented"),
    };

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
    vec![Array::from_shape_vec(IxDyn(&vec![batch_size, oh, ow, oc]), outp).unwrap()]
  }
}

impl <F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize> GadgetConsumer for Conv2DCircuit<F, C, D> {
  fn used_gadgets(&self, _layer_params: Vec<i64>) -> Vec<crate::gadgets::gadget::GadgetType> {
    vec![GadgetType::BiasDivRoundRelu6, GadgetType::InputLookup]
  }
}