use std::marker::PhantomData;
use halo2_proofs::arithmetic::FieldExt;
use halo2_proofs::plonk::{Advice, Column, ConstraintSystem, Expression, VirtualCells};
use wasmi::tracer::etable::{EEntry, RunInstructionTraceStep};

use crate::instruction::{encode_inst_expr, Instruction, InstructionConfig};
use crate::{
    cur, pre, next, constant, constant_from
};
use crate::jump::JumpConfig;
use crate::memory::MemoryConfig;
use crate::config_builder::op_const::ConstConfigBuilder;

pub struct Event {
    pub(crate) eid: u64,
    pub(crate) sp: u64,
    pub(crate) last_jump_eid: u64,
    pub(crate) instruction: Instruction,
    pub(crate) step_info: RunInstructionTraceStep,
}

impl From<&EEntry> for Event {
    fn from(eentry: &EEntry) -> Self {
        Event {
            eid: eentry.id,
            sp: eentry.sp,
            last_jump_eid: 0,
            instruction: Instruction::from(&eentry.inst),
            step_info: eentry.step.clone(),
        }
    }
}

pub trait EventOpcodeConfigBuilder<F: FieldExt> {
    fn configure(
        meta: &mut ConstraintSystem<F>,
        common: &EventCommonConfig,
        opcode_bit: Column<Advice>,
        cols: &mut impl Iterator<Item = Column<Advice>>,
        itable: &InstructionConfig<F>,
        mtable: &MemoryConfig<F>,
        jtable: &JumpConfig<F>,
    ) -> Box<dyn EventOpcodeConfig<F>>;
}

pub trait EventOpcodeConfig<F: FieldExt> {
    fn opcode(&self, meta: &mut VirtualCells<'_, F>) -> Expression<F>;
    fn sp_diff(&self, meta: &mut VirtualCells<'_, F>) -> Expression<F>;
}

#[derive(Clone)]
pub struct EventCommonConfig {
    pub enable: Column<Advice>,
    pub eid: Column<Advice>,
    pub moid: Column<Advice>,
    pub fid: Column<Advice>,
    pub bid: Column<Advice>,
    pub iid: Column<Advice>,
    pub mmid: Column<Advice>,
    pub sp: Column<Advice>,
    pub opcode: Column<Advice>,
}

#[derive(Clone)]
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
        memory_table: &MemoryConfig<F>,
        jump_table: &JumpConfig<F>,
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
        let mut opcode_bitmaps_iter = opcode_bitmaps.iter();
        let mut configs: Vec<Box<dyn EventOpcodeConfig<F>>> = vec![];
        {
            let opcode_bit = opcode_bitmaps_iter.next().unwrap();
            let config = ConstConfigBuilder::configure(
                meta,
                &common_config,
                opcode_bit.clone(),
                cols,
                inst_config,
                memory_table,
                jump_table,
            );

            configs.push(config);
        }

        meta.create_gate("opcode consistent", |meta| {
            let mut acc = constant_from!(0u64);
            for config in configs.iter() {
                acc = acc + config.opcode(meta);
            }

            // advice.opcode - acc == 0
            vec![cur!(meta, opcode) - acc]
        });

        meta.create_gate("sp diff consistent", |meta| {
            let mut acc = constant_from!(0u64);
            for config in configs.iter() {
                acc = acc + config.sp_diff(meta);
            }

            // sp + sum(diff) - sp.next == 0
            vec![cur!(meta, sp) + acc - next!(meta, sp)]
        });

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

        inst_config.configure_in_table(meta, "instruction in table", |meta| {
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