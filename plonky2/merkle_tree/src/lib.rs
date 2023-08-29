//Oggn
mod circuit;

#[cfg(test)]
mod tests {
    use plonky2::field::goldilocks_field::GoldilocksField as F;
    use plonky2::plonk::{circuit_builder::CircuitBuilder,
                        circuit_data::CircuitConfig,
                        config::PoseidonGoldilocksConfig};
    use plonky2::iop::witness::{PartialWitness, WitnessWrite};
    use std::time::Instant;
    
    
// Observation: Taking very long (5 min appx), probably because of limitations of my local machine
      #[test]
    fn small_plonky2_example(){ 

        // In general for f(x,w) = y. Public: x,y; Private: w
        // let y = mx+c. Public: (x,y); Private: (m,c)
        let config = CircuitConfig::standard_recursion_zk_config(); // Only zk config ?
        let mut builder = CircuitBuilder::<F,2>::new(config);

        let x = builder.add_virtual_public_input();
        let y = builder.add_virtual_public_input();
        let m = builder.add_virtual_target();
        let c = builder.add_virtual_target();

        let mx = builder.mul(m, x);
        let ydash = builder.add(mx, c);

        builder.connect(ydash, y);

        let cktdata = builder.build::<PoseidonGoldilocksConfig>();
        let mut pw = PartialWitness::<F>::new();
        pw.set_target(x, F(2));
        pw.set_target(y, F(7));
        pw.set_target(m, F(3));
        pw.set_target(c, F(1));

        let start = Instant::now();
        let phi = cktdata.prove(pw).unwrap();
        let ver = cktdata.verify(phi).is_err();
        assert_eq!(ver, false);
        println!("time elapsed: {:?}", start.elapsed());
    }
    
}
