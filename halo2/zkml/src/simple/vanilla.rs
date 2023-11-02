use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    plonk::{Advice, Assigned, Circuit, Column, ConstraintSystem, Error, Fixed, TableColumn},
    poly::Rotation,
};
use halo2_curves::ff::PrimeField;
use plonkish_backend::util::Itertools;
use rand::RngCore;
use std::iter;

#[derive(Clone, Debug)]
pub struct DegreeConfig {
    selector: Column<Fixed>,
    wire: Column<Advice>,
    table: TableColumn,
}


impl DegreeConfig {
    fn configure<F: PrimeField>(meta: &mut ConstraintSystem<F>) -> Self {
        let pi = meta.instance_column();
        let q = meta.fixed_column();
        let w = meta.advice_column();
        let table = meta.lookup_table_column();
        meta.enable_equality(w);

        meta.create_gate(
            "q * w + pi = 0",
            |meta| {
                let w = meta.query_advice(w, Rotation::cur());
                let q = meta.query_fixed(q, Rotation::cur());
                let pi = meta.query_instance(pi, Rotation::cur());

                Some(
                    q * w + pi,
                )
            },
        );

        meta.lookup(
            "w^2",
            |meta| {
                let wire = meta.query_advice(w, Rotation::cur());
                vec![(wire, table)]
            }
        );

        DegreeConfig {
            selector: q,
            wire: w,
            table
        }
    }
}


#[derive(Clone, Default, Debug)]
pub struct Degree<F: PrimeField>(usize, Vec<[Assigned<F>; 2]>);

impl<F: PrimeField> Degree<F> {
    pub fn rand(k: usize, mut rng: impl RngCore) -> Self {

        let mut values = vec![];
        for i in 0..50 {
            values.push([Assigned::Trivial(F::from(1)), Assigned::Trivial(F::from((30 as u64).pow(2)))]);
        }

        // let mut rand_row =
        //     || {
        //         [
        //             Assigned::Trivial(F::from(1)), 
        //             Assigned::Trivial(F::from( rng.next_u64() & 0xF )),
        //         ]
        //     };
        // let values = iter::repeat_with(|| {
        //     rand_row()
        // })
        // .take((1 << k) - 1)
        // .collect_vec();
        // println!("{}", values.len());
        Self(k, values)
    }

    pub fn num_instances(k: usize) -> Vec<usize> {
        vec![1<<k-1]
    }

    pub fn instances(&self) -> Vec<Vec<F>> {
        let pi = self.1.iter().map(| arr | {
            let [q, w] = arr;
            let w_f = {
                let mut tmp = w.clone();
                // for _ in 1..DEG {
                //     tmp = tmp * w.clone();
                // }
                tmp
            };
            (-q*w_f).evaluate()
        }).collect_vec();
        vec![pi]
    }


}


impl<F: PrimeField> Circuit<F> for Degree<F> {
    type Config = DegreeConfig;
    type FloorPlanner = SimpleFloorPlanner;
    type Params = ();

    fn without_witnesses(&self) -> Self {
        unimplemented!()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        // meta.set_minimum_degree(4);
        DegreeConfig::configure(meta)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "",
            |mut region| {
                for (offset, values) in self.1.iter().enumerate() {
                    let (selector, wire) = (values[0], values[1]);
                    region.assign_fixed(|| "", config.selector, offset, || Value::known(selector))?;
                    let cell = region.assign_advice(|| "", config.wire, offset, || Value::known(wire))?;
                    if offset > 0 {
                        cell.copy_advice(|| "", &mut region, config.wire, offset-1)?;
                    }
                }
                Ok(())
            },
        )?;

        layouter.assign_table(
            || "square table", 
            |mut table| {
                for i in 0..50 {
                    table.assign_cell(
                        || "", config.table, i, || Value::known(F::from((i * i) as u64)))?;
                }
                Ok(())
            }
        )
    }
}


