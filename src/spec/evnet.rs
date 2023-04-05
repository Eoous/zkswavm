use wasmi::tracer::etable::{EEntry, RunInstructionTraceStep};

use crate::instruction::Instruction;

#[derive(Clone)]
pub struct Event {
    pub(crate) eid: u64,
    pub(crate) sp: u64,
    pub(crate) last_jump_eid: u64,
    pub(crate) instruction: Instruction,
    pub(crate) step_info: RunInstructionTraceStep,
}

impl From<&EEntry> for Event {
    fn from(eentry: &EEntry) -> Self {
        Event {
            eid: eentry.id,
            sp: eentry.sp,
            last_jump_eid: 0,
            instruction: Instruction::from(&eentry.inst),
            step_info: eentry.step.clone(),
        }
    }
}