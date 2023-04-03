use std::marker::PhantomData;
use halo2_proofs::arithmetic::FieldExt;
use halo2_proofs::plonk::{Advice, Column, ConstraintSystem, Expression, VirtualCells};
use num_bigint::BigUint;

use crate::event::{EventCommonConfig, EventOpcodeConfig, EventOpcodeConfigBuilder};
use crate::instruction::InstructionConfig;
use crate::jump::JumpConfig;
use crate::memory::MemoryConfig;
use crate::opcode::Opcode;
use crate::utils::bn_to_field;
use crate::{cur, constant};

pub struct DropConfig<F: FieldExt> {
    enable: Column<Advice>,
    _mark: PhantomData<F>,
}

pub struct DropConfigBuilder {}

impl<F: FieldExt> EventOpcodeConfigBuilder<F> for DropConfigBuilder {
    fn configure(
        meta: &mut ConstraintSystem<F>,
        common: &EventCommonConfig,
        opcode_bit: Column<Advice>,
        cols: &mut impl Iterator<Item=Column<Advice>>,
        itable: &InstructionConfig<F>,
        mtable: &MemoryConfig<F>,
        jtable: &JumpConfig<F>
    ) -> Box<dyn EventOpcodeConfig<F>> {
        Box::new(DropConfig {
            enable: opcode_bit,
            _mark: PhantomData,
        })
    }
}

impl<F: FieldExt> EventOpcodeConfig<F> for DropConfig<F> {
    fn opcode(&self, meta: &mut VirtualCells<'_, F>) -> Expression<F> {
        constant!(bn_to_field(
            &(BigUint::from(Opcode::Drop as u64) << (64 + 13))
        )) * cur!(meta, self.enable)
    }

    fn sp_diff(&self, meta: &mut VirtualCells<'_, F>) -> Expression<F> {
        constant!(-F::one()) * cur!(meta, self.enable)
    }
}