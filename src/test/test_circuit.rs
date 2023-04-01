use std::marker::PhantomData;
use halo2_proofs::{arithmetic::FieldExt, circuit::SimpleFloorPlanner, plonk::Circuit};

use crate::{
    event::EventConfig,
    instruction::InstructionConfig,
    jump::JumpConfig,
    memory::MemoryConfig,
    builder::VAR_COLUMNS,
};

#[derive(Clone)]
pub struct TestCircuitConfig<F: FieldExt> {
    etable: EventConfig<F>,
    itable: InstructionConfig<F>,
    jtable: JumpConfig<F>,
    mtable: MemoryConfig<F>,
}

#[derive(Default)]
pub struct TestCircuit<F: FieldExt> {
    _data: PhantomData<F>,
}

impl<F: FieldExt> TestCircuit<F> {
    pub fn new() -> TestCircuit<F> {
        TestCircuit {
            _data: PhantomData
        }
    }
}

impl<F: FieldExt> Circuit<F> for TestCircuit<F> {
    type Config = TestCircuitConfig<F>;

    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut halo2_proofs::plonk::ConstraintSystem<F>) -> Self::Config {
        let mut cols = [(); VAR_COLUMNS].map(|_| meta.advice_column()).into_iter();
        let itable = InstructionConfig::new(meta);
        let jtable = JumpConfig::new(&mut cols);
        let mtable = MemoryConfig::new(meta, &mut cols);
        let etable = EventConfig::new(meta, &mut cols, &itable, &mtable, &jtable);

        Self::Config {
            etable,
            itable,
            jtable,
            mtable,
        }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        layouter: impl halo2_proofs::circuit::Layouter<F>,
    ) -> Result<(), halo2_proofs::plonk::Error> {
        todo!()
    }
}
