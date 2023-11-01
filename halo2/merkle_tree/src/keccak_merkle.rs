use axiom_eth::{keccak::KeccakChip, Field, EthChip};
use halo2_base::{
    gates::{
        flex_gate::{GateChip, GateInstructions},
        RangeInstructions
    }, 
    AssignedValue, 
    Context, 
    // QuantumCell::{Existing, Witness, Constant}
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CircuitInput {
    // pub key: Vec<u8>,
    // pub val: Vec<u8>,
    pub child: Vec<u8>,
    pub siblings: Vec<Vec<u8>>,
    pub root: Vec<u8>,
    pub index: u64,
    pub n_levels: u64,
}


pub fn merkle_tree_keccak<F: Field>(
    ctx: &mut Context<F>,
    eth: &EthChip<F>,
    keccak: &mut KeccakChip<F>,
    input: CircuitInput,
    _make_public: &mut Vec<AssignedValue<F>>,
) -> impl FnOnce(&mut Context<F>, &mut Context<F>, &EthChip<F>) + Clone {

    let bitify_gate = GateChip::<F>::default();
    let selector_gate = GateChip::<F>::default();
    let range = eth.range();

    let index = ctx.load_witness(F::from(input.index));
    
    let sels = bitify_gate.num_to_bits(ctx, index, (input.n_levels-1) as usize);
    
    // let mut init_input = input.key.clone();
    // init_input.extend(input.val.iter().cloned());
    // let bytes_assigned = ctx.assign_witnesses(init_input.iter().map(|byte| F::from(*byte as u64)));
    // let input_hash_idx = keccak.keccak_fixed_len(ctx, range.gate(), bytes_assigned, Some(init_input));
    // let mut curr = keccak.fixed_len_queries[input_hash_idx].output_assigned.clone();

    let mut curr = ctx.assign_witnesses(input.child.iter().map(|byte| F::from(*byte as u64)));

    for i in 0..(input.n_levels-1) as usize {
        let sibling = ctx.assign_witnesses(input.siblings[i].iter().map(|x| F::from((*x) as u64)));
        let mut l_list = vec![];
        let mut r_list = vec![];
        for j in 0..32 {

            let l_o = selector_gate.select(ctx, sibling[j], curr[j], sels[i]);
            let r_o = selector_gate.select(ctx, curr[j], sibling[j], sels[i]);
            l_list.push(l_o);
            r_list.push(r_o);
        }
        l_list.extend(r_list.iter());
        let parent_hash_idx = keccak.keccak_fixed_len(ctx, range.gate(), l_list, None);
        curr = keccak.fixed_len_queries[parent_hash_idx].output_assigned.clone();
    }

    let root = ctx.assign_witnesses(input.root.iter().map(|x| F::from((*x) as u64)));
    for i in 0..32 {
        ctx.constrain_equal(&root[i], &curr[i]);
    }

    #[allow(clippy::let_and_return)]
    let callback =
        |_ctx_gate: &mut Context<F>, _ctx_rlc: &mut Context<F>, _eth_chip: &EthChip<F>| {};

    callback

}