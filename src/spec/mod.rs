use crate::spec::{
    event::EventEntry, init_memory::InitMemory, instruction::InstructionEntry, jump::JumpEntry,
    memory::MemoryEvent,
};

pub mod event;
pub mod init_memory;
pub mod instruction;
pub mod jump;
pub mod memory;

#[derive(Default)]
pub struct CompileTable {
    pub instructions: Vec<InstructionEntry>,
    pub init_memory: Vec<InitMemory>,
}

#[derive(Default)]
pub struct ExecutionTable {
    pub event: Vec<EventEntry>,
    pub memory: Vec<MemoryEvent>,
    pub jump: Vec<JumpEntry>,
}
