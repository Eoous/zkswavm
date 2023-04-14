use std::cell::RefCell;
use std::rc::Rc;
use wasmi::{ImportsBuilder, ModuleInstance, RuntimeValue};

use crate::runtime::memory_event_of_step;
use crate::spec::event::EventEntry;
use crate::spec::ExecutionTable;
use crate::{
    runtime::{
        types::{CompileError, ExecutionError, Value},
        CompileOutcome, ExecutionOutcome, WasmRuntime,
    },
    spec::{instruction::InstructionEntry, CompileTable},
};

pub struct WasmiRuntime {}

impl From<Value> for RuntimeValue {
    fn from(value: Value) -> RuntimeValue {
        match value {
            Value::i32(value) => RuntimeValue::from(value),
            Value::u32(value) => RuntimeValue::from(value),
            Value::i64(value) => RuntimeValue::from(value),
            Value::u64(value) => RuntimeValue::from(value),
        }
    }
}

impl WasmRuntime for WasmiRuntime {
    type Module = wasmi::Module;

    fn new() -> WasmiRuntime {
        WasmiRuntime {}
    }

    fn compile(&self, textual_repr: &str) -> Result<CompileOutcome<Self::Module>, CompileError> {
        let binary = wabt::wat2wasm(&textual_repr).expect("failed to parse wat.");
        let module = wasmi::Module::from_buffer(&binary).expect("failed to load wasm binary.");

        let instance = ModuleInstance::new(&module, &ImportsBuilder::default())
            .expect("failed to instantiate wasm module.")
            .assert_no_start();

        let mut tracer = wasmi::tracer::Tracer::default();
        tracer.register_module_instance(&instance);

        Ok(CompileOutcome {
            textual_repr: textual_repr.to_string(),
            module,
            tables: CompileTable {
                instructions: tracer
                    .itable
                    .0
                    .iter()
                    .map(|inst| InstructionEntry::from(inst))
                    .collect(),
                init_memory: vec![], // todo
            },
        })
    }

    fn run(
        &self,
        compile_outcome: &CompileOutcome<Self::Module>,
        function_name: &str,
        args: Vec<Value>,
    ) -> Result<ExecutionOutcome, ExecutionError> {
        let instance = ModuleInstance::new(&compile_outcome.module, &ImportsBuilder::default())
            .expect("failed to instantiate wasm module")
            .assert_no_start();

        let mut tracer = wasmi::tracer::Tracer::default();
        tracer.register_module_instance(&instance);
        let tracer = Rc::new(RefCell::new(tracer));

        let tracer = tracer.borrow();
        let events: Vec<_> = tracer
            .etable
            .0
            .iter()
            .map(|e| EventEntry::from(e))
            .collect();
        let memorys: Vec<_> = events.iter().map(|e| memory_event_of_step(e)).collect();
        let jumps = vec![];

        Ok(ExecutionOutcome {
            tables: ExecutionTable {
                event: events,
                memory: memorys.into_iter().flat_map(|x| x.into_iter()).collect(),
                jump: jumps,
            },
        })
    }
}
