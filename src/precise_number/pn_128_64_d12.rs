/// Decimal fix-point number with 12 decimal places backed by u128
///
use crate::define_precise_number;

const ONE_CONST: u128 = 1_000_000_000;
const ROUNDING_CORRECTION: u128 = 1_000_000_000 / 2;
const PRECISION: u128 = 10; // TODO
const MAXIMUM_SQRT_BASE: u128 = u64::MAX as u128 * ONE_CONST;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u128_max_outer_to_precise() {
        let a = PreciseNumber::new(u64::MAX).unwrap();
        assert_eq!(a.to_imprecise().unwrap(), u64::MAX);
        let a_plus_1 = a.checked_add(&PreciseNumber::one()).unwrap();
        assert!(a_plus_1.to_imprecise().is_none());
    }

    #[test]
    fn test_u128_one_constant() {
        assert_eq!(ONE_CONST, 1_000_000_000);
    }

    #[test]
    fn test_u128_rounding_correction_constant() {
        assert_eq!(ROUNDING_CORRECTION, 500_000_000);
    }

    #[test]
    fn test_u128_maximum_sqrt_base_constant() {
        assert_eq!(MAXIMUM_SQRT_BASE, PreciseNumber::new(u64::MAX).unwrap().value);
    }
}
