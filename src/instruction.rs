use std::marker::PhantomData;
use halo2_proofs::arithmetic::FieldExt;
use halo2_proofs::circuit::{Layouter};
use halo2_proofs::plonk::{Column, ConstraintSystem, Error, Expression, Fixed, TableColumn, VirtualCells};
use num_bigint::BigUint;
use num_traits::{One, Zero};
use wasmi::tracer::itable::IEntry;

use crate::utils::{bn_to_field, Context};
use crate::{
  constant
};

pub struct Instruction {
    moid: u16,
    pub(crate) mmid: u16,
    fid: u16,
    bid: u16,
    iid: u16,
    opcode: u64,
    aux: u64
}

impl Instruction {
    pub fn encode(&self) -> BigUint {
        let mut bn = self.encode_addr();
        bn <<= 64u8;
        bn += self.opcode;
        bn <<= 64u8;
        bn += self.aux;
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

impl From<&IEntry> for Instruction {
    fn from(ientry: &IEntry) -> Instruction {
        Instruction {
            moid: ientry.module_instance_index as u16,
            mmid: ientry.module_instance_index as u16,
            fid: ientry.func_index as u16,
            bid: 0,
            iid: ientry.pc as u16,
            opcode: ientry.opcode as u64,
            aux: 0,
        }
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

pub struct InstructionConfig<F: FieldExt> {
    col: TableColumn,
    _mark: PhantomData<F>,
}

impl<F: FieldExt> InstructionConfig<F> {
    pub fn configure_in_table(&self, meta: &mut ConstraintSystem<F>, key: &'static str, expr: impl FnOnce(&mut VirtualCells<'_, F>) -> Expression<F>) {
        meta.lookup(key, |meta| vec![(expr(meta), self.col)]);
    }
}

pub struct InstructionChip<F: FieldExt> {
    config: InstructionConfig<F>,
}

impl<F: FieldExt> InstructionChip<F> {
    pub fn add_inst(&self, layouter: &mut impl Layouter<F>, insts: Vec<Instruction>) -> Result<(), Error> {
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