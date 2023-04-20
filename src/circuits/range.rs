use halo2_proofs::arithmetic::FieldExt;
use halo2_proofs::circuit::Layouter;
use halo2_proofs::plonk::{ConstraintSystem, Error, Expression, TableColumn, VirtualCells};
use std::marker::PhantomData;

#[derive(Clone)]
pub struct RangeConfig<F: FieldExt> {
    common_col: TableColumn,
    byte_col: TableColumn,
    _mark: PhantomData<F>,
}

impl<F: FieldExt> RangeConfig<F> {
    pub fn configure(cols: [TableColumn; 2]) -> Self {
        RangeConfig {
            common_col: cols[0],
            byte_col: cols[1],
            _mark: PhantomData,
        }
    }

    pub fn configure_in_common_range(
        &self,
        meta: &mut ConstraintSystem<F>,
        key: &'static str,
        expr: impl FnOnce(&mut VirtualCells<'_, F>) -> Expression<F>,
    ) {
        meta.lookup(key, |meta| vec![(expr(meta), self.common_col)]);
    }

    pub fn configure_in_byte_range(
        &self,
        meta: &mut ConstraintSystem<F>,
        key: &'static str,
        expr: impl FnOnce(&mut VirtualCells<'_, F>) -> Expression<F>,
    ) {
        meta.lookup(key, |meta| vec![(expr(meta), self.byte_col)]);
    }
}

pub struct RangeChip<F: FieldExt> {
    config: RangeConfig<F>,
    _phantom: PhantomData<F>,
}

impl<F: FieldExt> RangeChip<F> {
    pub fn new(config: RangeConfig<F>) -> RangeChip<F> {
        RangeChip {
            config,
            _phantom: PhantomData,
        }
    }

    pub fn init(&self, layouter: &mut impl Layouter<F>, range: usize) -> Result<(), Error> {
        layouter.assign_table(
            || "common range table",
            |mut table| {
                for i in 0..range {
                    table.assign_cell(
                        || "range table",
                        self.config.common_col,
                        i,
                        || Ok(F::from(i as u64)),
                    )?;
                }

                Ok(())
            },
        )?;

        layouter.assign_table(
            || "byte range table",
            |mut table| {
                for i in 0..255usize {
                    table.assign_cell(
                        || "range table",
                        self.config.byte_col,
                        i,
                        || Ok(F::from(i as u64)),
                    )?;
                }

                Ok(())
            },
        )?;

        Ok(())
    }
}
