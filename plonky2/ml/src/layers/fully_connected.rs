use std::{collections::HashMap, marker::PhantomData, rc::Rc};

use ndarray::{Array, ArrayView, Axis, IxDyn};
use plonky2::{
  hash::hash_types::RichField,
  iop::{
    generator::{GeneratedValues, SimpleGenerator},
    target::Target,
    witness::{PartitionWitness, Witness, WitnessWrite},
  },
  plonk::{
    circuit_builder::CircuitBuilder, circuit_data::CommonCircuitData, config::GenericConfig,
  },
  util::serialization::{Buffer, IoResult, Read, Write},
};
use plonky2_field::extension::Extendable;

use crate::{
  gadgets::{
    dot_prod::DotProductCircuit,
    gadget::{Gadget, GadgetConfig, GadgetType},
    nonlinear::relu::ReluCircuit,
    var_div::DivRoundCircuit,
  },
  layers::layer::ActivationType,
};

use super::layer::{GadgetConsumer, Layer, LayerConfig};

pub struct FullyConnectedConfig {
  pub normalize: bool, // Should be true
}

impl FullyConnectedConfig {
  pub fn construct(normalize: bool) -> Self {
    Self { normalize }
  }
}

pub struct FullyConnectedCircuit<
  F: RichField + Extendable<D>,
  C: GenericConfig<D, F = F>,
  const D: usize,
> {
  pub config: FullyConnectedConfig,
  pub(crate) _marker: PhantomData<C>,
}

impl<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize>
  FullyConnectedCircuit<F, C, D>
{
  fn get_activation(&self, layer_params: &Vec<i64>) -> ActivationType {
    let activation = layer_params[0];
    match activation {
      0 => ActivationType::None,
      1 => ActivationType::Relu,
      _ => panic!("Unsupported activation type for fully connected"),
    }
  }
}

