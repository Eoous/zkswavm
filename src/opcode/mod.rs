use wasmi::tracer::etable::RunInstructionTraceStep;
use crate::event::Event;
use crate::memory::{AccessType, LocationType, MemoryEvent, VarType};
use crate::opcode::stack_only::mem_op_from_stack_only_step;

pub(crate) mod stack_only;

pub fn memory_event_of_step(event: &Event) -> Vec<MemoryEvent> {
    let eid = event.eid;
    let mmid = event.instruction.mmid.into();

    match event.step_info {
        RunInstructionTraceStep::BrIfNez { value, dst_pc } => todo!(),
        RunInstructionTraceStep::Return { drop, keep } => todo!(),
        RunInstructionTraceStep::Call { index } => todo!(),
        RunInstructionTraceStep::GetLocal { depth, value } => {
            vec![
                MemoryEvent::new(
                    eid, mmid, depth.into(),
                    LocationType::Stack, AccessType::Read, VarType::I32,
                value.0,
                ),
                MemoryEvent::new(
                    eid, mmid.into(), event.sp - 1,
                    LocationType::Stack, AccessType::Write, VarType::I32,
                    value.0,
                )
            ]
        }
        RunInstructionTraceStep::I32Const { value } => {
            mem_op_from_stack_only_step::<0, 1>(eid, mmid, &[], &[value as u64])
        }
        RunInstructionTraceStep::I32BinOp { left, right, value } => {
            mem_op_from_stack_only_step::<2, 1>(eid, mmid, &[right as u64, left as u64], &[value as u64])
        }
        RunInstructionTraceStep::I32Comp { left, right, value } => {
            mem_op_from_stack_only_step::<2, 1>(eid, mmid, &[right as u64, left as u64], &[value as u64])
        }
    }
}