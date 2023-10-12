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
use plonky2::plonk::circuit_data::{CommonCircuitData, CircuitConfig};
use plonky2::plonk::vars::{
  EvaluationTargets, EvaluationVars, EvaluationVarsBase, EvaluationVarsBaseBatch,
  EvaluationVarsBasePacked,
};
use plonky2::util::serialization::{Buffer, IoResult, Read, Write};
use plonky2_field::types::Field;

#[derive(Debug, Clone)]
pub struct AdderGate {
  pub num_ops: usize,
}

pub const ADDER_SIZE: usize = 4;

impl AdderGate {
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

  pub fn wire_ith(i: usize) -> usize {
    i
  }

  pub fn wire_output() -> usize {
    ADDER_SIZE
  }
}

impl<F: RichField + Extendable<D>, const D: usize> Gate<F, D> for AdderGate {
  fn id(&self) -> String {
    "adder".to_string()
  }

  fn serialize(&self, dst: &mut Vec<u8>, _common_data: &CommonCircuitData<F, D>) -> IoResult<()> {
    dst.write_usize(self.num_ops)
  }

  fn deserialize(src: &mut Buffer, _common_data: &CommonCircuitData<F, D>) -> IoResult<Self> {
    let num_ops = src.read_usize()?;
    Ok(Self { num_ops })
  }

  fn eval_unfiltered(&self, vars: EvaluationVars<F, D>) -> Vec<F::Extension> {
    let mut computed_output = F::Extension::ZERO;
    for i in 0..ADDER_SIZE {
      computed_output += vars.local_wires[Self::wire_ith(i)];
    }
    let output = vars.local_wires[Self::wire_output()];

    vec![output - computed_output]
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
    let computed_output =
      builder.add_many_extension(&vars.local_wires[Self::wire_ith(0)..=Self::wire_ith(ADDER_SIZE)]);
    let output = vars.local_wires[Self::wire_output()];
    let diff = builder.sub_extension(output, computed_output);

    vec![diff]
  }

  fn generators(&self, row: usize, _local_constants: &[F]) -> Vec<WitnessGeneratorRef<F, D>> {
    vec![WitnessGeneratorRef::new(AdderGenerator { row }.adapter())]
  }

  fn num_wires(&self) -> usize {
    ADDER_SIZE + 1
  }

  fn num_constants(&self) -> usize {
    0
  }

  fn degree(&self) -> usize {
    1
  }

  fn num_constraints(&self) -> usize {
    1
  }
}

impl<F: RichField + Extendable<D>, const D: usize> PackedEvaluableBase<F, D> for AdderGate {
  fn eval_unfiltered_base_packed<P: PackedField<Scalar = F>>(
    &self,
    vars: EvaluationVarsBasePacked<P>,
    mut yield_constr: StridedConstraintConsumer<P>,
  ) {
    let mut computed_output = P::ZEROS;
    for r in 0..ADDER_SIZE {
      computed_output += vars.local_wires[Self::wire_ith(r)];
    }
    let output = vars.local_wires[Self::wire_output()];
    yield_constr.one(output - computed_output);
  }
}

#[derive(Clone, Debug, Default)]
pub struct AdderGenerator {
  row: usize,
}

impl<F: RichField + Extendable<D>, const D: usize> SimpleGenerator<F, D> for AdderGenerator {
  fn id(&self) -> String {
    "AdderGenerator".to_string()
  }

  fn dependencies(&self) -> Vec<Target> {
    (AdderGate::wire_ith(0)..AdderGate::wire_ith(ADDER_SIZE - 1))
      .map(|j| Target::wire(self.row, j))
      .collect()
  }

  fn run_once(&self, witness: &PartitionWitness<F>, out_buffer: &mut GeneratedValues<F>) {
    let get_wire = |wire: usize| -> F { witness.get_target(Target::wire(self.row, wire)) };

    let mut computed_output = F::ZERO;
    for i in 0..ADDER_SIZE {
      computed_output += get_wire(AdderGate::wire_ith(i));
    }

    let output_target = Target::wire(self.row, AdderGate::wire_output());

    out_buffer.set_target(output_target, computed_output)
  }

  fn serialize(&self, dst: &mut Vec<u8>, _common_data: &CommonCircuitData<F, D>) -> IoResult<()> {
    dst.write_usize(self.row)
  }

  fn deserialize(src: &mut Buffer, _common_data: &CommonCircuitData<F, D>) -> IoResult<Self> {
    let row = src.read_usize()?;
    Ok(Self { row })
  }
}
