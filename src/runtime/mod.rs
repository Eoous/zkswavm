pub mod types;
pub mod wasmi_interpreter;

use wasmi::tracer::etable::RunInstructionTraceStep;

use crate::runtime::{
    types::{CompileError, ExecutionError, Value},
    wasmi_interpreter::WasmiRuntime,
};

use crate::spec::{
    event::EventEntry,
    memory::{AccessType, LocationType, MemoryEvent, VarType},
    CompileTable, ExecutionTable,
};

pub struct CompileOutcome<M> {
    pub textual_repr: String,
    pub module: M,
    pub tables: CompileTable,
}

pub struct ExecutionOutcome {
    pub tables: ExecutionTable,
}

pub trait WasmRuntime {
    type Module;

    fn new() -> Self;
    fn compile(&self, textual_repr: &str) -> Result<CompileOutcome<Self::Module>, CompileError>;
    fn run(
        &self,
        compile_outcome: &CompileOutcome<Self::Module>,
        function_name: &str,
        args: Vec<Value>,
    ) -> Result<ExecutionOutcome, ExecutionError>;
}

pub type WasmInterpreter = WasmiRuntime;

pub fn memory_event_of_step(event: &EventEntry) -> Vec<MemoryEvent> {
    let eid = event.eid;
    let mmid = event.instruction.mmid.into();

    match &event.step_info {
        RunInstructionTraceStep::BrIfNez { value, dst_pc } => mem_op_from_stack_only_step(
            eid,
            mmid,
            VarType::I32,
            VarType::I32,
            &[*value as u64],
            &[],
        ),
        RunInstructionTraceStep::Return {
            drop,
            keep,
            drop_values,
            keep_values,
        } => {
            assert_eq!(*drop as usize, drop_values.len());
            assert_eq!(*keep as usize, keep_values.len());
            mem_op_from_stack_only_step(
                eid,
                mmid,
                VarType::I32,
                VarType::I32,
                drop_values.iter().map(|value| value.0).collect::<Vec<_>>()[..]
                    .try_into()
                    .unwrap(),
                keep_values.iter().map(|value| value.0).collect::<Vec<_>>()[..]
                    .try_into()
                    .unwrap(),
            )
        }
        RunInstructionTraceStep::Call { index } => {
            vec![]
        }
        RunInstructionTraceStep::GetLocal { depth, value } => {
            vec![
                MemoryEvent {
                    eid,
                    emid: 1,
                    mmid,
                    offset: *depth as u64,
                    ltype: LocationType::Stack,
                    atype: AccessType::Read,
                    vtype: VarType::I32,
                    value: value.0,
                },
                MemoryEvent {
                    eid,
                    emid: 1,
                    mmid: mmid.into(),
                    offset: 0,
                    ltype: LocationType::Stack,
                    atype: AccessType::Write,
                    vtype: VarType::I32,
                    value: value.0,
                },
            ]
        }
        RunInstructionTraceStep::I32Const { value } => mem_op_from_stack_only_step(
            eid,
            mmid,
            VarType::I32,
            VarType::I32,
            &[],
            &[*value as u64],
        ),
        RunInstructionTraceStep::I32BinOp { left, right, value } => mem_op_from_stack_only_step(
            eid,
            mmid,
            VarType::I32,
            VarType::I32,
            &[*right as u64, *left as u64],
            &[*value as u64],
        ),
        RunInstructionTraceStep::I32Comp { left, right, value } => mem_op_from_stack_only_step(
            eid,
            mmid,
            VarType::I32,
            VarType::I32,
            &[*right as u64, *left as u64],
            &[*value as u64],
        ),
    }
}

fn mem_op_from_stack_only_step(
    eid: u64,
    mmid: u64,
    inputs_type: VarType,
    outputs_type: VarType,
    pop_values: &[u64],
    push_values: &[u64],
) -> Vec<MemoryEvent> {
    let mut mem_ops = vec![];

    for i in 0..pop_values.len() {
        mem_ops.push(MemoryEvent {
            eid,
            emid: 1,
            mmid,
            offset: i as u64,
            ltype: LocationType::Stack,
            atype: AccessType::Read,
            vtype: inputs_type,
            value: pop_values[i],
        });
    }

    for i in 0..push_values.len() {
        mem_ops.push(MemoryEvent {
            eid,
            emid: 1,
            mmid,
            offset: i as u64,
            ltype: LocationType::Stack,
            atype: AccessType::Write,
            vtype: outputs_type,
            value: push_values[i],
        });
    }

    mem_ops
}
