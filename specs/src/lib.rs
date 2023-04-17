use crate::{
    etable::EventTableEntry, imtable::InitMemoryTableEntry, itable::InstructionTableEntry,
    jtable::JumpTableEntry, mtable::MemoryTableEntry,
};

pub mod etable;
pub mod imtable;
pub mod itable;
pub mod jtable;
pub mod mtable;
pub mod step;
pub mod types;

#[derive(Default)]
pub struct CompileTable {
    pub instructions: Vec<InstructionTableEntry>,
    pub init_memory: Vec<InitMemoryTableEntry>,
}

#[derive(Default)]
pub struct ExecutionTable {
    pub event: Vec<EventTableEntry>,
    pub memory: Vec<MemoryTableEntry>,
    pub jump: Vec<JumpTableEntry>,
}
