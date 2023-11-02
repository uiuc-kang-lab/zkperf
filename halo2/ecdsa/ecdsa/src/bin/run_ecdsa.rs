use ecdsa::test::test_ecdsa_verifier;

fn main() {

    let step = std::env::args().nth(1).expect("Step to Process");
    
    test_ecdsa_verifier(step);
}