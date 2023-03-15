use std::marker::PhantomData;
use halo2_proofs::arithmetic::FieldExt;
use halo2_proofs::plonk::{Advice, Column};
use crate::instruction::Instruction;

pub struct Event {
    id: u64,
    sp: u64,
    last_just_eid: u64,
    instruction: Instruction,
}

pub struct EventConfig {
    cols: Vec<Column<Advice>>,
}

pub struct EventChip<F: FieldExt> {
    config: EventConfig,
    _phantom: PhantomData<F>
}