impl<F: RichField + Extendable<D>, C: GenericConfig<D, F = F> + 'static, const D: usize> Layer<F, D>
  for FullyConnectedCircuit<F, C, D>
{
  fn make_circuit(
    &self,
    builder: &mut CircuitBuilder<F, D>,
    tensors: &Vec<Array<Rc<Target>, IxDyn>>,
    constants: &HashMap<i64, Rc<F>>,
    gadget_config: Rc<GadgetConfig>,
    layer_config: &LayerConfig,
    rand_targets: &mut Vec<Target>,
  ) -> Vec<Array<Rc<Target>, IxDyn>> {
    assert!(tensors.len() <= 3);
    let activation = self.get_activation(&layer_config.layer_params);

    let input = tensors[0].map(|t| **t);
    let weight = tensors[1].t().into_owned().map(|t| **t);
    let shape = [input.shape()[0], weight.shape()[1]];
    let mm_outp_flat = builder.add_virtual_targets(shape[0] * shape[1]);
    let mm_result = Array::from_shape_vec(IxDyn(&shape), mm_outp_flat).unwrap();
    let zero = constants.get(&0).unwrap().as_ref();

    // Compute and assign the result
    let dim = [input.shape()[0], input.shape()[1], weight.shape()[1]];
    builder.add_simple_generator(MatMulGenerator {
      dim,
      input: input.clone(),
      weight: weight.clone(),
      outp: mm_result.clone(),
    });

    // Fill random values
    let r1_ref = {
      let mut ts = vec![];
      for _ in 0..mm_result.shape()[0] {
        let t = builder.add_virtual_public_input();
        ts.push(t);
      }
      builder.register_public_inputs(&ts);
      ts
    };
    let r2_ref = {
      let mut ts = vec![];
      for _ in 0..mm_result.shape()[1] {
        let t = builder.add_virtual_public_input();
        ts.push(t);
      }
      builder.register_public_inputs(&ts);
      ts
    };
    rand_targets.extend(r1_ref.iter());
    rand_targets.extend(r2_ref.iter());

    let r1_ref = r1_ref.iter().collect::<Vec<_>>();
    let r2_ref = r2_ref.iter().collect::<Vec<_>>();

    // // Compute r1 * result
    let dot_prod_circuit = DotProductCircuit::construct(gadget_config.clone());
    let mut r1_res = vec![];
    // println!("r1_ref: {:?}", r1_ref.len());
    // println!("r2_ref: {:?}", r2_ref.len());
    // println!("mm_result: {:?}", mm_result.shape());
    for i in 0..mm_result.shape()[1] {
      let tmp = mm_result.index_axis(Axis(1), i);
      let mm_ci = tmp.iter().collect::<Vec<_>>();
      let r1_res_i = dot_prod_circuit.make_circuit(
        builder,
        &vec![mm_ci, r1_ref.clone()],
        &vec![*zero],
        gadget_config.clone(),
      );
      r1_res.push(r1_res_i[0].clone());
    }

    // Compute r1 * result * r2
    let r1_res_ref = r1_res.iter().collect::<Vec<_>>();
    let r1_res_r2 = dot_prod_circuit.make_circuit(
      builder,
      &vec![r1_res_ref, r2_ref.clone()],
      &vec![*zero],
      gadget_config.clone(),
    );
    let r1_res_r2 = r1_res_r2[0].clone();
    // println!("r1_res_r2: {:?}", r1_res_r2);

    // Compute r1 * input
    let mut r1_input = vec![];

    let ndim = input.ndim();
    let input = if ndim == 2 {
      ArrayView::from(&input)
    } else {
      input.index_axis(Axis(0), 0)
    };
    // println!("input: {:?}", input.shape());

    // println!("r1_ref: {:?}", r1_ref.len());
    for i in 0..input.shape()[1] {
      let tmp = input.index_axis(Axis(1), i);
      let input_ci = tmp.iter().collect::<Vec<_>>();
      let r1_input_i = dot_prod_circuit.make_circuit(
        builder,
        &vec![input_ci, r1_ref.clone()],
        &vec![*zero],
        gadget_config.clone(),
      );
      r1_input.push(r1_input_i[0].clone());
    }

    // Compute weight * r2
    let mut weight_r2 = vec![];
    for i in 0..weight.shape()[0] {
      let tmp = weight.index_axis(Axis(0), i);
      let weight_ci = tmp.iter().collect::<Vec<_>>();
      let weight_r2_i = dot_prod_circuit.make_circuit(
        builder,
        &vec![weight_ci, r2_ref.clone()],
        &vec![*zero],
        gadget_config.clone(),
      );
      weight_r2.push(weight_r2_i[0].clone());
    }

    // Compute (r1 * input) * (weight * r2)
    let r1_input_ref = r1_input.iter().collect::<Vec<_>>();
    let weight_r2_ref = weight_r2.iter().collect::<Vec<_>>();
    let r1_inp_weight_r2 = dot_prod_circuit.make_circuit(
      builder,
      &vec![r1_input_ref, weight_r2_ref],
      &vec![*zero],
      gadget_config.clone(),
    );

    let r1_inp_weight_r2 = r1_inp_weight_r2[0].clone();

    // check equality
    builder.connect(r1_res_r2, r1_inp_weight_r2);

    let final_result_flat = if self.config.normalize {
      let mm_flat = mm_result.iter().map(|t| &*t).collect::<Vec<_>>();

      let div_gadget = DivRoundCircuit::construct(gadget_config.clone());
      let sf = constants
        .get(&(gadget_config.scale_factor as i64))
        .unwrap()
        .as_ref();
      let mm_div =
        div_gadget.make_circuit(builder, &vec![mm_flat], &vec![*sf], gadget_config.clone());

      let mm_div = if tensors.len() == 3 {
        let bias = tensors[2].broadcast(shape.clone()).unwrap();
        let bias = bias.iter().map(|x| x.as_ref()).collect::<Vec<_>>();
        let mm_div = mm_div.iter().collect::<Vec<_>>();
        let mut mm_bias = vec![];
        for i in 0..mm_div.len() {
          mm_bias.push(builder.add(*mm_div[i], *bias[i % bias.len()]));
        }
        mm_bias
      } else {
        mm_div
      };

      let mm_div = if activation == ActivationType::Relu {
        let relu_circuit = ReluCircuit::construct(gadget_config.clone());
        let mm_div = mm_div.iter().collect::<Vec<_>>();
        let vec_inputs = vec![mm_div];
        relu_circuit.make_circuit(builder, &vec_inputs, &vec![], gadget_config)
      } else if activation == ActivationType::None {
        mm_div
      } else {
        panic!("Unsupported activation type");
      };

      mm_div.into_iter().map(|x| Rc::new(x)).collect::<Vec<_>>()
    } else {
      mm_result
        .into_iter()
        .map(|x| Rc::new(x))
        .collect::<Vec<_>>()
    };
    let final_result = Array::from_shape_vec(IxDyn(&shape), final_result_flat).unwrap();

    vec![final_result]
  }
}

