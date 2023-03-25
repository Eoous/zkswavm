use std::io::Read;
use halo2_proofs::{
    arithmetic::FieldExt,
    circuit::Region,
};
use num_bigint::BigUint;

pub struct Context<'a, F: FieldExt> {
    pub region: Box<Region<'a, F>>,
    pub offset: usize,
}

pub fn bn_to_field<F: FieldExt>(bn: &BigUint) -> F {
    let mut bytes = bn.to_bytes_le();
    bytes.resize(64, 0);
    let mut bytes = &bytes[..];

    let mut compressed = [0u8;64];
    bytes.read_exact(&mut compressed[..]).unwrap();
    F::from_bytes_wide(&mut compressed)
}

#[macro_export]
macro_rules! cur {
    ($meta: expr, $x: expr) => {
        $meta.query_advice($x, halo2_proofs::poly::Rotation::cur())
    };
}