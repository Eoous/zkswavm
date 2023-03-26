use std::marker::PhantomData;
use halo2_proofs::arithmetic::FieldExt;
use halo2_proofs::plonk::{Advice, Column, ConstraintSystem, Expression};
use wasmi::tracer::etable::{EEntry, RunInstructionTraceStep};

use crate::instruction::{encode_inst_expr, Instruction, InstructionConfig};
use crate::{
    cur, pre, constant, constant_from
};

pub struct Event {
    pub(crate) eid: u64,
    pub(crate) sp: u64,
    pub(crate) last_jump_eid: u64,
    pub(crate) instruction: Instruction,
    pub(crate) step_info: RunInstructionTraceStep,
}

impl From<EEntry> for Event {
    fn from(eentry:EEntry ) -> Self {
        Event {
            eid: eentry.id,
            sp: eentry.sp,
            last_jump_eid: 0,
            instruction: Instruction::from(eentry.inst),
            step_info: eentry.step,
        }
    }
}

pub trait EventOpcodeConfigBuilder<F: FieldExt> {
    fn configure(
        meta: &mut ConstraintSystem<F>,
        common: &EventCommonConfig,
        opcode_bit: Column<Advice>,
        cols: &mut impl Iterator<Item = Column<Advice>>
    ) -> Self;
}

pub trait EventOpcodeConfig<F: FieldExt> {
    fn opcode(&self) -> Expression<F>;
}

pub struct EventCommonConfig {
    enable: Column<Advice>,
    eid: Column<Advice>,
    moid: Column<Advice>,
    fid: Column<Advice>,
    bid: Column<Advice>,
    iid: Column<Advice>,
    mmid: Column<Advice>,
    sp: Column<Advice>,
    opcode: Column<Advice>,
}

pub struct EventConfig<F: FieldExt> {
    opcode_bitmaps: Vec<Column<Advice>>,
    common_config: EventCommonConfig,
    _mark: PhantomData<F>,
}

impl<F: FieldExt> EventConfig<F> {
    pub fn new<'a>(
        meta: &mut ConstraintSystem<F>,
        cols: &mut impl Iterator<Item = Column<Advice>>,
        inst_config: &InstructionConfig<F>,
    ) -> EventConfig<F> {
        let enable = cols.next().unwrap();
        let eid = cols.next().unwrap();
        let moid = cols.next().unwrap();
        let fid = cols.next().unwrap();
        let bid = cols.next().unwrap();
        let iid = cols.next().unwrap();
        let mmid = cols.next().unwrap();
        let sp = cols.next().unwrap();
        let opcode = cols.next().unwrap();
        let common_config = EventCommonConfig {
            enable,
            eid,
            moid,
            fid,
            bid,
            iid,
            mmid,
            sp,
            opcode,
        };

        let mut opcode_bitmaps: Vec<Column<Advice>> = vec![];
        for bit in opcode_bitmaps.iter() {
            meta.create_gate("opcode_bitmaps asssert bit", |meta| {
                // bit * (bit - 1)
                // bit == 0 || bit == 1
                vec![cur!(meta, bit.clone()) * (cur!(meta, bit.clone()) - constant_from!(1u64))]
            });
        }

        meta.create_gate("opcode_bitmaps pick one", |meta| {
            // sum(bits) - 1 == 0
            vec![
                opcode_bitmaps
                    .iter()
                    .map(|x| cur!(meta, *x))
                    .reduce(|acc, x| acc + x).unwrap()
                    - constant_from!(1u64)
            ]
        });

        meta.create_gate("eid increase", |meta| {
            // eid.cur - eid.pre - 1 == 0
            vec![
                cur!(meta, common_config.enable)
                    * (cur!(meta, common_config.eid) - pre!(meta, common_config.eid) - constant_from!(1u64))
            ]
        });

        inst_config.configure_in_table(meta, |meta| {
            cur!(meta, enable)
                * encode_inst_expr(
                cur!(meta, common_config.moid),
                cur!(meta, common_config.mmid),
                cur!(meta, common_config.fid),
                cur!(meta, common_config.bid),
                cur!(meta, common_config.iid),
                cur!(meta, common_config.opcode)
            )
        });

        EventConfig {
            opcode_bitmaps,
            common_config,
            _mark: PhantomData,
        }
    }
}

pub struct EventChip<F: FieldExt> {
    config: EventConfig<F>,
    _phantom: PhantomData<F>
}

impl<F: FieldExt> EventChip<F> {
    pub fn new(config: EventConfig<F>) -> EventChip<F> {
        EventChip {
            config,
            _phantom: PhantomData,
        }
    }
}