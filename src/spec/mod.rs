use crate::spec::{
    evnet::EventEntry,
    init_memory::InitMemory,
    instruction::InstructionEntry,
    memory::MemoryEvent,
    jump::JumpEntry,
};

pub mod evnet;
pub mod init_memory;
pub mod instruction;
pub mod jump;
pub mod memory;

pub struct CompileTable {
    pub instructions: Vec<InstructionEntry>,
    pub init_memory: Vec<InitMemory>,
}

pub struct ExecutionTable {
    pub event: Vec<EventEntry>,
    pub memory: Vec<MemoryEvent>,
    pub jump: Vec<JumpEntry>,
}
