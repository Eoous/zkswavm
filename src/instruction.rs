use std::marker::PhantomData;
use halo2_proofs::arithmetic::FieldExt;
use halo2_proofs::circuit::{Layouter};
use halo2_proofs::plonk::{Column, ConstraintSystem, Error, Expression, Fixed, TableColumn, VirtualCells};
use num_bigint::BigUint;
use num_traits::{One, Zero};
use wasmi::tracer::itable::IEntry;

use crate::{
    constant,
    utils::{bn_to_field, Context},
    spec::instruction::InstructionEntry,
};

impl InstructionEntry {
    pub fn encode(&self) -> BigUint {
        let opcode: BigUint = self.opcode.into();
        let mut bn = self.encode_addr();
        bn <<= 128usize;
        bn += opcode;
        bn
    }

    pub fn encode_addr(&self) -> BigUint {
        let mut bn = BigUint::zero();
        bn += self.moid;
        bn <<= 16u8;
        bn += self.mmid;
        bn <<= 16u8;
        bn += self.fid;
        bn <<= 16u8;
        bn += self.bid;
        bn <<= 16u8;
        bn += self.iid;
        bn
    }
}

impl From<&IEntry> for InstructionEntry {
    fn from(ientry: &IEntry) -> InstructionEntry {
        todo!()
    }
}

pub fn encode_inst_expr<F: FieldExt>(
    moid: Expression<F>,
    mmid: Expression<F>,
    fid: Expression<F>,
    bid: Expression<F>,
    iid: Expression<F>,
    opcode: Expression<F>,
) -> Expression<F> {
    let mut bn = BigUint::one();
    let mut acc = opcode;
    bn <<= 64u8;
    acc = acc + iid * constant!(bn_to_field(&bn));
    bn <<= 16u8;
    acc = acc + bid * constant!(bn_to_field(&bn));
    bn <<= 16u8;
    acc = acc + fid * constant!(bn_to_field(&bn));
    bn <<= 16u8;
    acc = acc + mmid * constant!(bn_to_field(&bn));
    bn <<= 16u8;
    acc = acc + moid * constant!(bn_to_field(&bn));

    acc
}

#[derive(Clone)]
pub struct InstructionConfig<F: FieldExt> {
    col: TableColumn,
    _mark: PhantomData<F>,
}

impl<F: FieldExt> InstructionConfig<F> {
    pub fn new(meta: &mut ConstraintSystem<F>) -> InstructionConfig<F> {
        InstructionConfig {
            col: meta.lookup_table_column(),
            _mark: PhantomData,
        }
    }

    pub fn configure_in_table(&self, meta: &mut ConstraintSystem<F>, key: &'static str, expr: impl FnOnce(&mut VirtualCells<'_, F>) -> Expression<F>) {
        meta.lookup(key, |meta| vec![(expr(meta), self.col)]);
    }
}

#[derive(Clone)]
pub struct InstructionChip<F: FieldExt> {
    config: InstructionConfig<F>,
}

impl<F: FieldExt> InstructionChip<F> {
    pub fn new(config: InstructionConfig<F>) -> InstructionChip<F> {
        InstructionChip {
            config
        }
    }

    pub fn add_inst(&self, layouter: &mut impl Layouter<F>, insts: &Vec<InstructionEntry>) -> Result<(), Error> {
        layouter.assign_table(
            || "init instructions",
            |mut table| {
                for(i, v) in insts.iter().enumerate() {
                    table.assign_cell(
                        || "init instruction table",
                        self.config.col,
                        i,
                        || Ok(bn_to_field::<F>(&v.encode())),
                    )?;
                }

                Ok(())
            },
        )?;

        Ok(())
    }
}