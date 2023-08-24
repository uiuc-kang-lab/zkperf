use std::vec;
use rand::Rng;
use halo2_proofs::halo2curves::ff::PrimeField;

pub fn to_field<F: PrimeField> (leaves: Vec<i32>) -> Vec<F>
{
    let mut leaves_field = vec![];
    for i in 0..leaves.len(){
        if leaves[i] >= 0  {
            leaves_field.push(F::from(leaves[i] as u64));
        }
        else {
            leaves_field.push(-F::from(leaves[i] as u64));
        }
    }
    return leaves_field;
}


// Just for the purpose of development of codebase
pub fn create_random_data<F: PrimeField>(mut n: usize) -> Vec<F> 
{    
    let mut rng = rand::thread_rng();
    if n.is_power_of_two(){
        n = n.next_power_of_two();
    }

    let mut leaves: Vec<i32> = vec![];
    for _i in 0..n{
        let buf:i32 = rng.gen();
        leaves.push(buf);
    } 
    return to_field(leaves);
}




// For a case where data is being imported from outside
// Need to figure this out
// pub fn load_data(){ 
    
// }

#[cfg(test)]
mod tests {
    use super::*;
    use halo2_proofs::halo2curves::pasta::Fp;


    #[test]
    fn test_field_converter() {
        let example_leaves = [13,2,3,4,6].to_vec();
        let mut example_leaves_filed = vec![];
        for i in 0..example_leaves.len(){
            example_leaves_filed.push(Fp::from(example_leaves[i] as u64));
        }
        assert_eq!(example_leaves_filed, to_field(example_leaves));

    }
}