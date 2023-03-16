use std::marker::PhantomData;
use halo2_proofs::arithmetic::FieldExt;
use halo2_proofs::plonk::{Advice, Column};
use wasmi::tracer::etable::{EEntry, RunInstructionTraceStep};

use crate::instruction::Instruction;

pub struct Event {
    eid: u64,
    sp: u64,
    last_just_eid: u64,
    instruction: Instruction,
    step_info: RunInstructionTraceStep,
}

impl From<EEntry> for Event {
    fn from(eentry:EEntry ) -> Self {
        Event {
            eid: eentry.id,
            sp: eentry.sp,
            last_just_eid: 0,
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