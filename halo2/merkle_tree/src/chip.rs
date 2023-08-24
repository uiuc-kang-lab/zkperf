use halo2_gadgets::poseidon::primitives::P128Pow5T3;
use halo2_proofs::{
    arithmetic::Field,
    circuit::*,
    dev::cost::*, // Trying for the first time
    plonk::*,
    poly::Rotation,
    transcript::{Blake2bRead, Blake2bWrite, TranscriptReadBuffer, TranscriptWriterBuffer, Challenge255}, halo2curves::ff::PrimeField
};

use halo2_proofs::halo2curves::pasta::Fp;

use crate::hash::poseidon::{PoseidonConfig, PoseidonChip};


#[derive(Debug, Clone)]
pub struct MerkleTreeConfig{
    pub public_input: Column<Instance>,
    pub left: Column<Advice>,
    pub right: Column<Advice>,
    pub orientation: Column<Advice>,
    pub s_bool: Selector,
    pub s_swap: Selector,
    pub hash_config: PoseidonConfig<3,2,2>,
}

#[derive(Debug, Clone)]
pub struct MerkleTreeChip{
    config: MerkleTreeConfig,
}
 
impl MerkleTreeChip {    
    
    pub fn construct(config: MerkleTreeConfig) -> MerkleTreeChip{
        MerkleTreeChip{
            config,
        }
    }

    pub fn configure(meta: &mut ConstraintSystem<Fp>) -> MerkleTreeConfig
    {
        let public_input = meta.instance_column();
        let left = meta.advice_column();
        let right = meta.advice_column();
        let orientation = meta.advice_column();

        let s_bool = meta.selector();
        let s_swap = meta.selector();

        // Reference: https://github.com/summa-dev/summa-solvency/blob/master/zk_prover/src/chips/merkle_sum_tree.rs
        // Reference: https://github.com/DrPeterVanNostrand/halo2-merkle/blob/0ff8eb439b97318cf65141ee31f17b1f75114bed/src/main.rs#L136

        // Gate1: Check if orientation is boolean
        meta.create_gate("Boolean Orientation", |meta|
        {
            let s = meta.query_selector(s_bool);
            let orientation = meta.query_advice(orientation, Rotation::cur());
            vec![s*orientation.clone()*(Expression::Constant(Fp::from(1)) - orientation)]
        });

        // Gate2: Check Swaping
        meta.create_gate("Swap Check", |meta|
        {
            let s = meta.query_selector(s_swap);
            let l = meta.query_advice(left, Rotation::cur());
            let l_next = meta.query_advice(left, Rotation::next());
            let r = meta.query_advice(right, Rotation::cur());
            let r_next = meta.query_advice(right, Rotation::next());
            let orientation = meta.query_advice(orientation, Rotation::cur());

            let check = 
            s* ((orientation* Expression::Constant(Fp::from(2))* (r.clone() - l.clone()) - (l_next - l)) - (r - r_next));
            vec![check]
        });
        MerkleTreeConfig{
            public_input,
            left,
            right,
            orientation,
            s_bool,
            s_swap,
            hash_config: PoseidonChip::<P128Pow5T3,3,2,2>::configure(meta),  
        }
    }

    pub fn load_first_row(){

    }

    pub fn load_the_rest(){

    }
    pub fn expose_public(){

    }

}

