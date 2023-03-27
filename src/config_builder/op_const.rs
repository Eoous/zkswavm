use halo2_proofs::{
    arithmetic::FieldExt,
    plonk::{Advice, Column, ConstraintSystem, Expression, VirtualCells},
};
use num_bigint::BigUint;
use std::marker::PhantomData;

use crate::{
    constant, constant_from, cur,
    event::{EventOpcodeConfig, EventOpcodeConfigBuilder},
    instruction::InstructionConfig,
    jump::JumpConfig,
    memory::MemoryConfig,
    opcode::Opcode,
    utils::bn_to_field,
};
use crate::event::EventCommonConfig;

pub struct ConstConfig<F: FieldExt> {
    vtype: Column<Advice>,
    value: Column<Advice>,
    enable: Column<Advice>,
    _mark: PhantomData<F>,
}

pub struct ConstConfigBuilder {}

impl<F: FieldExt> EventOpcodeConfigBuilder<F> for ConstConfigBuilder {
    fn configure(
        meta: &mut ConstraintSystem<F>,
        common: &EventCommonConfig,
        opcode_bit: Column<Advice>,
        cols: &mut impl Iterator<Item = Column<Advice>>,
        itable: &InstructionConfig<F>,
        mtable: &MemoryConfig<F>,
        jtable: &JumpConfig<F>,
    ) -> Box<dyn EventOpcodeConfig<F>> {
        let value = cols.next().unwrap();
        let vtype = cols.next().unwrap();

        mtable.configure_stack_write_in_table(
            "const mlookup",
            "const mlookup rev",
            meta,
            |meta| cur!(meta, opcode_bit),
            |meta| cur!(meta, common.eid),
            |meta| constant_from!(1u64),
            |meta| cur!(meta, common.sp),
            |meta| cur!(meta, vtype),
            |meta| cur!(meta, value),
        );

        Box::new(ConstConfig {
            enable: opcode_bit,
            value,
            vtype,
            _mark: PhantomData,
        })
    }
}

impl<F: FieldExt> EventOpcodeConfig<F> for ConstConfig<F> {
    fn opcode(&self, meta: &mut VirtualCells<'_, F>) -> Expression<F> {
        // FIXME
        (constant!(bn_to_field(
            &(BigUint::from(Opcode::Const as u64) << (64 + 13))
        )) + cur!(meta, self.vtype) * constant!(bn_to_field(&(BigUint::from(1u64) << (64 + 13))))
            + cur!(meta, self.value))
            * cur!(meta, self.enable)
    }

    fn sp_diff(&self, meta: &mut VirtualCells<'_, F>) -> Expression<F> {
        constant_from!(1u64) * cur!(meta, self.enable)
    }
}
