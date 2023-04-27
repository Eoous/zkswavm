use halo2_proofs::arithmetic::FieldExt;
use halo2_proofs::circuit::Cell;
use halo2_proofs::plonk::{Advice, Column, ConstraintSystem, Error, Expression, VirtualCells};
use halo2_proofs::poly::Rotation;
use lazy_static::lazy_static;
use num_bigint::BigUint;
use specs::mtable::{AccessType, LocationType, MemoryTableEntry, VarType};
use std::marker::PhantomData;

use crate::circuits::memory_init::InitMemoryConfig;
use crate::circuits::range::RangeConfig;
use crate::circuits::utils::row_diff::RowDiffConfig;
use crate::circuits::utils::{bn_to_field, Context};
use crate::{constant, constant_from, cur, next, pre};

lazy_static! {
    static ref VAR_TYPE_SHIFT: BigUint = BigUint::from(1u64) << 64;
    static ref ACCESS_TYPE_SHIFT: BigUint = BigUint::from(1u64) << 77;
    static ref LOC_TYPE_SHIFT: BigUint = BigUint::from(1u64) << 79;
    static ref OFFSET_SHIFT: BigUint = BigUint::from(1u64) << 80;
    static ref MMID_SHIFT: BigUint = BigUint::from(1u64) << 96;
    static ref EMID_SHIFT: BigUint = BigUint::from(1u64) << 112;
    static ref EID_SHIFT: BigUint = BigUint::from(1u64) << 128;
}

#[derive(Clone)]
pub struct MemoryConfig<F: FieldExt> {
    eid: RowDiffConfig<F>,
    emid: RowDiffConfig<F>,
    mmid: RowDiffConfig<F>,
    offset: RowDiffConfig<F>,
    ltype: RowDiffConfig<F>,

    atype: Column<Advice>,
    vtype: Column<Advice>,
    value: Column<Advice>,

    same_location: Column<Advice>,
    enable: Column<Advice>,
    rest_mops: Column<Advice>,

    _mark: PhantomData<F>,
}

impl<F: FieldExt> MemoryConfig<F> {
    /// RowDiffConfig needs 3 cols. 3 * 5 + 5 = 20
    ///
    /// Now MemoryConfig needs 20 cols.
    pub fn new(
        meta: &mut ConstraintSystem<F>,
        cols: &mut impl Iterator<Item = Column<Advice>>,
    ) -> MemoryConfig<F> {
        let emid = RowDiffConfig::configure("mtable emid", meta, cols, |_| constant_from!(1));
        let ltype = RowDiffConfig::configure("mtable ltype", meta, cols, |_| constant_from!(1));
        let mmid = RowDiffConfig::configure("mtable mmid", meta, cols, |_| constant_from!(1));
        let offset = RowDiffConfig::configure("mtable offset", meta, cols, |_| constant_from!(1));
        let eid = RowDiffConfig::configure("mtable eid", meta, cols, |_| constant_from!(1));

        let value = cols.next().unwrap();
        let atype = cols.next().unwrap();
        let vtype = cols.next().unwrap();
        let enable = cols.next().unwrap();
        let same_location = cols.next().unwrap();
        let rest_mops = cols.next().unwrap();

        MemoryConfig {
            ltype,
            mmid,
            offset,
            eid,
            emid,
            atype,
            vtype,
            value,
            enable,
            same_location,
            rest_mops,
            _mark: PhantomData,
        }
    }

    fn encode_for_lookup(&self, meta: &mut VirtualCells<'_, F>) -> Expression<F> {
        self.eid.data(meta) * constant!(bn_to_field(&EID_SHIFT))
            + self.emid.data(meta) * constant!(bn_to_field(&EMID_SHIFT))
            + self.mmid.data(meta) * constant!(bn_to_field(&MMID_SHIFT))
            + self.offset.data(meta) * constant!(bn_to_field(&OFFSET_SHIFT))
            + self.ltype.data(meta) * constant!(bn_to_field(&LOC_TYPE_SHIFT))
            + cur!(meta, self.atype) * constant!(bn_to_field(&ACCESS_TYPE_SHIFT))
            + cur!(meta, self.vtype) * constant!(bn_to_field(&VAR_TYPE_SHIFT))
            + cur!(meta, self.value)
    }

