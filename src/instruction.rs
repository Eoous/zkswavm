use std::marker::PhantomData;
use halo2_proofs::arithmetic::FieldExt;
use halo2_proofs::circuit::Value;
use halo2_proofs::plonk::{Column, Error, Fixed};
use num_bigint::BigUint;
use num_traits::Zero;
use wasmi::tracer::itable::IEntry;

use crate::utils::{bn_to_field, Context};

pub struct Instruction {
    moid: u16,
    mmid: u16,
    fid: u16,
    bid: u16,
    iid: u16,
    opcode: u64,
}

impl Instruction {
    pub fn encode(&self) -> BigUint {
        let mut bn = self.encode_addr();
        bn <<= 64u8;
        bn += self.opcode;
        bn
    }

    pub fn encode_addr(&self) -> BigUint {
        let mut bn = BigUint::zero();
        bn <<= 16u8;
        bn += self.moid;
        bn <<= 16u8;
        bn <<= self.mmid;
        bn <<= 16u8;
        bn += self.fid;
        bn <<= 16u8;
        bn += self.bid;
        bn <<= 16u8;
        bn += self.iid;
        bn <<= 64u8;
        bn += self.opcode;
        bn
    }
}

impl From<IEntry> for Instruction {
    fn from(ientry: IEntry) -> Instruction {
        Instruction {
            moid: ientry.module_instance_index as u16,
            mmid: 0,
            fid: ientry.func_index as u16,
            bid: 0,
            iid: ientry.pc as u16,
            opcode: ientry.opcode as u64,
        }
    }
}

pub struct InstructionConfig {
    col: Column<Fixed>,
}

pub struct InstructionChip<F: FieldExt> {
    config: InstructionConfig,
    _phantom: PhantomData<F>,
}

impl<F: FieldExt> InstructionChip<F> {
    pub fn add_inst(&self, ctx: &mut Context<'_, F>, inst: Instruction) -> Result<(), Error> {
        let value: Value<F>= Value::known(bn_to_field(&inst.encode()));
        println!("{:?}", bn_to_field::<F>(&inst.encode()));
        ctx.region.assign_fixed(
            || "instruction table",
            self.config.col,
            ctx.offset,
            //|| Value::<F>::known(bn_to_field(&inst.encode())),
            || value.clone()
        )?;
        ctx.offset += 1;

        Ok(())
    }
}