use halo2_proofs::arithmetic::FieldExt;
use halo2_proofs::plonk::{Advice, Column, Error};
use num_bigint::BigUint;
use specs::jtable::JumpTableEntry;
use std::marker::PhantomData;

use crate::circuits::utils::{bn_to_field, Context};
use crate::circuits::Encode;

impl Encode for JumpTableEntry {
    fn encode(&self) -> BigUint {
        todo!()
    }
}

#[derive(Clone)]
pub struct JumpConfig<F: FieldExt> {
    col: Column<Advice>,
    _mark: PhantomData<F>,
}

impl<F: FieldExt> JumpConfig<F> {
    pub fn configure(cols: &mut impl Iterator<Item = Column<Advice>>) -> JumpConfig<F> {
        JumpConfig {
            col: cols.next().unwrap(),
            _mark: PhantomData,
        }
    }
}

pub struct JumpChip<F: FieldExt> {
    config: JumpConfig<F>,
    _phantom: PhantomData<F>,
}

impl<F: FieldExt> JumpChip<F> {
    pub fn new(config: JumpConfig<F>) -> JumpChip<F> {
        JumpChip {
            config,
            _phantom: PhantomData,
        }
    }

    pub fn add_jump(
        &self,
        ctx: &mut Context<'_, F>,
        jump: Box<JumpTableEntry>,
    ) -> Result<(), Error> {
        ctx.region.assign_advice_from_constant(
            || "jump table entry",
            self.config.col,
            ctx.offset,
            bn_to_field(&jump.encode()),
        )?;

        Ok(())
    }
}
