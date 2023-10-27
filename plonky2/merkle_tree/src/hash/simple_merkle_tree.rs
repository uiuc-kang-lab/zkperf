// Simple Merkle Tree impl

use itertools::Itertools;
use num::Integer;
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::hash::hash_types::BytesHash;
use plonky2::hash::keccak::KeccakHash;
use plonky2::plonk::config::Hasher;
use plonky2_util::log2_strict;

#[derive(Debug, Clone)]
pub struct MerkleTree {
    pub count_levels: usize,
    pub tree: Vec<Vec<BytesHash<32>>>, // contains vectors of hashes for the levels in the tree (count_levels-1 vectors)
    pub root: BytesHash<32>,
}

impl MerkleTree {
    // From list of hashes with length len, take each pair and hash them, resulting in a new vector of hashes of length len/2
    fn next_level_hashes(current_level: Vec<BytesHash<32>>) -> Vec<BytesHash<32>> {
        let temp: Vec<&[BytesHash<32>]> = current_level.chunks(2).into_iter().collect_vec();
        let next_level: Vec<BytesHash<32>> = temp
            .into_iter()
            .map(|x| <KeccakHash<32> as Hasher<GoldilocksField>>::two_to_one(x[0], x[1]))
            .collect();
        next_level
    }

    // Create a Merkle Tree given 2^n leaves.
    pub fn build(leaves: Vec<GoldilocksField>) -> Self {
        // This panics if length is not a power of 2
        let count_levels = log2_strict(leaves.len());

        // To get the first level, hash all leaves
        let level0: Vec<BytesHash<32>> = leaves
            .into_iter()
            .map(|leaf| <KeccakHash<32> as Hasher<GoldilocksField>>::hash_or_noop(&[leaf]))
            .collect();

        let mut levels = Vec::new();
        levels.push(level0);
        // For next levels, hash every hashes. Ends at 2 hashes.
        for i in 0..(count_levels - 1) {
            let next_level = Self::next_level_hashes(levels[i].clone());
            levels.push(next_level);
        }

        // Final hash for root.
        let last_hashes: Vec<BytesHash<32>> = levels.clone().last().unwrap().to_vec();
        let root =
            <KeccakHash<32> as Hasher<GoldilocksField>>::two_to_one(last_hashes[0], last_hashes[1]);
        MerkleTree {
            count_levels: count_levels,
            tree: levels.clone(),
            root: root,
        }
    }

    // Returns count_levels elements that together with the leaf show that a leaf is part of this Merkle Tree, given the root
    // starts at the element at the lowest level and goes up
    pub fn get_merkle_proof(self, leaf_index: usize) -> Vec<BytesHash<32>> {
        assert!(leaf_index < self.tree[0].len());

        let mut proof_hashes = Vec::new();
        let mut updated_index = leaf_index;

        // Grab the correct hash per level
        for i in 0..(self.count_levels) {
            let level_i: &Vec<BytesHash<32>> = &self.tree[i];
            let selected_hash = if updated_index.is_odd() {
                level_i[updated_index - 1]
            } else {
                level_i[updated_index + 1]
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
