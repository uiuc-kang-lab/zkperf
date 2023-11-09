use plonky2::field::extension::Extendable;
use plonky2::field::packed::PackedField;
use plonky2::gates::gate::Gate;
use plonky2::gates::packed_util::PackedEvaluableBase;
use plonky2::gates::util::StridedConstraintConsumer;
use plonky2::hash::hash_types::RichField;
use plonky2::iop::ext_target::ExtensionTarget;
use plonky2::iop::generator::{GeneratedValues, SimpleGenerator, WitnessGeneratorRef};
use plonky2::iop::target::Target;
use plonky2::iop::witness::{PartitionWitness, Witness, WitnessWrite};
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::{CircuitConfig, CommonCircuitData};
use plonky2::plonk::vars::{
  EvaluationTargets, EvaluationVars, EvaluationVarsBase, EvaluationVarsBaseBatch,
  EvaluationVarsBasePacked,
};
use plonky2::util::serialization::{Buffer, IoResult, Read, Write};
use plonky2_field::types::Field;

#[derive(Debug, Clone)]
pub struct BiasDivRoundGate {
  pub num_ops: usize,
}

impl BiasDivRoundGate {
  pub fn new_from_config(config: &CircuitConfig) -> Self {
    Self {
      num_ops: Self::num_ops(config),
    }
  }

  pub(crate) fn num_ops(config: &CircuitConfig) -> usize {
    let wires_per_entry = 5;
    config.num_routed_wires / wires_per_entry
  }

  pub fn wire_ith_input(i: usize) -> usize {
    5 * i
  }

  pub fn wire_ith_bias(i: usize) -> usize {
    5 * i + 1
  }

  pub fn wire_ith_div(i: usize) -> usize {
    5 * i + 2
  }

  pub fn wire_ith_mod(i: usize) -> usize {
    5 * i + 3
  }

  pub fn wire_ith_mod_div_lookup(i: usize) -> usize {
    5 * i + 4
  }
}

impl<F: RichField + Extendable<D>, const D: usize> Gate<F, D> for BiasDivRoundGate {
  fn id(&self) -> String {
    format!("{self:?}")
  }

  fn serialize(&self, dst: &mut Vec<u8>, _common_data: &CommonCircuitData<F, D>) -> IoResult<()> {
    dst.write_usize(self.num_ops)
  }

  fn deserialize(src: &mut Buffer, _common_data: &CommonCircuitData<F, D>) -> IoResult<Self> {
    let num_ops = src.read_usize()?;
    Ok(Self { num_ops })
  }

  // TODO change to deriving constraints from shift_min_val
  fn eval_unfiltered(&self, vars: EvaluationVars<F, D>) -> Vec<F::Extension> {
    let mut constraints = vec![];
    let sf = vars.local_constants[0];
    let two = F::Extension::from_canonical_u64(2);

    for i in 0..self.num_ops {
      let inp = vars.local_wires[Self::wire_ith_input(i)];
      let bias = vars.local_wires[Self::wire_ith_bias(i)];
      let div_res = vars.local_wires[Self::wire_ith_div(i)];
      let mod_res = vars.local_wires[Self::wire_ith_mod(i)];
      let mod_div_lookup = vars.local_wires[Self::wire_ith_mod_div_lookup(i)];

      // (div - bias) * 2 * sf + mod = 2 * inp + sf
      constraints.push(two * inp + sf - (sf * two * (div_res - bias) + mod_res));

      // 2 * sf > mod >= 0
      // 2 * sf - mod = mod_diff + 1
      constraints.push(two * sf - mod_res - F::Extension::ONE - mod_div_lookup);
    }
    constraints
  }

  fn eval_unfiltered_base_one(
    &self,
    _vars: EvaluationVarsBase<F>,
    _yield_constr: StridedConstraintConsumer<F>,
  ) {
    panic!("use eval_unfiltered_base_packed instead");
  }

  fn eval_unfiltered_base_batch(&self, vars_base: EvaluationVarsBaseBatch<F>) -> Vec<F> {
    self.eval_unfiltered_base_batch_packed(vars_base)
  }

  fn eval_unfiltered_circuit(
    &self,
    builder: &mut CircuitBuilder<F, D>,
    vars: EvaluationTargets<D>,
  ) -> Vec<ExtensionTarget<D>> {
    let mut constraints = vec![];
    let two = builder.constant_extension(F::Extension::from_canonical_u64(2));
      let one = builder.constant_extension(F::Extension::ONE);
    for i in 0..self.num_ops {
      let sf = vars.local_constants[0];

      let inp = vars.local_wires[Self::wire_ith_input(i)];
      let div_res = vars.local_wires[Self::wire_ith_div(i)];
      let bias = vars.local_wires[Self::wire_ith_bias(i)];
      let mod_res = vars.local_wires[Self::wire_ith_mod(i)];
      let mod_div_lookup = vars.local_wires[Self::wire_ith_mod_div_lookup(i)];

      // (div - bias) * 2 * sf + mod = 2 * inp + sf
      let constr0 = {
        let b = builder.sub_extension(div_res, bias);
        let u = builder.mul_many_extension([sf, two, b]);
        let v = builder.add_extension(u, mod_res);
        let ti = builder.mul_add_extension(two, inp, sf);
        builder.sub_extension(ti, v)
      };
      let constr1 = {
        let t = builder.add_many_extension([mod_res, one, mod_div_lookup]);
        builder.mul_sub_extension(two, sf, t)
      };

      constraints.extend([constr0, constr1].iter());
    }

    constraints
  }

