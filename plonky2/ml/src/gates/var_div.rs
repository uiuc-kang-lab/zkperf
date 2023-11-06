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
use rounded_div::RoundedDiv;

use crate::gadgets::gadget::convert_to_u128;

#[derive(Debug, Clone)]
pub struct DivRoundGate {
  pub num_ops: usize,
}

impl DivRoundGate {
  pub fn new_from_config(config: &CircuitConfig) -> Self {
    Self {
      num_ops: Self::num_ops(config),
    }
  }

  pub(crate) fn num_ops(config: &CircuitConfig) -> usize {
    let wires_per_entry = 4;
    config.num_routed_wires / wires_per_entry
  }

  pub fn wire_ith_input(i: usize) -> usize {
    4 * i
  }

  pub fn wire_ith_output(i: usize) -> usize {
    4 * i + 1
  }

  pub fn wire_ith_remainder(i: usize) -> usize {
    4 * i + 2
  }

  pub fn wire_ith_div_rem_diff(i: usize) -> usize {
    4 * i + 3
  }
}

impl<F: RichField + Extendable<D>, const D: usize> Gate<F, D> for DivRoundGate {
  fn id(&self) -> String {
    "div round".to_string()
  }

  fn serialize(&self, dst: &mut Vec<u8>, _common_data: &CommonCircuitData<F, D>) -> IoResult<()> {
    dst.write_usize(self.num_ops)
  }

  fn deserialize(src: &mut Buffer, _common_data: &CommonCircuitData<F, D>) -> IoResult<Self> {
    let num_ops = src.read_usize()?;
    Ok(Self { num_ops })
  }

