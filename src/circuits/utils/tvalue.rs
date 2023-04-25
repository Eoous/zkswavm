use halo2_proofs::arithmetic::FieldExt;
use halo2_proofs::plonk::{Advice, Column, ConstraintSystem, Error, Expression, VirtualCells};
use specs::mtable::VarType;

use crate::circuits::range::RangeConfig;
use crate::circuits::utils::value_64::Value64Config;
use crate::circuits::utils::Context;
use crate::{constant_from, cur};

pub struct TValueConfig<F: FieldExt> {
    vtype: Column<Advice>,
    value: Value64Config<F>,
}

impl<F: FieldExt> TValueConfig<F> {
    pub fn configure(
        meta: &mut ConstraintSystem<F>,
        cols: &mut impl Iterator<Item = Column<Advice>>,
        range: &RangeConfig<F>,
        enable: impl Fn(&mut VirtualCells<'_, F>) -> Expression<F>,
    ) -> TValueConfig<F> {
        let value = Value64Config::configure(meta, cols, range, &enable);
        let vtype = cols.next().unwrap();

        for i in 0..8usize {
            range.configure_in_vtype_byte_range(
                meta,
                "tvalue byte",
                |meta| {
                    (
                        constant_from!(i),
                        cur!(meta, vtype.clone()),
                        cur!(meta, value.bytes_le[i].clone()),
                    )
                },
                &enable,
            );
        }

        TValueConfig { vtype, value }
    }

    pub fn assign(&self, ctx: &mut Context<F>, vtype: VarType, value: u64) -> Result<(), Error> {
        self.value.assign(ctx, value)?;

        ctx.region.assign_advice(
            || "tvalue vtype",
            self.vtype.clone(),
            ctx.offset,
            || Ok((vtype as u64).into()),
        )?;

        Ok(())
    }
}
