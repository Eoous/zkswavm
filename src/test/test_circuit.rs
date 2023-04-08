use std::marker::PhantomData;

use halo2_proofs::{arithmetic::FieldExt, circuit::SimpleFloorPlanner, plonk::Circuit};

use crate::{
    builder::{CircuitBuilder, VAR_COLUMNS},
};
use crate::circuits::event::{EventChip, EventConfig};
use crate::circuits::instruction::{InstructionChip, InstructionConfig};
use crate::circuits::jump::JumpConfig;
use crate::circuits::memory::MemoryConfig;

#[derive(Clone)]
pub struct TestCircuitConfig<F: FieldExt> {
    etable: EventConfig<F>,
    itable: InstructionConfig<F>,
    jtable: JumpConfig<F>,
    mtable: MemoryConfig<F>,
}

#[derive(Default)]
pub struct TestCircuit<F: FieldExt> {
    circuit_builder: CircuitBuilder,
    _data: PhantomData<F>,
}

impl<F: FieldExt> TestCircuit<F> {
    pub fn new(builder: &CircuitBuilder) -> TestCircuit<F> {
        TestCircuit {
            circuit_builder: builder.clone(),
            _data: PhantomData,
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
        mut layouter: impl halo2_proofs::circuit::Layouter<F>,
    ) -> Result<(), halo2_proofs::plonk::Error> {
        let echip = EventChip::new(config.etable);
        let ichip = InstructionChip::new(config.itable);

        ichip.add_inst(&mut layouter, &self.circuit_builder.instruction_table)?;

        Ok(())
    }
}
