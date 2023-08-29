use plonky2::field::goldilocks_field::GoldilocksField as F;
use plonky2::plonk::{
    circuit_builder::CircuitBuilder, 
    circuit_data::CircuitConfig, 
    config::PoseidonGoldilocksConfig};
use plonky2::iop::witness::{PartialWitness, WitnessWrite};
use plonky2::hash::{hash_types::HashOut, poseidon::PoseidonHash};
use std::time::Instant;

pub struct MerkleTree{
    root: HashOut<F>,
    nodes: Vec<HashOut<F>>,
}

impl MerkleTree{
    pub fn new(root: HashOut<F>, nodes: Vec<HashOut<F>>)->Self{
        Self{
            root,
            nodes
        }
    }
}

pub fn prove_membership(leaf: Vec<F>, merkle_tree: MerkleTree){
        
    let config = CircuitConfig::standard_recursion_zk_config(); 
    let mut builder = CircuitBuilder::<F,2>::new(config);
   
    // Build circuit
    let target_leaf = builder.add_virtual_public_input_arr();
    let target_root = builder.add_virtual_hash();
    let mut target_nodes = vec![];
    for _i in 0..merkle_tree.nodes.len(){
        target_nodes.push(builder.add_virtual_hash());
    } 

    let mut digest = builder.hash_or_noop::<PoseidonHash>([target_nodes[0].elements,target_leaf].concat()); // Padding zeroes to match datatypes 
    for i in 1..merkle_tree.nodes.len(){
        digest = builder.hash_or_noop::<PoseidonHash>([digest.elements, target_nodes[i].elements].concat());
    }

    // Constraint
    for i in 0..4{
        builder.connect(digest.elements[i], target_root.elements[i]);
    }
    
    // Assign Values
    let cktdata = builder.build::<PoseidonGoldilocksConfig>();
    let mut pw = PartialWitness::<F>::new();

    pw.set_target_arr(&target_leaf, &leaf[..]);
    pw.set_hash_target(target_root, merkle_tree.root);
    for i in 0..merkle_tree.nodes.len(){
        pw.set_hash_target(target_nodes[i], merkle_tree.nodes[i]);
    }

    let start = Instant::now();
    let phi = cktdata.prove(pw).unwrap();
    let ver = cktdata.verify(phi).is_err();
    assert_eq!(ver, false);
    println!("time elapsed: {:?}", start.elapsed())

}

#[cfg(test)]
mod tests{
    use plonky2::plonk::config::Hasher;
    use rand::Rng;
    use super::*;

    #[test]
    fn test_merkle(){
        let mut rng = rand::thread_rng();
        let leaf = [F(12), F(0), F(0), F(0)];
        let mut nodes = vec![];
        let mut buf = PoseidonHash::hash_or_noop(&[F(rng.gen())]);
        nodes.push(buf);
        let mut digest = PoseidonHash::hash_or_noop(&[leaf, buf.elements].concat());
        for _i in 1..3{
            buf = PoseidonHash::hash_or_noop(&[F(rng.gen())]);
            nodes.push(buf);
            digest = PoseidonHash::hash_or_noop(&[digest.elements, buf.elements].concat());
        }

        let merkle_tree = MerkleTree::new(digest, nodes);
        prove_membership(leaf.to_vec(), merkle_tree);
    }


}