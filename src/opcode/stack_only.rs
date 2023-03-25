use crate::memory::{AccessType, LocationType, MemoryEvent, VarType};

pub(crate)
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
        mem_ops.push(MemoryEvent::new(
            eid,
            mmid,
            i as u64,
            LocationType::Stack,
            AccessType::Read,
            inputs_type,
            pop_values[i]
        ));
    }

    for i in 0..push_values.len() {
        mem_ops.push(MemoryEvent::new(
            eid,
            mmid,
            i as u64,
            LocationType::Stack,
            AccessType::Write,
            outputs_type,
            push_values[i],
        ));
    }

    mem_ops
}