    pub fn configure_stack_read_in_table(
        &self,
        key: &'static str,
        meta: &mut ConstraintSystem<F>,
        enable: impl Fn(&mut VirtualCells<'_, F>) -> Expression<F>,
        eid: impl Fn(&mut VirtualCells<'_, F>) -> Expression<F>,
        emid: impl Fn(&mut VirtualCells<'_, F>) -> Expression<F>,
        sp: impl Fn(&mut VirtualCells<'_, F>) -> Expression<F>,
        vtype: impl Fn(&mut VirtualCells<'_, F>) -> Expression<F>,
        value: impl Fn(&mut VirtualCells<'_, F>) -> Expression<F>,
    ) {
        meta.lookup_any(key, |meta| {
            vec![(
                (eid(meta) * constant!(bn_to_field(&EID_SHIFT))
                    + emid(meta) * constant!(bn_to_field(&EMID_SHIFT))
                    + sp(meta) * constant!(bn_to_field(&OFFSET_SHIFT))
                    + constant!(bn_to_field(&LOC_TYPE_SHIFT))
                        * constant_from!(LocationType::Stack)
                    + constant!(bn_to_field(&ACCESS_TYPE_SHIFT))
                        * constant_from!(AccessType::Write)
                    + vtype(meta) * constant!(bn_to_field(&VAR_TYPE_SHIFT))
                    + value(meta))
                    * enable(meta),
                self.encode_for_lookup(meta) * enable(meta),
            )]
        });
    }

    pub fn configure_stack_write_in_table(
        &self,
        key: &'static str,
        meta: &mut ConstraintSystem<F>,
        enable: impl Fn(&mut VirtualCells<'_, F>) -> Expression<F>,
        eid: impl Fn(&mut VirtualCells<'_, F>) -> Expression<F>,
        emid: impl Fn(&mut VirtualCells<'_, F>) -> Expression<F>,
        sp: impl Fn(&mut VirtualCells<'_, F>) -> Expression<F>,
        vtype: impl Fn(&mut VirtualCells<'_, F>) -> Expression<F>,
        value: impl Fn(&mut VirtualCells<'_, F>) -> Expression<F>,
    ) {
        meta.lookup_any(key, |meta| {
            vec![(
                (eid(meta) * constant!(bn_to_field(&EID_SHIFT))
                    + emid(meta) * constant!(bn_to_field(&EMID_SHIFT))
                    + sp(meta) * constant!(bn_to_field(&OFFSET_SHIFT))
                    + constant!(bn_to_field(&LOC_TYPE_SHIFT))
                        * constant_from!(LocationType::Stack)
                    + constant!(bn_to_field(&ACCESS_TYPE_SHIFT))
                        * constant_from!(AccessType::Write)
                    + vtype(meta) * constant!(bn_to_field(&VAR_TYPE_SHIFT))
                    + value(meta))
                    * enable(meta),
                self.encode_for_lookup(meta) * enable(meta),
            )]
        });
    }

    pub fn configure(
        meta: &mut ConstraintSystem<F>,
        cols: &mut impl Iterator<Item = Column<Advice>>,
        range: &RangeConfig<F>,
        memory_init: &InitMemoryConfig<F>,
    ) -> MemoryConfig<F> {
        let memory = Self::new(meta, cols);

        memory.configure_enable(meta);
        memory.configure_sort(meta, range);
        memory.configure_stack_or_heap(meta);
        memory.configure_range(meta, range);
        memory.configure_same_location(meta);
        memory.configure_rule(meta, memory_init);

        memory
    }

    fn configure_enable(&self, meta: &mut ConstraintSystem<F>) -> &MemoryConfig<F> {
        meta.create_gate("enable seq", |meta| {
            let cur = cur!(meta, self.enable);
            let next = next!(meta, self.enable);

            // next * (cur - 1) == 0
            // cur  * (cur - 1) == 0
            vec![
                next * (cur.clone() - Expression::Constant(F::one())),
                cur.clone() * (cur.clone() - Expression::Constant(F::one())),
            ]
        });

        self
    }

    fn configure_same_location(&self, meta: &mut ConstraintSystem<F>) -> &MemoryConfig<F> {
        meta.create_gate("is same location", |meta| {
            let same_location = cur!(meta, self.same_location);

            vec![
                self.ltype.is_same(meta) * self.mmid.is_same(meta) * self.offset.is_same(meta)
                    - same_location,
            ]
        });

        self
    }

    fn configure_stack_or_heap(&self, meta: &mut ConstraintSystem<F>) -> &MemoryConfig<F> {
        meta.create_gate("stack_or_heap", |meta| {
            let ltype = self.ltype.data(meta);

            vec![ltype.clone() * (ltype - Expression::Constant(F::one()))]
        });

        self
    }

    fn configure_range(
        &self,
        meta: &mut ConstraintSystem<F>,
        range: &RangeConfig<F>,
    ) -> &MemoryConfig<F> {
        range.configure_in_common_range(meta, "mmid in range", |meta| self.mmid.data(meta));
        range.configure_in_common_range(meta, "offset in range", |meta| self.offset.data(meta));
        range.configure_in_common_range(meta, "eid in range", |meta| self.eid.data(meta));

        range.configure_in_common_range(meta, "emid in range", |meta| self.emid.data(meta));
        range.configure_in_common_range(meta, "vtype in range", |meta| self.emid.data(meta));

        self
    }

    fn configure_sort(
        &self,
        meta: &mut ConstraintSystem<F>,
        range: &RangeConfig<F>,
    ) -> &MemoryConfig<F> {
        range.configure_in_common_range(meta, "ltype sort", |meta| {
            self.is_enable(meta) * self.ltype.diff(meta)
        });
        range.configure_in_common_range(meta, "mmid sort", |meta| {
            self.is_enable(meta) * self.ltype.is_same(meta) * self.mmid.diff(meta)
        });
        range.configure_in_common_range(meta, "offset sort", |meta| {
            self.is_enable(meta)
                * self.ltype.is_same(meta)
                * self.mmid.is_same(meta)
                * self.offset.is_same(meta)
        });
        range.configure_in_common_range(meta, "eid sort", |meta| {
            self.is_enable(meta) * self.is_same_location(meta) * self.eid.diff(meta)
        });
        range.configure_in_common_range(meta, "emid sort", |meta| {
            self.is_enable(meta)
                * self.is_same_location(meta)
                * self.eid.is_same(meta)
                * self.emid.diff(meta)
        });

        self
    }

    fn configure_rule(
        &self,
        meta: &mut ConstraintSystem<F>,
        memory_init: &InitMemoryConfig<F>,
    ) -> &MemoryConfig<F> {
        meta.create_gate("memory read after write", |meta| {
            vec![
                self.is_enable(meta) * self.is_read_not_bit(meta) * self.diff(meta, self.value),
                self.is_enable(meta) * self.is_read_not_bit(meta) * self.diff(meta, self.vtype),
            ]
        });

        meta.create_gate("memory emid unique", |meta| {
            vec![self.is_enable(meta) * self.is_same_location(meta) * self.emid.is_same(meta)]
        });

        meta.create_gate("memory stack first line", |meta| {
            vec![
                self.is_enable(meta)
                    * (self.is_same_location(meta) - Expression::Constant(F::one()))
                    * self.is_stack(meta)
                    * (cur!(meta, self.atype) - constant_from!(AccessType::Write)),
            ]
        });

        // first line in heap
        memory_init.configure_in_table(meta, "memory heap first line", |meta| {
            self.is_enable(meta)
                * (Expression::Constant(F::one()) - self.is_same_location(meta))
                * self.is_heap(meta)
                * memory_init.encode(
                    self.mmid.data(meta),
                    self.offset.data(meta),
                    cur!(meta, self.value),
                )
        });

        memory_init.configure_in_table(meta, "rest mops decrease", |meta| {
            self.is_enable(meta)
                * self.is_not_init(meta)
                * (cur!(meta, self.rest_mops) - next!(meta, self.rest_mops) - constant_from!(1))
        });

        memory_init.configure_in_table(meta, "rest mop zero when disabled", |meta| {
            (self.is_enable(meta) - constant_from!(1)) * cur!(meta, self.rest_mops)
        });

        self
    }

    fn is_heap(&self, meta: &mut VirtualCells<F>) -> Expression<F> {
        Expression::Constant(F::one()) - self.ltype.data(meta)
    }

    fn is_stack(&self, meta: &mut VirtualCells<F>) -> Expression<F> {
        self.ltype.data(meta)
    }

    fn diff(&self, meta: &mut VirtualCells<F>, col: Column<Advice>) -> Expression<F> {
        cur!(meta, col) - pre!(meta, col)
    }

    fn is_read_not_bit(&self, meta: &mut VirtualCells<F>) -> Expression<F> {
        let atype = cur!(meta, self.atype);
        (atype.clone() - constant_from!(AccessType::Init))
            * (atype - constant_from!(AccessType::Write))
    }

    fn is_not_init(&self, meta: &mut VirtualCells<F>) -> Expression<F> {
        let read_f = F::from(AccessType::Read as u64);
        let write_f = F::from(AccessType::Write as u64);
        let init_f = F::from(AccessType::Init as u64);
        let atype = cur!(meta, self.atype);
        (atype.clone() - constant_from!(AccessType::Write))
            * (atype.clone() - constant_from!(AccessType::Init))
            * constant!(((read_f - write_f) * (read_f - init_f)).invert().unwrap())
            + (atype.clone() - constant_from!(AccessType::Read))
                * (atype.clone() - constant_from!(AccessType::Init))
                * constant!(((write_f - read_f) * (write_f - init_f)).invert().unwrap())
    }

    fn is_same_location(&self, meta: &mut VirtualCells<F>) -> Expression<F> {
        cur!(meta, self.same_location)
    }

    fn is_enable(&self, meta: &mut VirtualCells<F>) -> Expression<F> {
        cur!(meta, self.enable)
    }
}

pub struct MemoryChip<F: FieldExt> {
    config: MemoryConfig<F>,
    _phantom: PhantomData<F>,
}

impl<F: FieldExt> MemoryChip<F> {
    pub fn new(config: MemoryConfig<F>) -> MemoryChip<F> {
        MemoryChip {
            config,
            _phantom: PhantomData,
        }
    }

    pub fn assign(
        &self,
        ctx: &mut Context<'_, F>,
        entries: &Vec<MemoryTableEntry>,
        etable_rest_mops_cell: Cell,
    ) -> Result<(), Error> {
        let mut mops = entries.iter().fold(0, |acc, e| {
            acc + if e.atype == AccessType::Init { 0 } else { 1 }
        });
        let mut last_entry: Option<&MemoryTableEntry> = None;
        for (i, entry) in entries.into_iter().enumerate() {
            macro_rules! row_diff_assign {
                ($x: ident) => {
                    self.config.$x.assign(
                        ctx,
                        (entry.$x as u64).into(),
                        ((entry.$x as u64)
                            - last_entry.as_ref().map(|x| x.$x as u64).unwrap_or(0u64))
                        .into(),
                    )?;
                };
            }

            row_diff_assign!(eid);
            row_diff_assign!(emid);
            row_diff_assign!(mmid);
            row_diff_assign!(offset);
            row_diff_assign!(ltype);

            ctx.region.assign_advice(
                || "memory atype",
                self.config.atype,
                ctx.offset,
                || Ok((entry.atype as u64).into()),
            )?;

            ctx.region.assign_advice(
                || "memory vtype",
                self.config.vtype,
                ctx.offset,
                || Ok((entry.vtype as u64).into()),
            )?;

            ctx.region.assign_advice(
                || "memory value",
                self.config.value,
                ctx.offset,
                || Ok((entry.value as u64).into()),
            )?;

            ctx.region.assign_advice(
                || "memory enable",
                self.config.enable,
                ctx.offset,
                || Ok(F::one()),
            )?;

            let cell = ctx.region.assign_advice(
                || "memory enable",
                self.config.rest_mops,
                ctx.offset,
                || Ok(F::from(mops)),
            )?;
            if i == 0 {
                ctx.region
                    .constrain_equal(cell.cell(), etable_rest_mops_cell)?;
            }

            ctx.region.assign_advice(
                || "memory same_location",
                self.config.same_location,
                ctx.offset,
                || {
                    Ok(last_entry.as_ref().map_or(F::zero(), |last_entry| {
                        if last_entry.is_same_location(&entry) {
                            F::one()
                        } else {
                            F::zero()
                        }
                    }))
                },
            )?;

            if entry.atype != AccessType::Init {
                mops -= 1;
            }
            last_entry = Some(entry);
            ctx.next();
        }

        Ok(())
    }
}
