/// Decimal fix-point number with 12 decimal places backed by u64
///
use crate::{define_muldiv, define_precise_number, define_sqrt_tests};
use num_traits::ToPrimitive;

const ONE_CONST: u64 = 10_000;
const ROUNDING_CORRECTION: u64 = 10_000 / 2;
const PRECISION: u64 = 3; // TODO
const MAXIMUM_SQRT_BASE: u64 = u32::MAX as u64 * ONE_CONST;
define_precise_number!(
    PreciseNumber,
    u32,
    u64,
    ONE_CONST,
    1e4f64,
    0u64,
    ROUNDING_CORRECTION,
    PRECISION,
    MAXIMUM_SQRT_BASE,
    |value| value.to_u64()
);
define_muldiv!(PreciseNumber, u32, u64, u128);
define_sqrt_tests!(PreciseNumber, u32, u64, u128);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u64_max_outer_to_precise() {
        let a = PreciseNumber::new(u32::MAX).unwrap();
        assert_eq!(a.to_imprecise().unwrap(), u32::MAX);
        let a_plus_1 = a.checked_add(&PreciseNumber::one()).unwrap();
        assert!(a_plus_1.to_imprecise().is_none());
    }

    #[test]
    fn test_u64_one_constant() {
        assert_eq!(ONE_CONST, 10000);
    }

    #[test]
    fn test_u64_rounding_correction_constant() {
        assert_eq!(ROUNDING_CORRECTION, 5000);
    }

    #[test]
    fn test_u64_maximum_sqrt_base_constant() {
        assert_eq!(
            MAXIMUM_SQRT_BASE,
            PreciseNumber::new(u32::MAX).unwrap().value
        );
    }

    #[test]
    fn test_call_muldiv() {
        let a = crate::precise_number::pn_128_64_d9::PreciseNumber::new(10).unwrap();
        let b = crate::precise_number::pn_128_64_d9::PreciseNumber::new(5).unwrap();
        let c = crate::precise_number::pn_128_64_d9::PreciseNumber::new(2).unwrap();
        let result = a.mul_div_floor(b, c).unwrap();
        assert_eq!(
            result,
            crate::precise_number::pn_128_64_d9::PreciseNumber::new(25).unwrap()
        );
    }
}
