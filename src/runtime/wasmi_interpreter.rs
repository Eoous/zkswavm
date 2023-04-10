use wasmi::RuntimeValue;
use crate::{
    runtime::types::{
        Value,
    },
    runtime::WasmRuntime,
};
use crate::runtime::{CompileOutcome, ExecutionOutcome};
use crate::runtime::types::{CompileError, ExecutionError};

pub struct WasmiRuntime {}

impl From<Value> for RuntimeValue {
    fn from(value: Value) -> RuntimeValue {
        match value {
            Value::i32(value) => {
                RuntimeValue::from(value)
            },
            Value::u32(value) => {
                RuntimeValue::from(value)
            },
            Value::i64(value) => {
                RuntimeValue::from(value)
            },
            Value::u64(value) => {
                RuntimeValue::from(value)
            },
        }
    }
}

impl WasmRuntime for WasmiRuntime {
    type Module = ();

    fn new() -> Self {
        todo!()
    }

    fn compile(&self, textual_repr: &str) -> Result<CompileOutcome<Self::Module>, CompileError> {
        todo!()
    }

    fn run(&self, compile_outcome: &CompileOutcome<Self::Module>, function_name: &str, args: Vec<Value>) -> Result<ExecutionOutcome, ExecutionError> {
        todo!()
    }
}