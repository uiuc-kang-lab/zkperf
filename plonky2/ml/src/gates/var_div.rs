use num_traits::ToPrimitive;
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

#[derive(Debug, Clone)]
pub struct DivRoundGate {
  pub num_ops: usize,
}

impl DivRoundGate {
  pub fn new(_num_ops: usize) -> Self {
    Self { num_ops: 1 }
  }

  pub fn new_from_config(config: &CircuitConfig) -> Self {
    Self {
      num_ops: Self::num_ops(config),
    }
  }

  pub(crate) fn num_ops(_config: &CircuitConfig) -> usize {
    1
  }

  pub fn wire_input() -> usize {
    0
  }

  pub fn wire_output() -> usize {
    1
  }

  pub fn wire_remainder() -> usize {
    2
  }

  pub fn wire_div_rem_diff() -> usize {
    3
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
    let mut constraints = vec![];
    let a = vars.local_wires[Self::wire_input()];
    let c = vars.local_wires[Self::wire_output()];
    let diff = vars.local_wires[Self::wire_div_rem_diff()];
    let r = vars.local_wires[Self::wire_remainder()];

    // (2 * a + b) = (2 * b) * c + r
    let two = F::Extension::from_canonical_u64(2);
    let lhs = a * two + b;
    let rhs = b * two * c + r;
    constraints.push(lhs - rhs);

    // 2 * b - r >= 1 => exists diff >= 0 s.t. 2 * b - r - 1 = diff
    constraints.push(two * b - r - F::Extension::ONE - diff);

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
    let mut constraints = Vec::with_capacity(2 * self.num_ops);
    let b = vars.local_constants[0];
    let a = vars.local_wires[Self::wire_input()];
    let c = vars.local_wires[Self::wire_output()];
    let diff = vars.local_wires[Self::wire_div_rem_diff()];
    let r = vars.local_wires[Self::wire_remainder()];

    let one = builder.constant_extension(F::Extension::ONE);
    let two = builder.constant_extension(F::Extension::from_canonical_u64(2));
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

    constraints
  }

  fn generators(&self, row: usize, local_constants: &[F]) -> Vec<WitnessGeneratorRef<F, D>> {
    vec![WitnessGeneratorRef::new(
      DivRoundGenerator {
        row,
        b: local_constants[0],
        shift_min_val: local_constants[1],
        div_outp_min_val: local_constants[2],
      }
      .adapter(),
    )]
  }

  fn num_wires(&self) -> usize {
    4
  }

  fn num_constants(&self) -> usize {
    3
  }

  fn degree(&self) -> usize {
    2
  }

  fn num_constraints(&self) -> usize {
    2
  }
}

impl<F: RichField + Extendable<D>, const D: usize> PackedEvaluableBase<F, D> for DivRoundGate {
  fn eval_unfiltered_base_packed<P: PackedField<Scalar = F>>(
    &self,
    vars: EvaluationVarsBasePacked<P>,
    mut yield_constr: StridedConstraintConsumer<P>,
  ) {
    let b = vars.local_constants[0];
    let a = vars.local_wires[Self::wire_input()];
    let c = vars.local_wires[Self::wire_output()];
    let diff = vars.local_wires[Self::wire_div_rem_diff()];
    let r = vars.local_wires[Self::wire_remainder()];

    let two = P::ONES + P::ONES;
    let lhs = a * two + b;
    let rhs = b * two * c + r;
    yield_constr.one(lhs - rhs);
    yield_constr.one(two * b - r - P::ONES - diff);
  }
}

#[derive(Clone, Debug, Default)]
pub struct DivRoundGenerator<F: RichField + Extendable<D>, const D: usize> {
  row: usize,
  b: F,
  shift_min_val: F,
  div_outp_min_val: F,
}

