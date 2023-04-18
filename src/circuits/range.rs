use std::marker::PhantomData;
use halo2_proofs::arithmetic::FieldExt;
use halo2_proofs::circuit::{Layouter};
use halo2_proofs::plonk::{ConstraintSystem, Error, Expression, TableColumn, VirtualCells};

#[derive(Clone)]
pub struct RangeConfig<F: FieldExt> {
    cols: [TableColumn; 1],
    _mark: PhantomData<F>,
}

impl<F: FieldExt> RangeConfig<F> {
    pub fn configure(cols: [TableColumn; 1]) -> Self {
        RangeConfig {
            cols,
            _mark: PhantomData,
        }
    }

    pub fn configure_in_range(
        &self,
        meta: &mut ConstraintSystem<F>,
        key: &'static str,
        expr: impl FnOnce(&mut VirtualCells<'_, F>) -> Expression<F>,
    ) {
        meta.lookup(key, |meta| vec![(expr(meta), self.cols[0])]);
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
                        self.config.cols[0],
                        i,
                        || Ok(F::from(i as u64)),
                    )?;
                }

                Ok(())
            }
        )?;

        Ok(())
    }
}