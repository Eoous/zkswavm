use wasmi::tracer::etable::RunInstructionTraceStep;
use crate::event::Event;
use crate::memory::{AccessType, LocationType, MemoryEvent, VarType};
use crate::opcode::stack_only::mem_op_from_stack_only_step;

pub(crate) mod stack_only;

pub fn memory_event_of_step(event: &Event) -> Vec<MemoryEvent> {
    let eid = event.eid;
    let mmid = event.instruction.mmid.into();

    match event.step_info {
        RunInstructionTraceStep::BrIfNez { value, dst_pc } => {
            mem_op_from_stack_only_step(eid, mmid, VarType::I32, VarType::I32, &[value as u64], &[])
        },
        RunInstructionTraceStep::Return { drop, keep} => {
            let drop_values: [u64;0] = [];
            let keep_values: [u64;0] = [];
            assert_eq!(drop as usize, drop_values.len());
            assert_eq!(keep as usize, keep_values.len());
            mem_op_from_stack_only_step(
                eid, mmid,
                VarType::I32, VarType::I32,
                drop_values.iter().map(|value|*value).collect::<Vec<_>>()[..].try_into().unwrap(),
                keep_values.iter().map(|value| *value).collect::<Vec<_>>()[..].try_into().unwrap(),
            )
        },
        RunInstructionTraceStep::Call { index } => {
            vec![]
        },
        RunInstructionTraceStep::GetLocal { depth, value } => {
            vec![
                MemoryEvent::new(
                    eid, mmid, depth.into(),
                    LocationType::Stack, AccessType::Read, VarType::I32,
                value.0,
                ),
                MemoryEvent::new(
                    eid, mmid.into(), 0,
                    LocationType::Stack, AccessType::Write, VarType::I32,
                    value.0,
                ),
            ]
        }
        RunInstructionTraceStep::I32Const { value } => {
            mem_op_from_stack_only_step(eid, mmid, VarType::I32, VarType::I32, &[], &[value as u64])
        }
        RunInstructionTraceStep::I32BinOp { left, right, value } => {
            mem_op_from_stack_only_step(eid, mmid, VarType::I32, VarType::I32, &[right as u64, left as u64], &[value as u64])
        }
        RunInstructionTraceStep::I32Comp { left, right, value } => {
            mem_op_from_stack_only_step(eid, mmid, VarType::I32, VarType::I32, &[right as u64, left as u64], &[value as u64])
        }
    }
}