impl<F: RichField + Extendable<D>, const D: usize> SimpleGenerator<F, D>
  for DivRoundGenerator<F, D>
{
  fn id(&self) -> String {
    "DivRoundGenerator".to_string()
  }

  fn dependencies(&self) -> Vec<Target> {
    vec![Target::wire(self.row, DivRoundGate::wire_input())]
  }

  fn run_once(&self, witness: &PartitionWitness<F>, out_buffer: &mut GeneratedValues<F>) {
    let get_wire = |wire: usize| -> F { witness.get_target(Target::wire(self.row, wire)) };
    let a = get_wire(DivRoundGate::wire_input());
    let b_int = self.b.to_canonical_biguint().to_u128().unwrap();
    // println!("a: {}, b_int: {}", a, b_int);

    let div_outp_min_val_i64 = self.div_outp_min_val.to_canonical_u64() as i64;
    let div_inp_min_val_pos_i64 = -(self.shift_min_val.to_canonical_u64() as i64);
    let div_inp_min_val_pos_i64 = div_inp_min_val_pos_i64 / (b_int as i64) * (b_int as i64);
    let div_inp_min_val_pos = F::from_canonical_u64(div_inp_min_val_pos_i64 as u64);

    let a_pos = a + div_inp_min_val_pos;
    let a = a_pos.to_canonical_biguint().to_u128().unwrap();
    // c = (2 * a + b) / (2 * b)
    let c_pos = a.rounded_div(b_int);
    let c = (c_pos as i128 - (div_inp_min_val_pos_i64 as u128 / b_int) as i128) as i64;

    let rem_floor = (a as i128) - (c_pos * b_int) as i128;
    let r = 2 * rem_floor + (b_int as i128);
    let r = r as i64;
    let diff = 2 * b_int as i64 - r - 1;
    // println!(
    //   "domvi: {}, smv: {}, dimvp: {}, a: {}, c_pos: {}, c: {}, rem_floor: {}, r: {}, diff: {}",
    //   div_outp_min_val_i64,
    //   self.shift_min_val,
    //   div_inp_min_val_pos_i64,
    //   a,
    //   c_pos,
    //   c,
    //   rem_floor,
    //   r,
    //   diff
    // );

    let output_target = Target::wire(self.row, DivRoundGate::wire_output());
    let div_rem_diff_target = Target::wire(self.row, DivRoundGate::wire_div_rem_diff());
    let remainder_target = Target::wire(self.row, DivRoundGate::wire_remainder());

    let div = {
      let offset = F::from_canonical_u64(-div_outp_min_val_i64 as u64);
      let c = F::from_canonical_u64((c - div_outp_min_val_i64) as u64);
      c - offset
    };

    // println!("div: {}", div);
    // println!("lhs: {}, rhs: {}", get_wire(DivRoundGate::wire_input()) * F::from_canonical_i64(2) + self.b, self.b * F::from_canonical_i64(2) * div + F::from_canonical_i64(r));
    assert!(
      get_wire(DivRoundGate::wire_input()) * F::from_canonical_i64(2) + self.b
        == self.b * F::from_canonical_i64(2) * div + F::from_canonical_i64(r)
    );

    assert!(
      F::from_canonical_i64(2) * self.b - F::from_canonical_i64(r)
        == F::from_canonical_i64(diff) + F::ONE
    );
    out_buffer.set_target(div_rem_diff_target, F::from_canonical_i64(diff));
    out_buffer.set_target(remainder_target, F::from_canonical_i64(r));
    out_buffer.set_target(output_target, div)
  }

  fn serialize(&self, dst: &mut Vec<u8>, _common_data: &CommonCircuitData<F, D>) -> IoResult<()> {
    dst.write_usize(self.row)?;
    dst.write_field(self.b)?;
    dst.write_field(self.shift_min_val)?;
    dst.write_field(self.div_outp_min_val)
  }

  fn deserialize(src: &mut Buffer, _common_data: &CommonCircuitData<F, D>) -> IoResult<Self> {
    let row = src.read_usize()?;
    let b = src.read_field()?;
    let shift_min_val = src.read_field()?;
    let div_outp_min_val = src.read_field()?;
    Ok(Self {
      row,
      b,
      shift_min_val,
      div_outp_min_val,
    })
  }
}
