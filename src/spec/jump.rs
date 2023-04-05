use crate::spec::instruction::InstructionEntry;

pub struct JumpEntry {
    eid: u64,
    last_jump_eid: u64,
    instruction: Box<InstructionEntry>,
}