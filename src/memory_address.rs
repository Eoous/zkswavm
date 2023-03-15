use halo2_proofs::arithmetic::FieldExt;
use halo2_proofs::plonk::{Advice, Column, ConstraintSystem};

enum MemTrace {
    ModuleTrace(u64, u64, u64),
    LocalTrace(u64, u64, u64),
}

const MEM_TRACE_COLUMNS: usize = 4;

#[derive(Clone, Debug)]
struct MemoryAddressConfig {
    columns: [Column<Advice>; MEM_TRACE_COLUMNS],
}

impl MemoryAddressConfig {
    pub fn configure<F: FieldExt>(&mut self, meta: &mut ConstraintSystem<F>) {

    }
}