use num_bigint::BigUint;

pub mod config_builder;
pub mod event;
pub mod instruction;
pub mod jump;
pub mod memory;
pub mod memory_init;
pub mod range;
pub mod row_diff;

trait Encode {
    fn encode(&self) -> BigUint;
}
