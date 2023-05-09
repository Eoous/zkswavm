use halo2_proofs::arithmetic::FieldExt;
use halo2_proofs::circuit::Cell;
use halo2_proofs::plonk::{Advice, Column, ConstraintSystem, Error, Expression, VirtualCells};
use specs::etable::EventTableEntry;
use specs::itable::{Opcode, OpcodeClass};
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::circuits::config_builder::op_const::ConstConfigBuilder;
use crate::circuits::config_builder::op_drop::DropConfigBuilder;
use crate::circuits::config_builder::op_local_get::LocalGetConfigBuilder;
use crate::circuits::config_builder::op_return::ReturnConfigBuilder;
use crate::circuits::instruction::{encode_inst_expr, InstructionConfig};
use crate::circuits::jump::JumpConfig;
use crate::circuits::memory::MemoryConfig;
use crate::circuits::range::RangeConfig;
use crate::circuits::utils::{bn_to_field, Context};
use crate::{constant, constant_from, cur, next, pre};

pub trait EventOpcodeConfigBuilder<F: FieldExt> {
    fn configure(
        meta: &mut ConstraintSystem<F>,
        common: &EventCommonConfig,
        opcode_bit: Column<Advice>,
        cols: &mut impl Iterator<Item = Column<Advice>>,
        rtable: &RangeConfig<F>,
        itable: &InstructionConfig<F>,
        mtable: &MemoryConfig<F>,
        jtable: &JumpConfig<F>,
    ) -> Box<dyn EventOpcodeConfig<F>>;
}

