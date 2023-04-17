use std::io::Read;

use halo2_proofs::{arithmetic::FieldExt, circuit::Region};
use num_bigint::BigUint;

pub mod row_diff;

pub struct Context<'a, F: FieldExt> {
    pub region: Box<Region<'a, F>>,
    pub offset: usize,
    records: Vec<usize>,
}

impl<'a, F: FieldExt> Context<'a, F> {
    pub fn new(region: Region<'a, F>) -> Context<'a, F> {
        Context {
            region: Box::new(region),
            offset: 0usize,
            records: vec![],
        }
    }

    pub fn next(&mut self) {
        self.offset += 1;
    }

    pub fn reset(&mut self) {
        self.offset = 0;
        self.records.clear();
    }

    pub fn push(&mut self) {
        self.records.push(self.offset);
    }

    pub fn pop(&mut self) {
        self.offset = self.records.pop().unwrap();
    }
}

pub fn bn_to_field<F: FieldExt>(bn: &BigUint) -> F {
    let mut bytes = bn.to_bytes_le();
    bytes.resize(64, 0);
    let mut bytes = &bytes[..];

    let mut compressed = [0u8; 64];
    bytes.read_exact(&mut compressed[..]).unwrap();
    F::from_bytes_wide(&mut compressed)
}

#[macro_export]
macro_rules! cur {
    ($meta: expr, $x: expr) => {
        $meta.query_advice($x, halo2_proofs::poly::Rotation::cur())
    };
}

#[macro_export]
macro_rules! pre {
    ($meta: expr, $x: expr) => {
        $meta.query_advice($x, halo2_proofs::poly::Rotation::prev())
    };
}

#[macro_export]
macro_rules! next {
    ($meta: expr, $x: expr) => {
        $meta.query_advice($x, halo2_proofs::poly::Rotation::next())
    };
}

#[macro_export]
macro_rules! constant_from {
    ($x: expr) => {
        halo2_proofs::plonk::Expression::Constant(F::from($x as u64))
    };
}

#[macro_export]
macro_rules! constant {
    ($x: expr) => {
        halo2_proofs::plonk::Expression::Constant($x)
    };
}
