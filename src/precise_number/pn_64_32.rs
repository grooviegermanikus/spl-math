use crate::define_precise_number;

const ONE_CONST: u64 = 10_000;
const ROUNDING_CORRECTION: u64 = 10_000 / 2;
const PRECISION: u64 = 3; // TODO
const MAXIMUM_SQRT_BASE: u64 = u32::MAX as u64;
define_precise_number!(
    PreciseNumber,
    u32,
    u64,
    ONE_CONST,
    0u64,
    ROUNDING_CORRECTION,
    PRECISION,
    MAXIMUM_SQRT_BASE
);