pub trait EventOpcodeConfig<F: FieldExt> {
    fn opcode(&self, meta: &mut VirtualCells<'_, F>) -> Expression<F>;
    fn sp_diff(&self, meta: &mut VirtualCells<'_, F>) -> Expression<F>;
    fn assign(&self, ctx: &mut Context<'_, F>, entry: &EventTableEntry) -> Result<(), Error>;
    fn opcode_class(&self) -> OpcodeClass;
}

#[derive(Clone)]
pub struct EventCommonConfig {
    pub enable: Column<Advice>,
    pub rest_mops: Column<Advice>,
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
    common_config: EventCommonConfig,
    opcode_bitmaps: BTreeMap<OpcodeClass, Column<Advice>>,
    opcode_configs: BTreeMap<OpcodeClass, Rc<Box<dyn EventOpcodeConfig<F>>>>,
    _mark: PhantomData<F>,
}

impl<F: FieldExt> EventConfig<F> {
    pub fn configure(
        meta: &mut ConstraintSystem<F>,
        cols: &mut (impl Iterator<Item = Column<Advice>> + Clone),
        range_table: &RangeConfig<F>,
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
        let rest_mops = cols.next().unwrap();
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
            rest_mops,
        };

        let mut opcode_bitmaps_vec = vec![];
        let mut opcode_bitmaps = BTreeMap::new();
        let mut opcode_configs: BTreeMap<OpcodeClass, Rc<Box<dyn EventOpcodeConfig<F>>>> =
            BTreeMap::new();

        macro_rules! configure [
            ($($x:ident),*) => ({
                $($x{}; opcode_bitmaps_vec.push(cols.next().unwrap());)*

                let mut opcode_bitmaps_iter = opcode_bitmaps_vec.iter();
                $(
                    let opcode_bit = opcode_bitmaps_iter.next().unwrap();
                    let config = $x::configure(
                        meta,
                        &common_config,
                        opcode_bit.clone(),
                        &mut cols.clone(),
                        range_table,
                        inst_config,
                        memory_table,
                        jump_table,
                    );
                    opcode_bitmaps.insert(config.opcode_class(), opcode_bit.clone());
                    opcode_configs.insert(config.opcode_class(), Rc::new(config));
                )*
            })
        ];

        configure![
            ConstConfigBuilder,
            DropConfigBuilder,
            LocalGetConfigBuilder,
            ReturnConfigBuilder
        ];

        meta.create_gate("opcode consistent", |meta| {
            let mut acc = constant_from!(0u64);
            for (_, config) in opcode_configs.iter() {
                acc = acc + config.opcode(meta);
            }

            // advice.opcode - acc == 0
            vec![cur!(meta, opcode) - acc]
        });

        meta.create_gate("sp diff consistent", |meta| {
            let mut acc = constant_from!(0u64);
            for (_, config) in opcode_configs.iter() {
                acc = acc + config.sp_diff(meta);
            }

            // sp + sum(diff) - sp.next == 0
            vec![cur!(meta, sp) + acc - next!(meta, sp)]
        });

        for (_, bit) in opcode_bitmaps.iter() {
            meta.create_gate("opcode_bitmaps asssert bit", |meta| {
                // bit * (bit - 1)
                // bit == 0 || bit == 1
                vec![cur!(meta, *bit) * (cur!(meta, *bit) - constant_from!(1u64))]
            });
        }

        meta.create_gate("opcode_bitmaps pick one", |meta| {
            // sum(bits) - 1 == 0
            vec![
                opcode_bitmaps
                    .iter()
                    .map(|(_, x)| cur!(meta, *x))
                    .reduce(|acc, x| acc + x)
                    .unwrap()
                    - constant_from!(1u64),
            ]
        });

        meta.create_gate("eid increase", |meta| {
            // eid.cur - eid.pre - 1 == 0
            vec![
                cur!(meta, common_config.enable)
                    * (cur!(meta, common_config.eid)
                        - pre!(meta, common_config.eid)
                        - constant_from!(1u64)),
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
                    cur!(meta, common_config.opcode),
                )
        });

        meta.create_gate("rest_mops decrease", |meta| {
            let curr_mops = opcode_bitmaps
                .iter()
                .map(|(opcode_class, x)| cur!(meta, *x) * constant_from!(opcode_class.mops()))
                .reduce(|acc, x| acc + x)
                .unwrap();

            vec![
                cur!(meta, common_config.enable)
                    * (cur!(meta, common_config.rest_mops)
                        - next!(meta, common_config.rest_mops)
                        - curr_mops),
            ]
        });

        meta.create_gate("rest_mops is zero at end", |meta| {
            vec![
                (cur!(meta, common_config.enable) - constant_from!(1))
                    * cur!(meta, common_config.rest_mops),
            ]
        });

        meta.create_gate("enable is bit", |meta| {
            vec![
                (cur!(meta, common_config.enable) - constant_from!(1))
                    * cur!(meta, common_config.enable),
            ]
        });

        EventConfig {
            common_config,
            opcode_bitmaps,
            opcode_configs,
            _mark: PhantomData,
        }
    }
}

pub struct EventChip<F: FieldExt> {
    config: EventConfig<F>,
    _phantom: PhantomData<F>,
}

impl<F: FieldExt> EventChip<F> {
    pub fn new(config: EventConfig<F>) -> EventChip<F> {
        EventChip {
            config,
            _phantom: PhantomData,
        }
    }

    pub fn assign(
        &self,
        ctx: &mut Context<'_, F>,
        entries: &Vec<EventTableEntry>,
    ) -> Result<Cell, Error> {
        let mut rest_mops_cell = None;
        let mut rest_mops = entries
            .iter()
            .fold(0, |acc, entry| acc + entry.inst.opcode.mops());

        for (i, entry) in entries.into_iter().enumerate() {
            ctx.region.assign_advice(
                || "event enable",
                self.config.common_config.enable,
                ctx.offset,
                || Ok(F::one()),
            )?;

            macro_rules! assign {
                ($x: ident, $value: expr) => {
                    ctx.region.assign_advice(
                        || concat!("event ", stringify!($x)),
                        self.config.common_config.$x,
                        ctx.offset,
                        || Ok($value),
                    )?;
                };
            }

            macro_rules! assign_as_u64 {
                ($x: ident, $value: expr) => {
                    assign!($x, F::from($value as u64))
                };
            }

            assign_as_u64!(enable, 1u64);
            assign_as_u64!(eid, entry.eid);
            assign_as_u64!(moid, entry.inst.moid);
            assign_as_u64!(fid, entry.inst.fid);
            assign_as_u64!(bid, entry.inst.bid);
            assign_as_u64!(iid, entry.inst.iid);
            assign_as_u64!(mmid, entry.inst.mmid);
            assign_as_u64!(sp, entry.sp);
            assign!(opcode, bn_to_field(&(entry.inst.opcode.clone().into())));

            let opcode_class = entry.inst.opcode.clone().into();

            ctx.region.assign_advice(
                || concat!("event opcode"),
                self.config
                    .opcode_bitmaps
                    .get(&opcode_class)
                    .unwrap()
                    .clone(),
                ctx.offset,
                || Ok(F::one()),
            )?;

            self.config
                .opcode_configs
                .get(&opcode_class)
                .unwrap()
                .as_ref()
                .as_ref()
                .assign(ctx, entry)?;

            let cell = ctx.region.assign_advice(
                || concat!("event rest_mops"),
                self.config.common_config.rest_mops,
                ctx.offset,
                || Ok(rest_mops.into()),
            )?;

            if i == 0 {
                rest_mops_cell = Some(cell.cell());
            }

            rest_mops -= entry.inst.opcode.mops();

            ctx.next();
        }

        Ok(rest_mops_cell.unwrap())
    }
}
