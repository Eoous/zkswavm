use wasmi::tracer::Tracer;

use crate::{
    memory::MemoryEvent,
    opcode::memory_event_of_step,
    spec::{
        evnet::Event,
        instruction::InstructionEntry,
    },
};

pub(crate) const VAR_COLUMNS: usize = 50;

#[derive(Default, Clone)]
pub struct CircuitBuilder {
    pub(crate) instruction_table: Vec<InstructionEntry>,
    pub(crate) event_table: Vec<Event>,
    pub(crate) memory_table: Vec<MemoryEvent>,
}

impl CircuitBuilder {
    pub fn from_tracer(tracer: &Tracer) -> Self {
        let instruction_table = tracer.itable.0.iter().map(|ientry| {
            InstructionEntry::from(ientry)
        }).collect();

        let event_table: Vec<Event> = tracer.etable.0.iter().map(|eentry| {
            Event::from(eentry)
        }).collect();

        let memory_table: Vec<Vec<MemoryEvent>> = event_table.iter().map(|event| {
            memory_event_of_step(event)
        }).collect();
        let memory_table = memory_table.into_iter().flat_map(|x| {
            x.into_iter()
        }).collect();

        CircuitBuilder {
            instruction_table,
            event_table,
            memory_table,
        }
    }
}

mod test {
    use halo2_proofs::arithmetic::FieldExt;

    use crate::test::test_circuit::TestCircuit;

    use super::*;

    impl CircuitBuilder {
        pub fn new_test_circuit<F: FieldExt>(&self) -> TestCircuit<F> {
            TestCircuit::new(&self)
        }
    }
}