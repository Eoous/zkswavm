use std::marker::PhantomData;
use halo2_proofs::arithmetic::FieldExt;
use halo2_proofs::plonk::{Advice, Column};

use crate::instruction::Instruction;

pub struct Jump {
    eid: u64,
    last_jump_eid: u64,
    instruction: Instruction,
}

pub struct JumpConfig {
    cols: [Column<Advice>; 3],
}

pub struct JumpChip<F: FieldExt> {
    config: JumpConfig,
    _phantom: PhantomData<F>,
}