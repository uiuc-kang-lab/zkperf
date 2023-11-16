use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    halo2curves::{
        ff::PrimeField,
        bn256::{Bn256, Fr, G1Affine}
    },
    plonk::{
        Advice, Circuit, Column, ConstraintSystem, Error,
        create_proof, keygen_pk, keygen_vk, verify_proof, TableColumn
    },
    poly::{
        kzg::{
          commitment::{KZGCommitmentScheme, ParamsKZG},
          multiopen::{ProverSHPLONK, VerifierSHPLONK},
          strategy::SingleStrategy,
        },
        Rotation
    },
    transcript::{
        Blake2bRead, Blake2bWrite, Challenge255, TranscriptReadBuffer, TranscriptWriterBuffer,
      },
};
use std::time::Instant;

#[derive(Clone, Debug)]
struct LookupConfig {
    lookup: TableColumn,
    advice: Column<Advice>,
}

#[derive(Clone, Debug, Default)]
struct LookupCircuit<F: PrimeField> {
    a: Vec<Value<F>>,
    m: usize,
}

impl<F: PrimeField> Circuit<F> for LookupCircuit<F> {
    type Config = LookupConfig;
    type FloorPlanner = SimpleFloorPlanner;
    type Params = ();

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let lookup = meta.lookup_table_column();
        let advice = meta.advice_column();
        meta.enable_equality(advice);

        meta.lookup(
            "lookup indexing", 
            |meta| {
                let advice = meta.query_advice(advice, Rotation::cur());
                vec![(advice, lookup)]
            }
        );

        Self::Config {
            lookup,
            advice,
        }
    }

    fn synthesize(&self, config: Self::Config, mut layouter: impl Layouter<F>) -> Result<(), Error> {

        layouter.assign_region(
            || "test advice", 
            |mut region| {
                for (idx, val) in self.a.iter().enumerate() {
                    region.assign_advice(
                        || "advice cell", 
                        config.advice, idx,
                        || *val
                    )?;
                }
                Ok(())
            }
        )?;

        layouter.assign_table(
            || "random lookup", 
            |mut table| {
                for i in 0..self.m {
                    table.assign_cell(
                        || "table cell", 
                        config.lookup, i, 
                        || Value::known(F::from(i as u64))
                    )?;
                }
                Ok(())
            }
        )?;
        Ok(())
    }

}

fn time_kzg(k: usize, circuit: LookupCircuit<Fr>) {
    
    let rng = rand::thread_rng();
    let params = ParamsKZG::<Bn256>::setup(k as u32, rng);
    let vk_circuit = circuit.clone();
    let vk = keygen_vk(&params, &vk_circuit).unwrap();
    let pk_circuit = circuit.clone();
    let pk = keygen_pk(&params, vk.clone(), &pk_circuit).unwrap();
    let proof_circuit = circuit.clone();
    let rng = rand::thread_rng();
    let start = Instant::now();
    let mut transcript = Blake2bWrite::<_, G1Affine, Challenge255<_>>::init(vec![]);
    create_proof::<
      KZGCommitmentScheme<Bn256>,
      ProverSHPLONK<'_, Bn256>,
      Challenge255<G1Affine>,
      _,
      Blake2bWrite<Vec<u8>, G1Affine, Challenge255<G1Affine>>,
      LookupCircuit<Fr>,
    >(
      &params,
      &pk,
      &[proof_circuit],
      &[&[]],
      rng,
      &mut transcript,
    )
    .unwrap();
    let proof = transcript.finalize();
    let prove_time = start.elapsed();
    println!("prove time: {:?}", prove_time);
    println!("proof size: {}", proof.len());

    let strategy = SingleStrategy::new(&params);
    let mut transcript_read = Blake2bRead::<_, _, Challenge255<_>>::init(&proof[..]);
    assert!(
        verify_proof::<
          KZGCommitmentScheme<Bn256>,
          VerifierSHPLONK<'_, Bn256>,
          Challenge255<G1Affine>,
          Blake2bRead<&[u8], G1Affine, Challenge255<G1Affine>>,
          SingleStrategy<'_, Bn256>,
        >(&params, &vk, strategy, &[&[]], &mut transcript_read)
        .is_ok(),
        "proof did not verify"
    );
    println!("verify time: {:?}", start.elapsed()-prove_time);

}

fn main() {

    let queries = [16374, 32758, 65526];
    let cols = [1000, 10000, 100000, 1000000, 10000000];
    // let queries = [16734];
    // let cols = [100000];
    
    for query in queries {
        for col in cols {
            let a = vec![Value::known(Fr::from(0)); query];
            let circuit = LookupCircuit {
                a,
                m: col
            };
            let bigger = if query > col { query } else { col };
            let k = (bigger as f32 + 1.0).log2().ceil();
            println!("testing circuit: [{}, {}, {}]", query, col, k as usize);
            time_kzg(k as usize, circuit);
        }
    }
}