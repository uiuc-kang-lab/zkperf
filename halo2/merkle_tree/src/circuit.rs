use halo2_proofs::{
    circuit::*,
    // dev::cost::*, // TODO: Try this
    halo2curves::pasta::{Fp,EqAffine},
    plonk::*,
    poly::{commitment::ParamsProver,
        ipa::{commitment::{IPACommitmentScheme, ParamsIPA},
            multiopen::{ProverIPA, VerifierIPA},
            strategy::AccumulatorStrategy,},
        VerificationStrategy,
    },
    transcript::{Blake2bRead, Blake2bWrite, Challenge255, TranscriptReadBuffer, TranscriptWriterBuffer},
};
use rand::rngs::OsRng;
use std::time::Instant;

use crate::chip::*;

#[derive(Default, Clone)]  
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

pub fn real_prover(circuit: MerkleTreeCircuit, public_input: &[&[halo2_proofs::halo2curves::pasta::Fp]]){
    let k = 10;
    let params = ParamsIPA::<EqAffine>::new(k);
    
    let vk_time_start = Instant::now();
    let vk = keygen_vk(&params, &circuit).unwrap();
    let vk_time = vk_time_start.elapsed();
    
    let pk_time_start = Instant::now();
    let pk = keygen_pk(&params, vk, &circuit).unwrap();
    let pk_time = pk_time_start.elapsed();

    let proof_time_start = Instant::now();
    let proof = {
        let mut transcript = Blake2bWrite::<_, _, Challenge255<_>>::init(vec![]);
        create_proof::<IPACommitmentScheme<EqAffine>, ProverIPA<EqAffine>, _, _, _, _>(
                        &params,
                        &pk,
                        &[circuit],
                        &[public_input],
                        OsRng,
                        &mut transcript,
        ).expect("Proof Gen Failed");
        transcript.finalize()
    };
    let proof_time = proof_time_start.elapsed();

    let verify_time_start = Instant::now();
    let strategy = AccumulatorStrategy::new(&params);
    let mut transcript = Blake2bRead::<_, _, Challenge255<_>>::init(&proof[..]);

    let res = verify_proof::<IPACommitmentScheme<EqAffine>, VerifierIPA<EqAffine>, _, _, _>(
            &params,
            pk.get_vk(),
            strategy,
            &[public_input],
            &mut transcript,
    ).map(|strategy| strategy.finalize()).unwrap();
    let verify_time = verify_time_start.elapsed();

    if !res{
       panic!("Verification Failed");
    }

    println!("Time to generate vk {:?}", vk_time);
    println!("Time to generate pk {:?}", pk_time);
    println!("Prover Time {:?}", proof_time);
    println!("Verifier Time {:?}", verify_time);
    
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

        let pub_input = vec![input.leaf, input.digest];
        // let pub_input = vec![input.leaf, Fp::zero()];

        let input_real_prover = [&pub_input[..], &pub_input[..]];
        real_prover(circuit.clone(), &input_real_prover);
        
        // completeness
        let pub_input_mock = vec![input.leaf, input.digest];
        let prover = MockProver::run(10, &circuit,
            vec![pub_input_mock.clone(), pub_input_mock]).unwrap();
        prover.assert_satisfied();
    
        // soundness
        let pub_input_mock = vec![input.leaf, Fp::zero()];
        let _prover2 = MockProver::run(10, &circuit,
            vec![pub_input_mock.clone(), pub_input_mock]).unwrap();
        // _prover2.assert_satisfied(); // Test should fail if uncommented
    }

}