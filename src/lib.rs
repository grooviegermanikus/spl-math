//! Math utilities.

extern crate core;

pub mod approximations;
pub mod checked_ceil_div;
pub mod uint;

// map precise number implementation
mod precise_number;
// type aliases for initial version of PreciseNumber
pub type PreciseNumber = precise_number::pn_256_128_d12::PreciseNumber;
pub type PreciseNumber128D12 = precise_number::pn_128_64_d12::PreciseNumber;
pub type PreciseNumber256D18 = precise_number::pn_256_128_d18::PreciseNumber;
