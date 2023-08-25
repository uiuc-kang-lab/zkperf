use std::vec;
use rand::Rng;
use halo2_proofs::halo2curves::pasta::Fp;

pub fn to_field (val: i32) -> Fp
{
    let val_field: Fp;
    if val >= 0{
        val_field = Fp::from(val as u64);
    }
    else{
        val_field = -Fp::from(val as u64);
    }    
    return val_field;
}


// Just for the purpose of development of codebase
pub fn create_random_data(mut n: usize) -> Vec<i32> 
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
    return leaves;
}


#[cfg(test)]
mod tests {
    use super::*;
    use halo2_proofs::halo2curves::pasta::Fp;


    #[test]
    fn test_field_converter() 
    {
        let example_leaves = [13,2,3,4,6].to_vec();
        let mut example_leaves_field = vec![];
        let mut example_leaves_field_func = vec![];
        for i in 0..example_leaves.len(){
            example_leaves_field.push(Fp::from(example_leaves[i].clone() as u64));
            example_leaves_field_func.push(to_field(example_leaves[i]))
        }
        assert_eq!(example_leaves_field, example_leaves_field_func);

    }
}