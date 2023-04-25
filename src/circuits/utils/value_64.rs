use halo2_proofs::arithmetic::FieldExt;
use halo2_proofs::plonk::{Advice, Column, ConstraintSystem, Error, Expression, VirtualCells};
use std::marker::PhantomData;

use crate::circuits::range::RangeConfig;
use crate::circuits::utils::Context;
use crate::{constant, cur};

#[derive(Clone)]
pub struct Value64Config<F: FieldExt> {
    pub bytes_le: [Column<Advice>; 8],
    pub value: Column<Advice>,
    _mark: PhantomData<F>,
}
impl<F: FieldExt> Value64Config<F> {
    pub fn configure(
        meta: &mut ConstraintSystem<F>,
        cols: &mut impl Iterator<Item = Column<Advice>>,
        range: &RangeConfig<F>,
        enable: impl Fn(&mut VirtualCells<'_, F>) -> Expression<F>,
    ) -> Value64Config<F> {
        let bytes_le = [0; 8].map(|_| cols.next().unwrap());
        let value = cols.next().unwrap();

        for byte in bytes_le.iter() {
            range.configure_in_byte_range(meta, "byte", |meta| {
                cur!(meta, byte.clone()) * enable(meta)
            });
        }

        meta.create_gate("value_64 sum", |meta| {
            let mut acc = cur!(meta, bytes_le[0].clone());
            let mut base = F::one();

            for i in 1..8usize {
                base = base * F::from(256u64);
                acc = acc + constant!(base) * cur!(meta, bytes_le[i].clone());
            }
            vec![(acc - cur!(meta, value.clone())) * enable(meta)]
        });

        Value64Config {
            bytes_le,
            value,
            _mark: PhantomData,
        }
    }

    pub fn assign(&self, ctx: &mut Context<'_, F>, value: u64) -> Result<(), Error> {
        ctx.region.assign_advice(
            || "value 64",
            self.value.clone(),
            ctx.offset,
            || Ok(value.into()),
        )?;

        let bytes = value.to_le_bytes();
        for i in 0..8 {
            ctx.region.assign_advice(
                || " value 64 byte",
                self.bytes_le[i],
                ctx.offset,
                || Ok((bytes[i] as u64).into()),
            )?;
        }

        Ok(())
    }
}
