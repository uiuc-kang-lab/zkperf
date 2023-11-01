use std::{collections::HashMap, marker::PhantomData, rc::Rc};

use ndarray::{Array, Axis, IxDyn};

use plonky2::{
  field::extension::Extendable,
  hash::hash_types::RichField,
  iop::target::Target,
  plonk::{circuit_builder::CircuitBuilder, config::GenericConfig},
};

use crate::{
  gadgets::gadget::GadgetConfig,
  layers::{
    fully_connected::{FullyConnectedCircuit, FullyConnectedConfig},
    layer::LayerConfig,
  },
};

use super::layer::{GadgetConsumer, Layer};

pub struct BatchMatMulCircuit<
  F: RichField + Extendable<D>,
  C: GenericConfig<D, F = F>,
  const D: usize,
> {
  pub(crate) _marker: PhantomData<C>,
}

impl<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>  + 'static, const D: usize> Layer<F, D>
  for BatchMatMulCircuit<F, C, D>
{
  fn make_circuit(
    &self,
    builder: &mut CircuitBuilder<F, D>,
    tensors: &Vec<Array<Rc<Target>, IxDyn>>,
    constants: &HashMap<i64, Rc<F>>,
    gadget_config: Rc<GadgetConfig>,
    layer_config: &LayerConfig,
  ) -> Vec<Array<Rc<Target>, IxDyn>> {
    let inp1 = &tensors[0];
    let inp2 = &tensors[1];
    println!("inp1: {:?}", inp1.shape());
    println!("inp2: {:?}", inp2.shape());

    assert_eq!(inp1.ndim(), 3);
    assert_eq!(inp2.ndim(), 3);
    assert_eq!(inp1.shape()[0], inp2.shape()[0]);

    let adj_y = layer_config.layer_params[1] == 1;
    if adj_y {
      assert_eq!(inp1.shape()[2], inp2.shape()[2]);
    } else {
      assert_eq!(inp1.shape()[2], inp2.shape()[1]);
    }

    let out_shape = if adj_y {
      vec![inp1.shape()[0], inp1.shape()[1], inp2.shape()[1]]
    } else {
      vec![inp1.shape()[0], inp1.shape()[1], inp2.shape()[2]]
    };

    let fc_circuit = FullyConnectedCircuit::<F, C, D> {
      config: FullyConnectedConfig::construct(true),
      _marker: PhantomData,
    };

    let mut outp = vec![];
    for i in 0..inp1.shape()[0] {
      let inp1_slice = inp1.index_axis(Axis(0), i).to_owned();
      // Due to tensorflow BS, transpose the "weights"
      let inp2_slice = if adj_y {
        inp2.index_axis(Axis(0), i).to_owned()
      } else {
        inp2.index_axis(Axis(0), i).t().to_owned()
      };
      println!("inp1_slice: {:?}", inp1_slice.shape());
      println!("inp2_slice: {:?}", inp2_slice.shape());
      // Batch MM doesn't have a fused activation, so insert it here
      // TODO: consider putting this in the converter?
      let tmp_config = LayerConfig {
        layer_params: vec![0],
        ..layer_config.clone()
      };
      let outp_slice = fc_circuit.make_circuit(
        builder,
        &vec![inp1_slice, inp2_slice],
        constants,
        gadget_config.clone(),
        &tmp_config,
      );
      outp.extend(outp_slice[0].iter().map(|x| x.clone()).collect::<Vec<_>>());
    }

    let outp = Array::from_shape_vec(IxDyn(out_shape.as_slice()), outp).unwrap();
    vec![outp]
  }
}

impl<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize> GadgetConsumer
  for BatchMatMulCircuit<F, C, D>
{
  fn used_gadgets(&self, _layer_params: Vec<i64>) -> Vec<crate::gadgets::gadget::GadgetType> {
    vec![]
  }
}
