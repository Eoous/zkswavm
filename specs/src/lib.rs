use crate::{
    event::EventEntry, init_memory::InitMemoryEntry, instruction::InstructionEntry,
    jump::JumpEntry, memory::MemoryEntry,
};

pub mod event;
pub mod init_memory;
pub mod instruction;
pub mod jump;
pub mod memory;
pub mod step;
pub mod types;

#[derive(Default)]
pub struct CompileTable {
    pub instructions: Vec<InstructionEntry>,
    pub init_memory: Vec<InitMemoryEntry>,
}

#[derive(Default)]
pub struct ExecutionTable {
    pub event: Vec<EventEntry>,
    pub memory: Vec<MemoryEntry>,
    pub jump: Vec<JumpEntry>,
}
