use crate::circuits::event::{EventCommonConfig, EventOpcodeConfig, EventOpcodeConfigBuilder};
use crate::circuits::instruction::InstructionConfig;
use crate::circuits::jump::JumpConfig;
use crate::circuits::memory::MemoryConfig;
use crate::circuits::range::RangeConfig;
use crate::circuits::utils::bn_to_field;
use crate::circuits::utils::tvalue::TValueConfig;
use crate::circuits::utils::Context;
use crate::{constant, constant_from, cur};
use halo2_proofs::arithmetic::FieldExt;
use halo2_proofs::plonk::{Advice, Column, ConstraintSystem, Error, Expression, VirtualCells};
use num_bigint::BigUint;
use specs::etable::EventTableEntry;
use specs::itable::OpcodeClass;
use specs::itable::OPCODE_ARG0_SHIFT;
use specs::itable::OPCODE_CLASS_SHIFT;

pub struct ReturnConfig<F: FieldExt> {
    drop: Column<Advice>,
    keep: Column<Advice>,
    tvalue: TValueConfig<F>,
}

pub struct ReturnConfigBuilder {}

impl<F: FieldExt> EventOpcodeConfigBuilder<F> for ReturnConfigBuilder {
    fn configure(
        meta: &mut ConstraintSystem<F>,
        common: &EventCommonConfig,
        opcode_bit: Column<Advice>,
        cols: &mut impl Iterator<Item = Column<Advice>>,
        rtable: &RangeConfig<F>,
        itable: &InstructionConfig<F>,
        mtable: &MemoryConfig<F>,
        jtable: &JumpConfig<F>,
    ) -> Box<dyn EventOpcodeConfig<F>> {
        let drop = cols.next().unwrap();
        let keep = cols.next().unwrap();
        let tvalue = TValueConfig::configure(meta, cols, rtable, |meta| cur!(meta, opcode_bit));

        meta.create_gate("keep is bit", |meta| {
            vec![cur!(meta, keep) * (cur!(meta, keep) - constant_from!(1))]
        });

        rtable.configure_in_common_range(meta, "return drop range", |meta| {
            cur!(meta, opcode_bit) * cur!(meta, drop)
        });

        mtable.configure_stack_read_in_table(
            "return mlookup #1",
            meta,
            |meta| cur!(meta, opcode_bit) * cur!(meta, keep),
            |meta| cur!(meta, common.eid),
            |_meta| constant_from!(1u64),
            |meta| cur!(meta, common.sp),
            |meta| cur!(meta, tvalue.vtype),
            |meta| cur!(meta, tvalue.value.value),
        );

        mtable.configure_stack_write_in_table(
            "return mlookup #2",
            meta,
            |meta| cur!(meta, opcode_bit) * cur!(meta, keep),
            |meta| cur!(meta, common.eid),
            |meta| constant_from!(2u64),
            |meta| cur!(meta, common.sp) - cur!(meta, drop),
            |meta| cur!(meta, tvalue.vtype),
            |meta| cur!(meta, tvalue.value.value),
        );

        Box::new(ReturnConfig { drop, keep, tvalue })
    }
}

impl<F: FieldExt> EventOpcodeConfig<F> for ReturnConfig<F> {
    fn opcode(&self, meta: &mut VirtualCells<'_, F>) -> Expression<F> {
        constant!(bn_to_field(
            &(BigUint::from(OpcodeClass::Return as u64) << OPCODE_CLASS_SHIFT)
        )) + cur!(meta, self.drop)
            * constant!(bn_to_field(&(BigUint::from(1u64) << OPCODE_ARG0_SHIFT)))
            + cur!(meta, self.keep)
    }

    fn sp_diff(&self, meta: &mut VirtualCells<'_, F>) -> Expression<F> {
        cur!(meta, self.keep) - cur!(meta, self.drop)
    }

    fn assign(&self, ctx: &mut Context<'_, F>, entry: &EventTableEntry) -> Result<(), Error> {
        match &entry.step_info {
            specs::step::StepInfo::Return {
                drop,
                keep,
                keep_values,
                ..
            } => {
                todo!();
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    fn opcode_class(&self) -> OpcodeClass {
        OpcodeClass::Return
    }
}
