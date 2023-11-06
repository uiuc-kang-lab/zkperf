use std::{collections::HashMap, rc::Rc, marker::PhantomData};

use ndarray::{Array, IxDyn};

use plonky2::{
  field::extension::Extendable, hash::hash_types::RichField, iop::target::Target,
  plonk::{circuit_builder::CircuitBuilder, config::GenericConfig},
};

use crate::{
  gadgets::{
    gadget::{Gadget, GadgetConfig, GadgetType},
    var_div::DivRoundCircuit,
  },
  layers::{conv2d::Conv2DCircuit, conv2d::PaddingEnum, layer::LayerConfig},
};

use super::layer::{GadgetConsumer, Layer};

pub struct AvgPool2DCircuit<
F: RichField + Extendable<D>,
C: GenericConfig<D, F = F>,
const D: usize,
> {
  pub(crate) _marker: PhantomData<C>,
}

impl<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>  + 'static, const D: usize> Layer<F, D> for AvgPool2DCircuit<F, C, D>{
  fn make_circuit(
    &self,
    builder: &mut CircuitBuilder<F, D>,
    tensors: &Vec<Array<Rc<Target>, IxDyn>>,
    _constants: &HashMap<i64, Rc<F>>,
    gadget_config: Rc<GadgetConfig>,
    layer_config: &LayerConfig,
    _rand_targets: &mut Vec<Target>
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

    let (oh, ow) = Conv2DCircuit::<F, C, D>::out_hw(h, w, sh, sw, fx, fy, PaddingEnum::Valid);
    let batch_size = inp.shape()[0];

    let mut sum_outp = vec![];
    for i in 0..oh {
      for j in 0..ow {
        for chan in 0..c {
          let pool_targets = (0..fx)
            .map(|x| (0..fy).map(move |y| inp[[0, x + i * sh, y + j * sw, chan]].as_ref()))
            .flatten()
            .collect::<Vec<_>>();
          sum_outp.push(builder.add_many(pool_targets));
        }
      }
    }

    let sum_ref = sum_outp.iter().collect::<Vec<_>>();

    let div_gadget = DivRoundCircuit::construct(gadget_config.clone());
    let div_outp = div_gadget.make_circuit(
      builder,
      &vec![sum_ref],
      &vec![F::from_canonical_usize(div)],
      gadget_config,
    );

    let outp = div_outp.iter().map(|t| Rc::new(*t)).collect::<Vec<_>>();

    vec![Array::from_shape_vec(IxDyn(&vec![batch_size, oh, ow, c]), outp).unwrap()]
  }
}

impl<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize> GadgetConsumer for AvgPool2DCircuit<F, C, D> {
  fn used_gadgets(&self, _layer_params: Vec<i64>) -> Vec<crate::gadgets::gadget::GadgetType> {
    vec![GadgetType::DivRound]
  }
}
