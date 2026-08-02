//! Example of a precise number definition.

use num_traits::ToPrimitive;
use spl_math::define_precise_number;

define_precise_number!(
        TestPreciseNumber8,
        u8,
        u8,
        10u8,
        1e1f64,
        0u8,
        5u8,
        1u8,
        10u8,
        |value| value.to_u8()
    );
