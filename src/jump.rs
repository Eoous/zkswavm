use std::marker::PhantomData;
use halo2_proofs::arithmetic::FieldExt;
use halo2_proofs::plonk::{Advice, Column, Error};

use crate::instruction::Instruction;
use crate::utils::{bn_to_field, Context};

pub struct Jump {
    eid: u64,
    last_jump_eid: u64,
    instruction: Box<Instruction>,
}

pub struct JumpConfig {
    cols: [Column<Advice>; 3],
}

pub struct JumpChip<F: FieldExt> {
    config: JumpConfig,
    _phantom: PhantomData<F>,
}

impl<F: FieldExt> JumpChip<F> {
    pub fn new(config: JumpConfig) -> JumpChip<F> {
        JumpChip {
            config,
            _phantom: PhantomData,
        }
    }

    pub fn add_jump(&self, ctx: &mut Context<'_, F>, jump: Box<Jump>) -> Result<(), Error> {
        ctx.region.assign_advice_from_constant(
            || "jump eid",
            self.config.cols[0],
            ctx.offset,
            F::from(jump.eid),
        )?;
        ctx.region.assign_advice_from_constant(
            || "jump last_jump_eid",
            self.config.cols[1],
            ctx.offset,
            F::from(jump.last_jump_eid),
        )?;
        ctx.region.assign_advice_from_constant(
            || "jump addr",
            self.config.cols[2],
            ctx.offset,
            bn_to_field::<F>(&jump.instruction.encode_addr()),
        )?;

        Ok(())
    }
}