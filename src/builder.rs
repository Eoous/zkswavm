use wasmi::tracer::Tracer;
use crate::event::Event;

use crate::instruction::Instruction;
use crate::memory::MemoryEvent;
use crate::opcode::memory_event_of_step;

pub struct CircuitBuilder {
    pub(crate) instruction_table: Vec<Instruction>,
    pub(crate) event_table: Vec<Event>,
    pub(crate) memory_table: Vec<MemoryEvent>,
}

impl CircuitBuilder {
    pub fn from_tracer(tracer: &Tracer) -> Self {
        let instruction_table = tracer.itable.0.iter().map(|ientry| {
            Instruction::from(ientry)
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