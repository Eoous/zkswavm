use std::{cell::RefCell, rc::Rc};

use halo2_proofs::{arithmetic::FieldExt, dev::MockProver, plonk::Error};
use specs::{CompileTable, ExecutionTable};
use wasmi::{ModuleRef, NopExternals};

use crate::test::test_circuit::TestCircuit;

const K: u32 = 5;

pub fn run_test_circuit<F: FieldExt>(
    compile_table: CompileTable,
    execution_table: ExecutionTable,
) -> Result<(), Error> {
    let circuit = TestCircuit::<F>::new(compile_table, execution_table);

    MockProver::run(K, &circuit, vec![])?;

    Ok(())
}
