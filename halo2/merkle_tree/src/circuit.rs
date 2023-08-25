use halo2_proofs::{
    circuit::*,
    // dev::cost::*, // TODO: Try this
    halo2curves::pasta::Fp,
    plonk::*,
};

use crate::chip::*;

#[derive(Default)]  
pub struct MerkleTreeCircuit{
    leaf: Fp,
    elements: Vec<Fp>,
    orientation: Vec<Fp>
}
impl Circuit<Fp> for MerkleTreeCircuit{
    type Config = MerkleTreeConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default() 
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
        MerkleTreeChip::configure(meta)
    }

    fn synthesize(&self, config: Self::Config, mut layouter: impl Layouter<Fp>) -> Result<(), Error> {
        let merklechip = MerkleTreeChip::construct(config);
        let leaf_cell = merklechip.load_leaf(self.leaf, layouter.namespace(|| "leaf"))?;
        merklechip.expose_public(layouter.namespace(|| "leaf"), &leaf_cell.clone(), 0)?;
        let mut offset: usize = 0;
        let mut digest = merklechip.load(layouter.namespace(|| "row".to_owned()+&0.to_string()), &leaf_cell.clone(), self.elements[0], self.orientation[0], offset);
        for i in 1..self.elements.len(){
            offset = offset + 2;
            digest = merklechip.load(layouter.namespace(|| "row".to_owned()+&i.to_string()), &digest, self.elements[i], self.orientation[i], offset);
        }
        merklechip.expose_public(layouter.namespace(|| "digest"), &digest, 1)
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::to_field;
    use halo2_gadgets::poseidon::primitives::{self as poseidon_hash, P128Pow5T3, ConstantLength};
    use halo2_proofs::halo2curves::pasta::Fp;
    use halo2_proofs::dev::MockProver;

    use super::*;

    struct Input{
        leaf: Fp,
        elements: Vec<Fp>,
        orientation: Vec<Fp>,
        digest: Fp,
    }
    impl Input{
        fn prepare_input(leaf:i32, elements:[i32;3], orientation:[i32;3])-> Input{
        
            let field_leaf = to_field(leaf);
            let mut field_elements = vec![];
            let mut field_orientation = vec![];
            for i in 0..elements.len(){
                field_elements.push(to_field(elements[i]));
                field_orientation.push(to_field(orientation[i]));
            }
            let mut digest = field_leaf;
            let mut msg: [Fp;2];
            for i in 0..elements.len(){
                if orientation[i] == 0 {
                    msg = [digest, field_elements[i]];
                } else {
                    msg = [field_elements[i], digest];
                }
                digest = poseidon_hash::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init().hash(msg);
            }
            return Input{leaf: field_leaf, elements: field_elements, orientation: field_orientation, digest};
        }
    }
    

    #[test]
    fn mock_run() 
    {
        let leaf = 1;
        let elements = [2,34,5678];
        let orientation = [0,0,0];
        let input = Input::prepare_input(leaf, elements, orientation);
        let circuit = MerkleTreeCircuit {
            leaf: input.leaf,
            elements: input.elements,
            orientation: input.orientation,
        };
        // completeness
        let pub_input = vec![input.leaf, input.digest];
        let prover = MockProver::run(10, &circuit,
            vec![pub_input.clone(), pub_input]).unwrap();
        prover.assert_satisfied();
    
        // soundness
        let pub_input = vec![input.leaf, Fp::zero()];
        let _prover2 = MockProver::run(10, &circuit,
            vec![pub_input.clone(), pub_input]).unwrap();
        // _prover2.assert_satisfied(); // Test should fail if uncommented
    }

}