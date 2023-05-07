use crate::types::Value;
use parity_wasm::elements::ValueType;
use strum_macros::EnumIter;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LocationType {
    Heap = 0,
    Stack = 1,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AccessType {
    Read = 1,
    Write = 2,
    Init = 3,
}

#[derive(Clone, Copy, Debug, PartialEq, EnumIter)]
pub enum VarType {
    U8 = 1,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
}

impl From<ValueType> for VarType {
    fn from(value: ValueType) -> VarType {
        match value {
            ValueType::I32 => Self::I32,
            ValueType::I64 => Self::I64,
            ValueType::F32 => todo!(),
            ValueType::F64 => todo!(),
        }
    }
}

impl From<crate::types::ValueType> for VarType {
    fn from(value: crate::types::ValueType) -> VarType {
        match value {
            crate::types::ValueType::I32 => VarType::I32,
            crate::types::ValueType::I64 => VarType::I64,
            _ => todo!(),
        }
    }
}

impl VarType {
    pub fn byte_size(&self) -> u64 {
        match self {
            VarType::U8 => 1,
            VarType::I8 => 1,
            VarType::U16 => 2,
            VarType::I16 => 2,
            VarType::U32 => 4,
            VarType::I32 => 4,
            VarType::U64 => 8,
            VarType::I64 => 8,
        }
    }
}

#[derive(Clone, Debug)]
pub struct MemoryTableEntry {
    pub eid: u64,
    pub emid: u64,
    pub mmid: u64,
    pub offset: u64,
    pub ltype: LocationType,
    pub atype: AccessType,
    pub vtype: VarType,
    pub value: u64,
}

impl MemoryTableEntry {
    pub fn is_same_location(&self, other: &MemoryTableEntry) -> bool {
        self.mmid == other.mmid && self.offset == other.offset && self.ltype == other.ltype
    }
}

#[derive(Default)]
pub struct MTable(Vec<MemoryTableEntry>);

impl MTable {
    pub fn new(entries: Vec<MemoryTableEntry>) -> MTable {
        MTable(entries)
    }

    pub fn sort(&mut self) {
        self.0
            .sort_by_key(|item| (item.ltype, item.mmid, item.eid, item.emid))
    }

    pub fn entries(&self) -> &Vec<MemoryTableEntry> {
        &self.0
    }
}
