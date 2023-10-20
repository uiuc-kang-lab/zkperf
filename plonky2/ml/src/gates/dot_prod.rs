use plonky2::{
  field::extension::Extendable,
  field::packed::PackedField,
  gates::gate::Gate,
  gates::packed_util::PackedEvaluableBase,
  gates::util::StridedConstraintConsumer,
  hash::hash_types::RichField,
  iop::ext_target::ExtensionTarget,
  iop::generator::{GeneratedValues, SimpleGenerator, WitnessGeneratorRef},
  iop::target::Target,
  iop::witness::{PartitionWitness, Witness, WitnessWrite},
  plonk::circuit_builder::CircuitBuilder,
  plonk::circuit_data::CommonCircuitData,
  plonk::vars::{
    EvaluationTargets, EvaluationVars, EvaluationVarsBase, EvaluationVarsBaseBatch,
    EvaluationVarsBasePacked,
  },
  util::serialization::{Buffer, IoResult, Read, Write},
};

#[derive(Debug, Clone)]
pub struct DotProductGate {}

pub const DOTPROD_SIZE: usize = 9;

impl DotProductGate {
  pub fn construct() -> Self {
    Self {}
  }

  // pub fn wire_input_start() -> usize {
  //   0
  // }

  // pub fn wire_weight_end() -> usize {
  //   2 * DOTPROD_SIZE * DOTPROD_SIZE - 1
  // }

  // pub fn wire_ith_input(i: usize, j: usize) -> usize {
  //   i * DOTPROD_SIZE + j
  // }

  // pub fn wire_ith_weight(i: usize, j: usize) -> usize {
  //   (DOTPROD_SIZE + i) * DOTPROD_SIZE + j
  // }

  // pub fn wire_ith_input(i: usize) -> usize {
  //   i
  // }

  // pub fn wire_ith_weight(i: usize) -> usize {
  //   DOTPROD_SIZE * DOTPROD_SIZE + i
  // }

  // pub fn wire_output() -> usize {
  //   2 * DOTPROD_SIZE * DOTPROD_SIZE
  // }

  pub fn wire_input_start() -> usize {
    0
  }

  pub fn wire_weight_end() -> usize {
    2 * DOTPROD_SIZE - 1
  }

  pub fn wire_ith_input(i: usize) -> usize {
    i
  }

  pub fn wire_ith_weight(i: usize) -> usize {
    DOTPROD_SIZE + i
  }

  pub fn wire_output() -> usize {
    2 * DOTPROD_SIZE
  }
}

impl<F: RichField + Extendable<D>, const D: usize> Gate<F, D> for DotProductGate {
  fn id(&self) -> String {
    "dot product".to_string()
  }

  fn serialize(&self, dst: &mut Vec<u8>, _common_data: &CommonCircuitData<F, D>) -> IoResult<()> {
    dst.write_usize(0)
  }

  fn deserialize(src: &mut Buffer, _common_data: &CommonCircuitData<F, D>) -> IoResult<Self> {
    let _n = src.read_usize()?;
    Ok(Self {})
  }

  fn eval_unfiltered(&self, vars: EvaluationVars<F, D>) -> Vec<F::Extension> {
    let zero = vars.local_constants[0];
    let mut computed_output: <F as Extendable<D>>::Extension = F::Extension::ZEROS;
    for i in 0..DOTPROD_SIZE {
      let input = vars.local_wires[Self::wire_ith_input(i)];
      let weight = vars.local_wires[Self::wire_ith_weight(i)];
      computed_output += (input - zero) * weight;
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
    let zero = vars.local_constants[0];

    let pairs = (0..DOTPROD_SIZE)
      .map(|i| {
        let input = vars.local_wires[Self::wire_ith_input(i)];
        let weight = vars.local_wires[Self::wire_ith_weight(i)];
        let input_zero = builder.sub_extension(input, zero);
        builder.mul_extension(input_zero, weight)
      })
      .collect::<Vec<_>>();
    let computed_output = builder.add_many_extension(pairs);

    let output = vars.local_wires[Self::wire_output()];
    let diff = builder.sub_extension(output, computed_output);

    vec![diff]
  }

  fn generators(&self, row: usize, local_constants: &[F]) -> Vec<WitnessGeneratorRef<F, D>> {
    vec![WitnessGeneratorRef::new(
      DotProductGenerator {
        row,
        zero: local_constants[0],
      }
      .adapter(),
    )]
  }

  fn num_wires(&self) -> usize {
    2 * DOTPROD_SIZE + 1
  }

  fn num_constants(&self) -> usize {
    1
  }

  fn degree(&self) -> usize {
    2
  }

  fn num_constraints(&self) -> usize {
    1
  }
}

impl<F: RichField + Extendable<D>, const D: usize> PackedEvaluableBase<F, D> for DotProductGate {
  fn eval_unfiltered_base_packed<P: PackedField<Scalar = F>>(
    &self,
    vars: EvaluationVarsBasePacked<P>,
    mut yield_constr: StridedConstraintConsumer<P>,
  ) {
    let mut computed_output = P::ZEROS;
    let zero = vars.local_constants[0];
    for i in 0..DOTPROD_SIZE {
      let input = vars.local_wires[Self::wire_ith_input(i)];
      let weight = vars.local_wires[Self::wire_ith_weight(i)];
      computed_output += (input - zero) * weight;
    }
    let output = vars.local_wires[Self::wire_output()];
    yield_constr.one(output - computed_output);
  }
}

#[derive(Clone, Debug, Default)]
pub struct DotProductGenerator<F: RichField + Extendable<D>, const D: usize> {
  row: usize,
  zero: F,
}

impl<F: RichField + Extendable<D>, const D: usize> SimpleGenerator<F, D>
  for DotProductGenerator<F, D>
{
  fn id(&self) -> String {
    "DotProductGenerator".to_string()
  }

  fn dependencies(&self) -> Vec<Target> {
    (DotProductGate::wire_input_start()..DotProductGate::wire_weight_end())
      .map(|i| Target::wire(self.row, i))
      .collect()
  }

  fn run_once(&self, witness: &PartitionWitness<F>, out_buffer: &mut GeneratedValues<F>) {
    let get_wire = |wire: usize| -> F { witness.get_target(Target::wire(self.row, wire)) };

    let mut computed_output = F::ZERO;
    for i in 0..DOTPROD_SIZE {
      let input = get_wire(DotProductGate::wire_ith_input(i));
      let weight = get_wire(DotProductGate::wire_ith_weight(i));
      computed_output += (input - self.zero) * weight;
    }

    let output_target = Target::wire(self.row, DotProductGate::wire_output());

    out_buffer.set_target(output_target, computed_output)
  }

  fn serialize(&self, dst: &mut Vec<u8>, _common_data: &CommonCircuitData<F, D>) -> IoResult<()> {
    dst.write_usize(self.row)?;
    dst.write_field(self.zero)
  }

  fn deserialize(src: &mut Buffer, _common_data: &CommonCircuitData<F, D>) -> IoResult<Self> {
    let row = src.read_usize()?;
    let zero = src.read_field()?;
    Ok(Self { row, zero })
  }
}
