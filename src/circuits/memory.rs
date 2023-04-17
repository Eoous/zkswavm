use halo2_proofs::arithmetic::FieldExt;
use halo2_proofs::plonk::{Advice, Column, ConstraintSystem, Expression, VirtualCells};
use halo2_proofs::poly::Rotation;
use lazy_static::lazy_static;
use num_bigint::BigUint;
use specs::mtable::{AccessType, LocationType, MemoryTableEntry, VarType};
use std::marker::PhantomData;

use crate::circuits::memory_init::MemoryInitConfig;
use crate::circuits::range::RangeConfig;
use crate::circuits::row_diff::RowDiffConfig;
use crate::utils::bn_to_field;
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
    emid: Column<Advice>,
    mmid: RowDiffConfig<F>,
    offset: RowDiffConfig<F>,

    ltype: RowDiffConfig<F>,
    atype: Column<Advice>,
    vtype: Column<Advice>,
    value: Column<Advice>,
    enable: Column<Advice>,
    same_location: Column<Advice>,

    _mark: PhantomData<F>,
}

impl<F: FieldExt> MemoryConfig<F> {
    pub fn new(
        meta: &mut ConstraintSystem<F>,
        cols: &mut impl Iterator<Item = Column<Advice>>,
    ) -> MemoryConfig<F> {
        let ltype = RowDiffConfig::configure("location type", meta, cols);
        let mmid = RowDiffConfig::configure("mmid", meta, cols);
        let offset = RowDiffConfig::configure("mm offset", meta, cols);
        let eid = RowDiffConfig::configure("eid", meta, cols);
        let value = cols.next().unwrap();
        let atype = cols.next().unwrap();
        let vtype = cols.next().unwrap();
        let enable = cols.next().unwrap();
        let same_location = cols.next().unwrap();
        let emid = cols.next().unwrap();

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
            _mark: PhantomData,
        }
    }

    fn encode_for_lookup(&self, meta: &mut VirtualCells<'_, F>) -> Expression<F> {
        self.eid.data(meta) * constant!(bn_to_field(&EID_SHIFT))
            + cur!(meta, self.emid) * constant!(bn_to_field(&EMID_SHIFT))
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
        key_rev: &'static str,
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

        meta.lookup_any(key_rev, |meta| {
            vec![(
                self.encode_for_lookup(meta) * enable(meta),
                (eid(meta) * constant!(bn_to_field(&EID_SHIFT))
                    + emid(meta) * constant!(bn_to_field(&EMID_SHIFT))
                    + sp(meta) * constant!(bn_to_field(&OFFSET_SHIFT))
                    + constant!(bn_to_field(&LOC_TYPE_SHIFT))
                        * constant_from!(LocationType::Stack)
                    + constant!(bn_to_field(&ACCESS_TYPE_SHIFT))
                        * constant_from!(AccessType::Read)
                    + vtype(meta) * constant!(bn_to_field(&VAR_TYPE_SHIFT))
                    + value(meta))
                    * enable(meta),
            )]
        });
    }

    pub fn configure_stack_write_in_table(
        &self,
        key: &'static str,
        key_rev: &'static str,
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

        meta.lookup_any(key_rev, |meta| {
            vec![(
                self.encode_for_lookup(meta) * enable(meta),
                (eid(meta) * constant!(bn_to_field(&EID_SHIFT))
                    + emid(meta) * constant!(bn_to_field(&EMID_SHIFT))
                    + sp(meta) * constant!(bn_to_field(&OFFSET_SHIFT))
                    + constant!(bn_to_field(&LOC_TYPE_SHIFT))
                        * constant_from!(LocationType::Stack)
                    + constant!(bn_to_field(&ACCESS_TYPE_SHIFT))
                        * constant_from!(AccessType::Read)
                    + vtype(meta) * constant!(bn_to_field(&VAR_TYPE_SHIFT))
                    + value(meta))
                    * enable(meta),
            )]
        });
    }

    pub fn configure(
        meta: &mut ConstraintSystem<F>,
        cols: &mut impl Iterator<Item = Column<Advice>>,
        range: &RangeConfig<F>,
        memory_init: &MemoryInitConfig<F>,
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
        range.configure_in_range(meta, "mmid in range", |meta| self.mmid.data(meta));
        range.configure_in_range(meta, "offset in range", |meta| self.offset.data(meta));
        range.configure_in_range(meta, "eid in range", |meta| self.eid.data(meta));

        range.configure_in_range(meta, "emid in range", |meta| cur!(meta, self.emid));
        range.configure_in_range(meta, "vtype in range", |meta| cur!(meta, self.vtype));

        self
    }

    fn configure_sort(
        &self,
        meta: &mut ConstraintSystem<F>,
        range: &RangeConfig<F>,
    ) -> &MemoryConfig<F> {
        range.configure_in_range(meta, "ltype sort", |meta| {
            self.is_enable(meta) * self.ltype.diff(meta)
        });
        range.configure_in_range(meta, "mmid sort", |meta| {
            self.is_enable(meta) * self.ltype.is_same(meta) * self.mmid.diff(meta)
        });
        range.configure_in_range(meta, "offset sort", |meta| {
            self.is_enable(meta)
                * self.ltype.is_same(meta)
                * self.mmid.is_same(meta)
                * self.offset.is_same(meta)
        });
        range.configure_in_range(meta, "eid sort", |meta| {
            self.is_enable(meta) * self.is_same_location(meta) * self.eid.diff(meta)
        });
        range.configure_in_range(meta, "emid sort", |meta| {
            self.is_enable(meta)
                * self.is_same_location(meta)
                * self.eid.is_same(meta)
                * (cur!(meta, self.emid) - pre!(meta, self.emid))
        });

        self
    }

    fn configure_rule(
        &self,
        meta: &mut ConstraintSystem<F>,
        memory_init: &MemoryInitConfig<F>,
    ) -> &MemoryConfig<F> {
        meta.create_gate("read after write", |meta| {
            vec![
                self.is_enable(meta) * self.is_read_not_bit(meta) * self.diff(meta, self.value),
                self.is_enable(meta) * self.is_read_not_bit(meta) * self.diff(meta, self.vtype),
            ]
        });

        meta.create_gate("stack first line", |meta| {
            vec![
                self.is_enable(meta)
                    * (self.is_same_location(meta) - Expression::Constant(F::one()))
                    * self.is_stack(meta)
                    * (cur!(meta, self.atype) - constant_from!(AccessType::Write)),
            ]
        });

        // first line in heap
        memory_init.configure_in_table(meta, "heap first line", |meta| {
            self.is_enable(meta)
                * (Expression::Constant(F::one()) - self.is_same_location(meta))
                * self.is_heap(meta)
                * memory_init.encode(
                    self.mmid.data(meta),
                    self.offset.data(meta),
                    cur!(meta, self.value),
                )
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

    fn is_same_location(&self, meta: &mut VirtualCells<F>) -> Expression<F> {
        cur!(meta, self.same_location)
    }

    fn is_enable(&self, meta: &mut VirtualCells<F>) -> Expression<F> {
        cur!(meta, self.enable)
    }
}
