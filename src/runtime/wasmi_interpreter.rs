use crate::runtime::memory_event_of_step;
use crate::runtime::{CompileOutcome, ExecutionOutcome, WasmRuntime};
use specs::etable::EventTableEntry;
use specs::mtable::MTable;
use specs::types::{CompileError, ExecutionError, Value};
use specs::ExecutionTable;
use specs::{itable::InstructionTableEntry, CompileTable};
use std::cell::RefCell;
use std::rc::Rc;
use wasmi::{ImportsBuilder, ModuleInstance, NopExternals, RuntimeValue};

pub struct WasmiRuntime {}

fn into_wasmi_value(v: Value) -> RuntimeValue {
    match v {
        Value::I32(v) => RuntimeValue::I32(v),
        Value::I64(v) => RuntimeValue::I64(v),
        Value::U32(_) => todo!(),
        Value::U64(_) => todo!(),
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
                    .map(|inst| inst.clone().into())
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

        assert_eq!(
            instance
                .invoke_export_trace(
                    function_name,
                    &args
                        .into_iter()
                        .map(|v| into_wasmi_value(v))
                        .collect::<Vec<_>>(),
                    &mut NopExternals,
                    tracer.clone(),
                )
                .expect("failed to execute export"),
            None,
        );

        let tracer = tracer.borrow();
        let events: Vec<_> = tracer.etable.0.iter().map(|e| e.clone().into()).collect();
        let mentries: Vec<_> = events
            .iter()
            .map(|e| memory_event_of_step(e, &mut 1))
            .collect();
        let mentries = mentries.into_iter().flat_map(|x| x.into_iter()).collect();
        let mut mtable = MTable::new(mentries);
        mtable.sort();

        let jumps = tracer
            .jtable
            .as_ref()
            .unwrap()
            .0
            .iter()
            .map(|jump| (*jump).clone().into())
            .collect::<Vec<_>>();

        Ok(ExecutionOutcome {
            tables: ExecutionTable {
                event: events,
                memory: mtable,
                jump: jumps,
            },
        })
    }
}
