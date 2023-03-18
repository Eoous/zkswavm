use std::marker::PhantomData;
use halo2_proofs::arithmetic::FieldExt;
use halo2_proofs::plonk::{Advice, Column, ConstraintSystem, Expression, VirtualCells};
use halo2_proofs::poly::Rotation;

pub struct RowDiffConfig<F: FieldExt> {
    data: Column<Advice>,
    same: Column<Advice>,
    _inv: Column<Advice>,
    _mark: PhantomData<F>,
}

impl<F: FieldExt> RowDiffConfig<F> {
    pub fn configure(name: &'static str, meta: &mut ConstraintSystem<F>, cols: &mut impl Iterator<Item = Column<Advice>>) -> RowDiffConfig<F> {
        let data = cols.next().unwrap();
        let same = cols.next().unwrap();
        let inv = cols.next().unwrap();
        meta.create_gate(name, |meta| {
            let cur = meta.query_advice(data, Rotation::cur());
            let pre = meta.query_advice(data, Rotation::prev());
            let inv = meta.query_advice(inv, Rotation::cur());
            let same = meta.query_advice(same, Rotation::cur());

            vec![
                (cur.clone() - pre.clone()) * inv.clone()
                             - same.clone()
                             - Expression::Constant(F::one()),
                (cur.clone() - pre.clone()) * same.clone(),
            ]
        });

        RowDiffConfig {
            data,
            same,
            _inv: inv,
            _mark: PhantomData,
        }
    }

    pub fn is_same(&self, meta: &mut VirtualCells<F>) -> Expression<F> {
        meta.query_advice(self.same, Rotation::cur())
    }

    pub fn data(&self, meta: &mut VirtualCells<F>) -> Expression<F> {
        meta.query_advice(self.data, Rotation::cur())
    }

    pub fn diff(&self, meta: &mut VirtualCells<F>) -> Expression<F> {
        let cur = meta.query_advice(self.data, Rotation::cur()),
        let pre = meta.query_advice(self.data, Rotation::prev()),
        cur - pre
    }
}