#[derive(Debug, Default)]
pub struct MatMulGenerator {
  dim: [usize; 3],
  input: Array<Target, IxDyn>,
  weight: Array<Target, IxDyn>,
  outp: Array<Target, IxDyn>,
}

impl<F: RichField + Extendable<D>, const D: usize> SimpleGenerator<F, D> for MatMulGenerator {
  fn id(&self) -> String {
    "MatMulGenerator".to_string()
  }

  fn dependencies(&self) -> Vec<Target> {
    let inputs = self.input.iter().map(|t| *t);
    let weights = self.weight.iter().map(|t| *t);
    inputs.chain(weights).collect::<Vec<_>>()
  }

  fn run_once(&self, witness: &PartitionWitness<F>, out_buffer: &mut GeneratedValues<F>) {
    let ndim = self.input.ndim();
    let input = if ndim == 2 {
      ArrayView::from(&self.input)
    } else {
      self.input.index_axis(Axis(0), 0)
    };

    let mut outp = vec![];
    for i in 0..input.shape()[0] {
      for j in 0..self.weight.shape()[1] {
        let inp_val = witness.get_target(input[[i, 0]]);
        let weight_val = witness.get_target(self.weight[[0, j]]);
        let mut sum = inp_val * weight_val;
        for k in 1..input.shape()[1] {
          let inp_val = witness.get_target(input[[i, k]]);
          let weight_val = witness.get_target(self.weight[[k, j]]);
          sum = sum + inp_val * weight_val;
        }
        outp.push(sum);
      }
    }

    let out_shape = [input.shape()[0], self.weight.shape()[1]];
    let outp_arr = Array::from_shape_vec(IxDyn(out_shape.as_slice()), outp).unwrap();
    for i in 0..out_shape[0] {
      for j in 0..out_shape[1] {
        out_buffer.set_target(self.outp[[i, j]], outp_arr[[i, j]]);
      }
    }
  }

  fn serialize(&self, dst: &mut Vec<u8>, _common_data: &CommonCircuitData<F, D>) -> IoResult<()> {
    let inputs = self.input.iter().map(|t| *t).collect::<Vec<_>>();
    let weights = self.weight.iter().map(|t| *t).collect::<Vec<_>>();
    let outps = self.outp.iter().map(|t| *t).collect::<Vec<_>>();
    dst.write_usize_vec(&self.dim)?;
    dst.write_target_vec(&inputs)?;
    dst.write_target_vec(&weights)?;
    dst.write_target_vec(&outps)
  }

  fn deserialize(src: &mut Buffer, _common_data: &CommonCircuitData<F, D>) -> IoResult<Self> {
    let dim_vec = src.read_usize_vec()?;
    let dim = [dim_vec[0], dim_vec[1], dim_vec[2]];
    let input_flat = src.read_target_vec()?;
    let weight_flat = src.read_target_vec()?;
    let outp_flat = src.read_target_vec()?;
    let input = Array::from_shape_vec(IxDyn(&[dim[0], dim[1]]), input_flat).unwrap();
    let weight = Array::from_shape_vec(IxDyn(&[dim[1], dim[2]]), weight_flat).unwrap();
    let outp = Array::from_shape_vec(IxDyn(&[dim[0], dim[2]]), outp_flat).unwrap();
    Ok(Self {
      dim,
      input,
      weight,
      outp,
    })
  }
}

impl<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize> GadgetConsumer
  for FullyConnectedCircuit<F, C, D>
{
  fn used_gadgets(&self, layer_params: Vec<i64>) -> Vec<crate::gadgets::gadget::GadgetType> {
    let activation = self.get_activation(&layer_params);
    let mut outp = vec![
      GadgetType::DivRound,
      GadgetType::DotProduct,
      GadgetType::InputLookup,
    ];
    match activation {
      ActivationType::Relu => outp.push(GadgetType::Relu),
      ActivationType::None => (),
      _ => panic!("Unsupported activation type"),
    }
    outp
  }
}
