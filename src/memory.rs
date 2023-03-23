use std::marker::PhantomData;
use halo2_proofs::arithmetic::FieldExt;
use halo2_proofs::plonk::{Advice, Column, ConstraintSystem, Expression, VirtualCells};
use halo2_proofs::poly::Rotation;

use crate::memory_init::MemoryInitConfig;
use crate::range::RangeConfig;
use crate::row_diff::RowDiffConfig;

pub enum LocationType {
    Heap,
    Stack,
}

impl<F: FieldExt> Into<Expression<F>> for LocationType {
    fn into(self) -> Expression<F> {
        match self {
            LocationType::Heap => Expression::Constant(F::from(0u64)),
            LocationType::Stack => Expression::Constant(F::from(1u64)),
        }
    }
}

pub enum AccessType {
    Read,
    Write,
    Init,
}

impl<F: FieldExt> Into<Expression<F>> for AccessType {
    fn into(self) -> Expression<F> {
        match self {
            AccessType::Read => Expression::Constant(F::from(1u64)),
            AccessType::Write => Expression::Constant(F::from(2u64)),
            AccessType::Init => Expression::Constant(F::from(3u64)),
        }
    }
}

pub enum VarType {
    U8,
    I32
}

pub struct MemoryEvent {
    eid: u64,
    mmid: u64,
    offset: u64,
    ltype: LocationType,
    atype: AccessType,
    vtype: VarType,
    value: u64,
}

impl MemoryEvent {
    pub fn new(
        eid: u64,
        mmid: u64,
        offset: u64,
        ltype: LocationType,
        atype: AccessType,
        vtype: VarType,
        value: u64,
    ) -> MemoryEvent {
        MemoryEvent {
            eid, mmid, offset,
            ltype, atype, vtype,
            value
        }
    }
}

pub struct MemoryConfig<F: FieldExt> {
    ltype: RowDiffConfig<F>,
    mmid: RowDiffConfig<F>,
    offset: RowDiffConfig<F>,
    eid: RowDiffConfig<F>,

    emid: Column<Advice>,
    atype: Column<Advice>,
    vtype: Column<Advice>,
    value: Column<Advice>,
    enable: Column<Advice>,
    same_location: Column<Advice>,

    _mark: PhantomData<F>,
}

impl<F: FieldExt> MemoryConfig<F> {
    fn new(
        meta: &mut ConstraintSystem<F>,
        cols: &mut impl Iterator<Item = Column<Advice>>
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
            ltype, mmid, offset, eid,
            emid, atype, vtype, value, enable, same_location,
            _mark: PhantomData,
        }
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
            let cur = meta.query_advice(self.enable, Rotation::cur());
            let next = meta.query_advice(self.enable, Rotation::next());

            vec![
                next * (cur.clone() - Expression::Constant(F::one())),
                cur.clone() * (cur.clone() - Expression::Constant(F::one())),
            ]
        });

        self
    }

    fn configure_same_location(&self, meta: &mut ConstraintSystem<F>) -> &MemoryConfig<F> {
        meta.create_gate("is same location", |meta| {
            let same_location = meta.query_advice(self.same_location, Rotation::cur());

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

            vec![
                ltype.clone() * (ltype - Expression::Constant(F::one()))
            ]
        });

        self
    }

    fn configure_range(&self, meta: &mut ConstraintSystem<F>, range: &RangeConfig<F>) -> &MemoryConfig<F> {
        range.configure_in_range(meta, |meta| self.mmid.data(meta));
        range.configure_in_range(meta, |meta| self.offset.data(meta));
        range.configure_in_range(meta, |meta| self.eid.data(meta));

        range.configure_in_range(meta, |meta| {
            meta.query_advice(self.emid, Rotation::cur())
        });
        range.configure_in_range(meta, |meta| {
            meta.query_advice(self.vtype, Rotation::cur())
        });

        self
    }

    fn configure_sort(&self, meta: &mut ConstraintSystem<F>, range: &RangeConfig<F>) -> &MemoryConfig<F> {
        range.configure_in_range(meta, |meta| {
            self.is_enable(meta) * self.ltype.diff(meta)
        });
        range.configure_in_range(meta, |meta| {
            self.is_enable(meta) * self.ltype.is_same(meta) * self.mmid.diff(meta)
        });
        range.configure_in_range(meta, |meta| {
            self.is_enable(meta) * self.ltype.is_same(meta) * self.mmid.is_same(meta) * self.offset.is_same(meta)
        });
        range.configure_in_range(meta, |meta| {
            self.is_enable(meta) * self.is_same_location(meta) * self.eid.diff(meta)
        });
        range.configure_in_range(meta, |meta| {
            self.is_enable(meta)
                * self.is_same_location(meta)
                * self.eid.is_same(meta)
                * (meta.query_advice(self.emid, Rotation::cur())
                 - meta.query_advice(self.emid, Rotation::prev()))
        });

        self
    }

    fn configure_rule(&self, meta: &mut ConstraintSystem<F>, memory_init: &MemoryInitConfig<F>) -> &MemoryConfig<F> {
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
                    * (meta.query_advice(self.atype, Rotation::cur()) - AccessType::Write.into()),
            ]
        });

        // first line in heap
        memory_init.configure_in_range(meta, |meta| {
            self.is_enable(meta)
                * (Expression::Constant(F::one()) - self.is_same_location(meta))
                * self.is_heap(meta)
                * memory_init.encode(
                self.mmid.data(meta),
                self.offset.data(meta),
                meta.query_advice(self.value, Rotation::cur())
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
        meta.query_advice(col, Rotation::cur()) - meta.query_advice(col, Rotation::prev())
    }

    fn is_read_not_bit(&self, meta: &mut VirtualCells<F>) -> Expression<F> {
        let atype = meta.query_advice(self.atype, Rotation::cur());
        (atype.clone() - AccessType::Init.into()) * (atype - AccessType::Write.into())
    }

    fn is_same_location(&self, meta: &mut VirtualCells<F>) -> Expression<F> {
        meta.query_advice(self.same_location, Rotation::cur())
    }

    fn is_enable(&self, meta: &mut VirtualCells<F>) -> Expression<F> {
        meta.query_advice(self.enable, Rotation::cur())
    }
}