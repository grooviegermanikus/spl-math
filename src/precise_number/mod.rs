mod convert_from_f64;
mod pn_128_64_d9;
mod pn_256_128_d12;
mod pn_256_128_d18;
mod pn_64_32_d4;
mod pn_impl;
mod pn_tests;
mod pn_muldiv_tests;

// type aliases for initial version of PreciseNumber
pub type PreciseNumber = pn_256_128_d12::PreciseNumber;
pub type PreciseNumber128D9 = pn_128_64_d9::PreciseNumber;
pub type PreciseNumber256D18 = pn_256_128_d18::PreciseNumber;
