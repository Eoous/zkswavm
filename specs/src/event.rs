use crate::instruction::InstructionEntry;

#[derive(Clone)]
pub struct EventEntry {
    pub eid: u64,
    pub sp: u64,
    pub last_jump_eid: u64,
    pub instruction: InstructionEntry,
    pub step_info: RunInstructionTraceStep,
}
