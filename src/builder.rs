use wasmi::tracer::Tracer;
use crate::event::Event;

use crate::instruction::Instruction;

pub struct CircuitBuilder {
    pub instruction_table: Vec<Instruction>,
    pub event_table: Vec<Event>,
}

impl CircuitBuilder {
    pub fn from_tracer(tracer: Tracer) -> Self {
        CircuitBuilder {
            instruction_table:
                tracer.itable.0
                    .into_iter()
                    .map(|ientry| Instruction::from(ientry))
                    .collect(),
            event_table:
                tracer.etable.0
                    .into_iter()
                    .map(|eentry| Event::from(eentry))
                    .collect(),
        }
    }
}