  fn eval_unfiltered(&self, vars: EvaluationVars<F, D>) -> Vec<F::Extension> {
    let b = vars.local_constants[0];
    let two = F::Extension::from_canonical_u64(2);
    let mut constraints = vec![];

    for i in 0..self.num_ops {
      let a = vars.local_wires[Self::wire_ith_input(i)];
      let c = vars.local_wires[Self::wire_ith_output(i)];
      let diff = vars.local_wires[Self::wire_ith_div_rem_diff(i)];
      let r = vars.local_wires[Self::wire_ith_remainder(i)];

      // (2 * a + b) = (2 * b) * c + r
      let lhs = a * two + b;
      let rhs = b * two * c + r;
      constraints.push(lhs - rhs);

      // 2 * b - r >= 1 => exists diff >= 0 s.t. 2 * b - r - 1 = diff
      constraints.push(two * b - r - F::Extension::ONE - diff);
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
    let one = builder.constant_extension(F::Extension::ONE);
    let two = builder.constant_extension(F::Extension::from_canonical_u64(2));
    let mut constraints = Vec::with_capacity(2 * self.num_ops);
    for i in 0..self.num_ops {
      let b = vars.local_constants[0];
      let a = vars.local_wires[Self::wire_ith_input(i)];
      let c = vars.local_wires[Self::wire_ith_output(i)];
      let diff = vars.local_wires[Self::wire_ith_div_rem_diff(i)];
      let r = vars.local_wires[Self::wire_ith_remainder(i)];

      // 2 * b
      let bb = builder.mul_extension(b, two);

      // 2 * a + b = 2 * b * c + r
      let constr0 = {
        let lhs = builder.mul_add_extension(a, two, b);
        let rhs = builder.mul_add_extension(bb, c, r);
        builder.sub_extension(lhs, rhs)
      };
      // 2 * b - r >= 1
      let constr1 = {
        let u = builder.add_many_extension([r, one, diff]);
        builder.sub_extension(bb, u)
      };
      constraints.extend([constr0, constr1].iter());
    }

    constraints
  }

  fn generators(&self, row: usize, local_constants: &[F]) -> Vec<WitnessGeneratorRef<F, D>> {
    (0..self.num_ops)
      .map(|i| {
        WitnessGeneratorRef::new(
          DivRoundGenerator {
            row,
            b: local_constants[0],
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
    self.num_ops * 4
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

impl<F: RichField + Extendable<D>, const D: usize> PackedEvaluableBase<F, D> for DivRoundGate {
  fn eval_unfiltered_base_packed<P: PackedField<Scalar = F>>(
    &self,
    vars: EvaluationVarsBasePacked<P>,
    mut yield_constr: StridedConstraintConsumer<P>,
  ) {
    let b = vars.local_constants[0];
    let two = P::ONES + P::ONES;
    for i in 0..self.num_ops {
      let a = vars.local_wires[Self::wire_ith_input(i)];
      let c = vars.local_wires[Self::wire_ith_output(i)];
      let diff = vars.local_wires[Self::wire_ith_div_rem_diff(i)];
      let r = vars.local_wires[Self::wire_ith_remainder(i)];

      let lhs = a * two + b;
      let rhs = b * two * c + r;
      yield_constr.one(lhs - rhs);
      yield_constr.one(two * b - r - P::ONES - diff);
    }
  }
}

#[derive(Clone, Debug, Default)]
pub struct DivRoundGenerator<F: RichField + Extendable<D>, const D: usize> {
  row: usize,
  b: F,
  shift_min_val: F,
  div_outp_min_val: F,
  i: usize,
}

impl<F: RichField + Extendable<D>, const D: usize> SimpleGenerator<F, D>
  for DivRoundGenerator<F, D>
{
  fn id(&self) -> String {
    "DivRoundGenerator".to_string()
  }

  fn dependencies(&self) -> Vec<Target> {
    vec![DivRoundGate::wire_ith_input(self.i)]
      .iter()
      .map(|&i| Target::wire(self.row, i))
      .collect()
  }

  fn run_once(&self, witness: &PartitionWitness<F>, out_buffer: &mut GeneratedValues<F>) {
    let get_wire = |wire: usize| -> F { witness.get_target(Target::wire(self.row, wire)) };
    let a = get_wire(DivRoundGate::wire_ith_input(self.i));
    let b_int = convert_to_u128(&self.b);

    let div_outp_min_val_i64 = self.div_outp_min_val.to_canonical_u64() as i64;

    let div_inp_min_val_pos_i64 = -(self.shift_min_val.to_canonical_u64() as i64);
    let div_inp_min_val_pos_i64 = div_inp_min_val_pos_i64 / (b_int as i64) * (b_int as i64);
    let div_inp_min_val_pos = F::from_canonical_u64(div_inp_min_val_pos_i64 as u64);

    let a_pos = a + div_inp_min_val_pos;
    let a = convert_to_u128(&a_pos);
    // c = (2 * a + b) / (2 * b)
    let c_pos = a.rounded_div(b_int);
    let c = (c_pos as i128 - (div_inp_min_val_pos_i64 as u128 / b_int) as i128) as i64;

    // r = (2 * a + b) % (2 * b)
    let rem_floor = (a as i128) - (c_pos * b_int) as i128;
    let r = 2 * rem_floor + (b_int as i128);
    let r = r as i64;
    let diff = 2 * b_int as i64 - r - 1;

    let output_target = Target::wire(self.row, DivRoundGate::wire_ith_output(self.i));
    let div_rem_diff_target = Target::wire(self.row, DivRoundGate::wire_ith_div_rem_diff(self.i));
    let remainder_target = Target::wire(self.row, DivRoundGate::wire_ith_remainder(self.i));

    let div = {
      let offset = F::from_canonical_u64(-div_outp_min_val_i64 as u64);
      let c = F::from_canonical_u64((c - div_outp_min_val_i64) as u64);
      c - offset
    };

    out_buffer.set_target(div_rem_diff_target, F::from_canonical_i64(diff));
    out_buffer.set_target(remainder_target, F::from_canonical_i64(r));
    out_buffer.set_target(output_target, div)
  }

  fn serialize(&self, dst: &mut Vec<u8>, _common_data: &CommonCircuitData<F, D>) -> IoResult<()> {
    dst.write_usize(self.row)?;
    dst.write_field(self.b)?;
    dst.write_field(self.shift_min_val)?;
    dst.write_field(self.div_outp_min_val)?;
    dst.write_usize(self.i)
  }

  fn deserialize(src: &mut Buffer, _common_data: &CommonCircuitData<F, D>) -> IoResult<Self> {
    let row = src.read_usize()?;
    let b = src.read_field()?;
    let shift_min_val = src.read_field()?;
    let div_outp_min_val = src.read_field()?;
    let i = src.read_usize()?;
    Ok(Self {
      row,
      b,
      shift_min_val,
      div_outp_min_val,
      i,
    })
  }
}
