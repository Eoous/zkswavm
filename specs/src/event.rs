use crate::{instruction::InstructionEntry, step::StepInfo};

#[derive(Clone)]
pub struct EventEntry {
    pub eid: u64,
    pub sp: u64,
    pub last_jump_eid: u64,
    pub instruction: InstructionEntry,
    pub step_info: StepInfo,
}
