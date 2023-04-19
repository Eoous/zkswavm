use std::marker::PhantomData;

use halo2_proofs::{arithmetic::FieldExt, circuit::SimpleFloorPlanner, plonk::Circuit};
use specs::itable::OpcodeClass::Const;
use specs::{CompileTable, ExecutionTable};

use crate::circuits::event::{EventChip, EventConfig};
use crate::circuits::instruction::{InstructionChip, InstructionConfig};
use crate::circuits::jump::JumpConfig;
use crate::circuits::memory::{MemoryChip, MemoryConfig};
use crate::circuits::memory_init::InitMemoryConfig;
use crate::circuits::range::{RangeChip, RangeConfig};
use crate::circuits::utils::Context;

const VAR_COLUMNS: usize = 50;

#[derive(Clone)]
pub struct TestCircuitConfig<F: FieldExt> {
    rtable: RangeConfig<F>,
    imtable: InitMemoryConfig<F>,
    itable: InstructionConfig<F>,
    etable: EventConfig<F>,
    jtable: JumpConfig<F>,
    mtable: MemoryConfig<F>,
}

#[derive(Default)]
pub struct TestCircuit<F: FieldExt> {
    compile_tables: CompileTable,
    execution_tables: ExecutionTable,
    _data: PhantomData<F>,
}

impl<F: FieldExt> TestCircuit<F> {
    pub fn new(c: CompileTable, e: ExecutionTable) -> TestCircuit<F> {
        TestCircuit {
            compile_tables: c,
            execution_tables: e,
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
        let rtable = RangeConfig::configure([meta.lookup_table_column()]);
        let imtable = InitMemoryConfig::configure(meta.lookup_table_column());
        let itable = InstructionConfig::configure(meta.lookup_table_column());
        let jtable = JumpConfig::configure(&mut cols);
        let mtable = MemoryConfig::configure(meta, &mut cols, &rtable, &imtable);
        let etable = EventConfig::configure(meta, &mut cols, &itable, &mtable, &jtable);

        Self::Config {
            rtable,
            imtable,
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
        let rchip = RangeChip::new(config.rtable);
        let ichip = InstructionChip::new(config.itable);
        let mchip = MemoryChip::new(config.mtable);

        rchip.init(&mut layouter, 16usize)?;
        ichip.assign(&mut layouter, &self.compile_tables.instructions)?;

        layouter.assign_region(
            || "memory",
            |region| {
                let mut ctx = Context::new(region);
                mchip.assign(&mut ctx, &self.execution_tables.memory)?;
                Ok(())
            },
        )?;

        Ok(())
    }
}
