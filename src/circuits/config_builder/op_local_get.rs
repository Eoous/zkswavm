use std::marker::PhantomData;

use crate::circuits::event::EventCommonConfig;
use crate::circuits::event::{EventOpcodeConfig, EventOpcodeConfigBuilder};
use crate::circuits::instruction::InstructionConfig;
use crate::circuits::jump::JumpConfig;
use crate::circuits::memory::MemoryConfig;
use crate::circuits::utils::{bn_to_field, Context};
use crate::{constant, constant_from, cur};

use crate::circuits::range::RangeConfig;
use crate::circuits::utils::tvalue::TValueConfig;
use halo2_proofs::plonk::Error;
use halo2_proofs::{
    arithmetic::FieldExt,
    plonk::{Advice, Column, ConstraintSystem, Expression, VirtualCells},
};
use num_bigint::BigUint;
use specs::etable::EventTableEntry;
use specs::itable::OpcodeClass;
use specs::step::StepInfo;

pub struct LocalGetConfig<F: FieldExt> {
    offset: Column<Advice>,
    tvalue: TValueConfig<F>,
    _mark: PhantomData<F>,
}

pub struct LocalGetConfigBuilder {}

impl<F: FieldExt> EventOpcodeConfigBuilder<F> for LocalGetConfigBuilder {
    fn configure(
        meta: &mut ConstraintSystem<F>,
        common: &EventCommonConfig,
        opcode_bit: Column<Advice>,
        cols: &mut impl Iterator<Item = Column<Advice>>,
        range_table: &RangeConfig<F>,
        instruction_table: &InstructionConfig<F>,
        memory_table: &MemoryConfig<F>,
        jump_table: &JumpConfig<F>,
    ) -> Box<dyn EventOpcodeConfig<F>> {
        let offset = cols.next().unwrap();
        let tvalue =
            TValueConfig::configure(meta, cols, range_table, |meta| cur!(meta, opcode_bit));

        memory_table.configure_stack_read_in_table(
            "local get mlookup",
            meta,
            |meta| cur!(meta, opcode_bit),
            |meta| cur!(meta, common.eid),
            |meta| constant_from!(1u64),
            |meta| cur!(meta, common.sp),
            |meta| cur!(meta, tvalue.vtype),
            |meta| cur!(meta, tvalue.value.value),
        );

        memory_table.configure_stack_write_in_table(
            "local get mlookup",
            meta,
            |meta| cur!(meta, opcode_bit),
            |meta| cur!(meta, common.eid),
            |meta| constant_from!(2u64),
            |meta| cur!(meta, common.sp),
            |meta| cur!(meta, tvalue.vtype),
            |meta| cur!(meta, tvalue.value.value),
        );

        Box::new(LocalGetConfig {
            offset,
            tvalue,
            _mark: PhantomData,
        })
    }
}

impl<F: FieldExt> EventOpcodeConfig<F> for LocalGetConfig<F> {
    fn opcode(&self, meta: &mut VirtualCells<'_, F>) -> Expression<F> {
        // (1 << 64) + offset
        constant!(bn_to_field(
            &(BigUint::from(OpcodeClass::LocalGet as u64) << 64)
        )) + cur!(meta, self.offset)
    }

    fn sp_diff(&self, meta: &mut VirtualCells<'_, F>) -> Expression<F> {
        constant_from!(1u64)
    }

    fn assign(&self, ctx: &mut Context<'_, F>, entry: &EventTableEntry) -> Result<(), Error> {
        match entry.step_info {
            StepInfo::GetLocal {
                depth,
                vtype,
                value,
            } => {
                ctx.region.assign_advice(
                    || "op_const offset",
                    self.offset,
                    ctx.offset,
                    || Ok(F::from(depth as u64)),
                )?;

                self.tvalue.assign(ctx, vtype, value)?;
            }
            _ => unreachable!(),
        }

        Ok(())
    }

    fn opcode_class(&self) -> OpcodeClass {
        OpcodeClass::LocalGet
    }
}
