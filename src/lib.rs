//! Math utilities.

extern crate core;

pub mod approximations;
pub mod checked_ceil_div;
pub mod uint;

// map precise number implementation
mod precise_number;
pub type PreciseNumber = precise_number::pn_256_128::PreciseNumber;
pub type PreciseNumber128 = precise_number::pn_128_64::PreciseNumber;
