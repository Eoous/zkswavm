use crate::circuits::utils::Context;
use halo2_proofs::arithmetic::FieldExt;
use halo2_proofs::plonk::{Advice, Column, ConstraintSystem, Error, Expression, VirtualCells};
use halo2_proofs::poly::Rotation;
use std::marker::PhantomData;

#[derive(Clone)]
pub struct RowDiffConfig<F: FieldExt> {
    data: Column<Advice>,
    same: Column<Advice>,
    inv: Column<Advice>,
    _mark: PhantomData<F>,
}

impl<F: FieldExt> RowDiffConfig<F> {
    pub fn configure(
        name: &'static str,
        meta: &mut ConstraintSystem<F>,
        cols: &mut impl Iterator<Item = Column<Advice>>,
        enable: impl FnOnce(&mut VirtualCells<'_, F>) -> Expression<F>,
    ) -> RowDiffConfig<F> {
        let data = cols.next().unwrap();
        let same = cols.next().unwrap();
        let inv = cols.next().unwrap();
        meta.create_gate(name, |meta| {
            let enable = enable(meta);
            let cur = meta.query_advice(data, Rotation::cur());
            let pre = meta.query_advice(data, Rotation::prev());
            let inv = meta.query_advice(inv, Rotation::cur());
            let same = meta.query_advice(same, Rotation::cur());

            // (cur - pre) * inv - same - 1 == 0
            // (cur - pre) * same == 0
            vec![
                (cur.clone() - pre.clone()) * inv.clone()
                    - same.clone()
                    - Expression::Constant(F::one()),
                (cur.clone() - pre.clone()) * same.clone(),
            ]
            .into_iter()
            .map(|x| x * enable.clone())
            .collect::<Vec<Expression<F>>>()
        });

        RowDiffConfig {
            data,
            same,
            inv: inv,
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
        let cur = meta.query_advice(self.data, Rotation::cur());
        let pre = meta.query_advice(self.data, Rotation::prev());
        cur - pre
    }

    pub fn assign(&self, ctx: &mut Context<F>, data: F, diff: F) -> Result<(), Error> {
        ctx.region
            .assign_advice_from_constant(|| "row diff data", self.data, ctx.offset, data)?;

        ctx.region.assign_advice(
            || "row diff inv",
            self.inv,
            ctx.offset,
            || Ok(diff.invert().unwrap_or(F::zero())),
        )?;
        ctx.region.assign_advice(
            || "row diff same",
            self.same,
            ctx.offset,
            || {
                Ok(if diff.is_zero().into() {
                    F::one()
                } else {
                    F::zero()
                })
            },
        )?;

        Ok(())
    }
}
