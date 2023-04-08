use std::marker::PhantomData;

use halo2_proofs::{
    arithmetic::FieldExt,
    plonk::{Advice, Column, ConstraintSystem, Expression, VirtualCells},
};
use num_bigint::BigUint;

use crate::{
    constant, constant_from, cur,
    spec::instruction::OpcodeClass,
    utils::bn_to_field
};
use crate::circuits::event::{EventOpcodeConfig, EventOpcodeConfigBuilder};
use crate::circuits::event::EventCommonConfig;
use crate::circuits::instruction::InstructionConfig;
use crate::circuits::jump::JumpConfig;
use crate::circuits::memory::MemoryConfig;

pub struct LocalGetConfig<F: FieldExt> {
    offset: Column<Advice>,
    vtype: Column<Advice>,
    value: Column<Advice>,
    _mark: PhantomData<F>,
}

pub struct LocalGetConfigBuilder {}

impl<F: FieldExt> EventOpcodeConfigBuilder<F> for LocalGetConfigBuilder {
    fn configure(
        meta: &mut ConstraintSystem<F>,
        common: &EventCommonConfig,
        opcode_bit: Column<Advice>,
        cols: &mut impl Iterator<Item = Column<Advice>>,
        instruction_table: &InstructionConfig<F>,
        memory_table: &MemoryConfig<F>,
        jump_table: &JumpConfig<F>,
    ) -> Box<dyn EventOpcodeConfig<F>> {
        let offset = cols.next().unwrap();
        let value = cols.next().unwrap();
        let vtype = cols.next().unwrap();

        memory_table.configure_stack_read_in_table(
            "local get mlookup",
            "local get mlookup rev",
            meta,
            |meta| cur!(meta, opcode_bit),
            |meta| cur!(meta, common.eid),
            |meta| constant_from!(1u64),
            |meta| cur!(meta, common.sp),
            |meta| cur!(meta, vtype),
            |meta| cur!(meta, value),
        );

        memory_table.configure_stack_write_in_table(
            "local get mlookup",
            "local get mlookup rev",
            meta,
            |meta| cur!(meta, opcode_bit),
            |meta| cur!(meta, common.eid),
            |meta| constant_from!(2u64),
            |meta| cur!(meta, common.sp),
            |meta| cur!(meta, vtype),
            |meta| cur!(meta, value),
        );

        Box::new(LocalGetConfig {
            offset,
            value,
            vtype,
            _mark: PhantomData,
        })
    }
}

impl<F: FieldExt> EventOpcodeConfig<F> for LocalGetConfig<F> {
    fn opcode(&self, meta: &mut VirtualCells<'_, F>) -> Expression<F> {
        // (1 << 64) + offset
        constant!(bn_to_field(&(BigUint::from(OpcodeClass::LocalGet as u64) << 64)))
            + cur!(meta, self.offset)
    }

    fn sp_diff(&self, meta: &mut VirtualCells<'_, F>) -> Expression<F> {
        constant_from!(1u64)
    }
}
