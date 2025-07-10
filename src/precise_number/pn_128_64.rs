use crate::define_precise_number;

const ONE_CONST: u128 = 1_000_000_000;
const ROUNDING_CORRECTION: u128 = 1_000_000_000 / 2;
const PRECISION: u128 = 10;
const MAXIMUM_SQRT_BASE: u128 = u64::MAX as u128;
define_precise_number!(
    PreciseNumber,
    u64,
    u128,
    ONE_CONST,
    0u128,
    ROUNDING_CORRECTION,
    PRECISION,
    MAXIMUM_SQRT_BASE
);

