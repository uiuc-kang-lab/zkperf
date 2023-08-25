use halo2_gadgets::poseidon::primitives::P128Pow5T3;
use halo2_proofs::{circuit::*, plonk::*,poly::Rotation, halo2curves::pasta::Fp};

use crate::hash::poseidon::{PoseidonConfig, PoseidonChip};

#[derive(Debug, Clone)]
pub struct MerkleTreeConfig{
    pub public_input: Column<Instance>,
    pub left: Column<Advice>,
    pub right: Column<Advice>,
    pub orientation: Column<Advice>,
    pub s_swap: Selector,
    pub hash_config: PoseidonConfig<3,2,2>,
}

#[derive(Debug, Clone)]
pub struct MerkleTreeChip{
    config: MerkleTreeConfig,
}
 
impl MerkleTreeChip{        
    pub fn construct(config: MerkleTreeConfig) -> MerkleTreeChip
    {
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
        meta.enable_equality(public_input);
        meta.enable_equality(left);
        meta.enable_equality(right);
        let s_swap = meta.selector();

        // Reference: https://github.com/summa-dev/summa-solvency/blob/master/zk_prover/src/chips/merkle_sum_tree.rs
        // Reference: https://github.com/DrPeterVanNostrand/halo2-merkle/blob/0ff8eb439b97318cf65141ee31f17b1f75114bed/src/main.rs#L136

        // Gate1: Check if orientation is boolean
        meta.create_gate("Boolean Orientation", |meta|
        {
            let s = meta.query_selector(s_swap);
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
            s_swap,
            hash_config: PoseidonChip::<P128Pow5T3,3,2,2>::configure(meta),  
        }
    }

    pub fn load_leaf(&self,leaf: Fp, mut layouter: impl Layouter<Fp>) -> Result<AssignedCell<Fp, Fp>, Error>
    {
        layouter.assign_region(|| "load leaf",|mut region| 
            {
                region.assign_advice_from_constant(|| "leaf",self.config.left,0,leaf)
            },
        )

    }

    pub fn load(&self,mut layouter: impl Layouter<Fp>, digest: &AssignedCell<Fp, Fp>, element:Fp, orientation: Fp, offset: usize) -> AssignedCell<Fp, Fp>
    {
        let (l,r) = layouter.assign_region(|| "load row", |mut region|{
            digest.copy_advice(|| "left", &mut region, self.config.left, offset)?;
            region.assign_advice(|| "right", self.config.right, offset, || Value::known(element))?;
            region.assign_advice(|| "orientation", self.config.orientation, offset, || Value::known(orientation))?;
            self.config.s_swap.enable(&mut region, offset)?;

            let left: Value<Fp>;
            let right: Value<Fp>;
            if orientation == Fp::from(0){
                 left = digest.value().map(|x| x.to_owned());
                 right = Value::known(element);   
            }
            else{
                 right = digest.value().map(|x| x.to_owned());
                 left = Value::known(element);
            }
            let left_cell = region.assign_advice(||"swapped left", self.config.left, offset+1, || left)?;
            let right_cell = region.assign_advice(||"swapped right", self.config.right, offset+1, || right)?;
            Ok((left_cell,right_cell))
            }).unwrap();
        
        let poseidon_chip = PoseidonChip::<P128Pow5T3, 3, 2, 2>::construct(self.config.hash_config.clone());
        let digest = poseidon_chip.hash(layouter.namespace(|| "poseidon"), &[l, r]).unwrap();
        digest
    }
    
    pub fn expose_public(&self, mut layouter: impl Layouter<Fp>,assg_cell: &AssignedCell<Fp, Fp>, row: usize,) -> Result<(), Error> 
    {
        layouter.constrain_instance(assg_cell.cell(), self.config.public_input, row)
    }

}