  fn generators(&self, row: usize, local_constants: &[F]) -> Vec<WitnessGeneratorRef<F, D>> {
    (0..self.num_ops)
      .map(|i| {
        WitnessGeneratorRef::new(
          BiasDivRoundGenerator {
            row,
            sf: local_constants[0],
            shift_min_val: local_constants[1],
            div_outp_min_val: local_constants[2],
            i,
          }
          .adapter(),
        )
      })
      .collect()
  }

  fn num_wires(&self) -> usize {
    self.num_ops * 5
  }

  fn num_constants(&self) -> usize {
    3
  }

  fn degree(&self) -> usize {
    2
  }

  fn num_constraints(&self) -> usize {
    self.num_ops * 2
  }
}

impl<F: RichField + Extendable<D>, const D: usize> PackedEvaluableBase<F, D> for BiasDivRoundGate {
  fn eval_unfiltered_base_packed<P: PackedField<Scalar = F>>(
    &self,
    vars: EvaluationVarsBasePacked<P>,
    mut yield_constr: StridedConstraintConsumer<P>,
  ) {
    let sf = vars.local_constants[0];
    let two = P::ONES + P::ONES;
    for i in 0..self.num_ops {
      let inp = vars.local_wires[Self::wire_ith_input(i)];
      let div_res = vars.local_wires[Self::wire_ith_div(i)];
      let bias = vars.local_wires[Self::wire_ith_bias(i)];
      let mod_res = vars.local_wires[Self::wire_ith_mod(i)];
      let mod_div_lookup = vars.local_wires[Self::wire_ith_mod_div_lookup(i)];

      // (div - bias) * 2 * sf + mod = 2 * inp + sf
      yield_constr.one(two * inp + sf - (sf * two * (div_res - bias) + mod_res));

      yield_constr.one(two * sf - mod_res - P::ONES - mod_div_lookup);
    }
  }
}

#[derive(Clone, Debug, Default)]
pub struct BiasDivRoundGenerator<F: RichField + Extendable<D>, const D: usize> {
  row: usize,
  sf: F,
  shift_min_val: F,
  div_outp_min_val: F,
  i: usize,
}

impl<F: RichField + Extendable<D>, const D: usize> SimpleGenerator<F, D>
  for BiasDivRoundGenerator<F, D>
{
  fn id(&self) -> String {
    "BiasDivRoundGenerator".to_string()
  }

  fn dependencies(&self) -> Vec<Target> {
    vec![
      BiasDivRoundGate::wire_ith_input(self.i),
      BiasDivRoundGate::wire_ith_bias(self.i),
    ]
    .iter()
    .map(|&i| Target::wire(self.row, i))
    .collect()
  }

  fn run_once(&self, witness: &PartitionWitness<F>, out_buffer: &mut GeneratedValues<F>) {
    let get_wire = |wire: usize| -> F { witness.get_target(Target::wire(self.row, wire)) };

    let inp = get_wire(BiasDivRoundGate::wire_ith_input(self.i));
    let div_outp_min_val_i64 = self.div_outp_min_val.to_canonical_u64() as i64;
    let div_inp_min_val_pos_i64 = -(self.shift_min_val.to_canonical_u64() as i64);
    let div_inp_min_val_pos = F::from_canonical_u64(div_inp_min_val_pos_i64 as u64);

    let div_val = self.sf.to_canonical_u64() as i64;

    let bias = get_wire(BiasDivRoundGate::wire_ith_bias(self.i)) + div_inp_min_val_pos;
    let bias = bias.to_canonical_u64() as i64 - div_inp_min_val_pos_i64;

    let x_pos = inp + div_inp_min_val_pos;
    let inp = x_pos.to_canonical_u64() as i64;
    let div_inp = 2 * inp + div_val;
    let div_res = div_inp / (2 * div_val) - div_inp_min_val_pos_i64 / div_val;
    let mod_res = div_inp % (2 * div_val);

    let div_res = div_res + bias;
    let div_res = F::from_canonical_u64((div_res - div_outp_min_val_i64) as u64)
      - F::from_canonical_u64(-div_outp_min_val_i64 as u64);
    let mod_div_lookup = 2 * div_val - mod_res - 1;

    let div_res_target = Target::wire(self.row, BiasDivRoundGate::wire_ith_div(self.i));
    let mod_res_target = Target::wire(self.row, BiasDivRoundGate::wire_ith_mod(self.i));
    let mod_div_lookup_target = Target::wire(self.row, BiasDivRoundGate::wire_ith_mod_div_lookup(self.i));

    // address outp outside of gate since its value relies on lut
    out_buffer.set_target(div_res_target, div_res);
    out_buffer.set_target(mod_res_target, F::from_canonical_u64(mod_res as u64));
    out_buffer.set_target(
      mod_div_lookup_target,
      F::from_canonical_u64(mod_div_lookup as u64),
    );
  }

  fn serialize(&self, dst: &mut Vec<u8>, _common_data: &CommonCircuitData<F, D>) -> IoResult<()> {
    dst.write_usize(self.row)?;
    dst.write_field(self.sf)?;
    dst.write_field(self.shift_min_val)?;
    dst.write_field(self.div_outp_min_val)?;
    dst.write_usize(self.i)
  }

  fn deserialize(src: &mut Buffer, _common_data: &CommonCircuitData<F, D>) -> IoResult<Self> {
    let row = src.read_usize()?;
    let sf = src.read_field()?;
    let shift_min_val = src.read_field()?;
    let div_outp_min_val = src.read_field()?;
    let i = src.read_usize()?;
    Ok(Self {
      row,
      sf,
      shift_min_val,
      div_outp_min_val,
      i,
    })
  }
}
