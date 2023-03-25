use std::marker::PhantomData;
use halo2_proofs::arithmetic::FieldExt;
use halo2_proofs::plonk::{Advice, Column};
use wasmi::tracer::etable::{EEntry, RunInstructionTraceStep};

use crate::instruction::Instruction;

pub struct Event {
    pub(crate) eid: u64,
    pub(crate) sp: u64,
    pub(crate) last_jump_eid: u64,
    pub(crate) instruction: Instruction,
    pub(crate) step_info: RunInstructionTraceStep,
}

impl From<EEntry> for Event {
    fn from(eentry:EEntry ) -> Self {
        Event {
            eid: eentry.id,
            sp: eentry.sp,
            last_jump_eid: 0,
            instruction: Instruction::from(eentry.inst),
            step_info: eentry.step,
        }
    }
}

pub struct EventConfig {
    cols: [Column<Advice>; 4],
    aux_cols: [Column<Advice>; 4],
}

impl EventConfig {
    pub fn new(cols: [Column<Advice>; 4], aux_cols: [Column<Advice>; 4]) -> EventConfig {
        EventConfig {
            cols,
            aux_cols
        }
    }
}

pub struct EventChip<F: FieldExt> {
    config: EventConfig,
    _phantom: PhantomData<F>
}

impl<F: FieldExt> EventChip<F> {
    pub fn new(config: EventConfig) -> EventChip<F> {
        EventChip {
            config,
            _phantom: PhantomData,
        }
    }
}