// Simple Merkle Tree impl

use itertools::Itertools;
use num::Integer;
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2_field::types::PrimeField64;
use plonky2_util::log2_strict;
use sha3::Digest;
use sha3::Keccak256;

#[derive(Debug, Clone)]
pub struct MerkleTree {
    pub count_levels: usize,
    pub tree: Vec<Vec<Vec<u8>>>, // contains vectors of hashes for the levels in the tree (count_levels-1 vectors)
    pub root: Vec<u8>,
}

impl MerkleTree {
    fn keccak256(inp: &[u8]) -> Vec<u8> {
        let mut hasher = Keccak256::new();
        hasher.update(inp);
        let result = hasher.finalize();
        hex::decode(hex::encode(result)).unwrap()
    }

    fn next_level_hashes(current_level: Vec<Vec<u8>>) -> Vec<Vec<u8>> {
        let temp: Vec<&[Vec<u8>]> = current_level.chunks(2).into_iter().collect_vec();
        let next_level = temp
            .into_iter()
            .map(|x| {
                let inp = [x[0].clone(), x[1].clone()].concat();
                Self::keccak256(inp.as_slice())
            })
            .collect();
        next_level
    }

    pub fn build(leaves: Vec<GoldilocksField>) -> Self {
        // This panics if length is not a power of 2
        let count_levels = log2_strict(leaves.len());

        // To get the first level, hash all leaves
        let level0: Vec<Vec<u8>> = leaves
            .into_iter()
            .map(|leaf| {
                let inp = leaf.to_canonical_u64().to_le_bytes();
                Self::keccak256(inp.as_slice())
            })
            .collect();

        let mut levels = Vec::new();
        levels.push(level0);
        // For next levels, hash every hashes. Ends at 2 hashes.
        for i in 0..(count_levels - 1) {
            let next_level = Self::next_level_hashes(levels[i].clone());
            levels.push(next_level);
        }

        // Final hash for root.
        let last_hashes = levels.clone().last().unwrap().to_vec();
        let root = {
            let inp = [last_hashes[0].clone(), last_hashes[1].clone()].concat();
            Self::keccak256(inp.as_slice())
        };
        MerkleTree {
            count_levels,
            tree: levels.clone(),
            root,
        }
    }

    // Returns count_levels elements that together with the leaf show that a leaf is part of this Merkle Tree, given the root
    // starts at the element at the lowest level and goes up
    pub fn get_merkle_proof(self, leaf_index: usize) -> Vec<Vec<u8>> {
        assert!(leaf_index < self.tree[0].len());

        let mut proof_hashes = Vec::new();
        let mut updated_index = leaf_index;

        // Grab the correct hash per level
        for i in 0..(self.count_levels) {
            let level_i = &self.tree[i];
            let selected_hash = if updated_index.is_odd() {
                level_i[updated_index - 1].clone()
            } else {
                level_i[updated_index + 1].clone()
            };
            proof_hashes.push(selected_hash);
            updated_index = updated_index / 2;
        }

        proof_hashes
    }

    // pub fn get_in_between_hashes(self, leaf_index: usize) -> Vec<HashOut<GoldilocksField>>{
    //   assert!(leaf_index < self.tree[0].len());
    //   let mut index = leaf_index / 2;
    //   let mut hashes = Vec::new();
    //   for i in 1..self.count_levels {
    //     hashes.push(self.tree[i][index]);
    //     index = index / 2;
    //   }
    //   hashes.push(self.root);
    //   hashes
    // }
}
