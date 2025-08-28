/// Decimal fix-point number with 12 decimal places backed by U256
/// backward-compatible with spl-math's PreciseNumber (12 decimal places)

use crate::define_precise_number;
use crate::uint::U256;

const ONE_CONST: U256 = U256([1000000000000, 0, 0, 0]);
const ROUNDING_CORRECTION: U256 = U256([1000000000000 / 2, 0, 0, 0]);
const PRECISION: U256 = U256([100, 0, 0, 0]);
const MAXIMUM_SQRT_BASE: U256 = U256([18446743073709551616, 18446744073709551615, 999999999999, 0]); // u128::MAX
define_precise_number!(
    PreciseNumber,
    u128,
    U256,
    ONE_CONST,
    U256::zero(),
    ROUNDING_CORRECTION,
    PRECISION,
    MAXIMUM_SQRT_BASE
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u256_max_outer_to_precise() {
        let a = PreciseNumber::new(u128::MAX).unwrap();
        assert_eq!(a.to_imprecise().unwrap(), u128::MAX);
        let a_plus_1 = a.checked_add(&PreciseNumber::one()).unwrap();
        assert!(a_plus_1.to_imprecise().is_none());
    }

    #[test]
    fn test_u256_one_constant() {
        let one = U256::from(1_000_000_000_000u128);
        assert_eq!(ONE_CONST, one);
    }

    #[test]
    fn test_u256_rounding_correction_constant() {
        let rounding = U256::from(1_000_000_000_000u128) / 2;
        assert_eq!(ROUNDING_CORRECTION, rounding);
    }

    #[test]
    fn test_u256_maximum_sqrt_base_constant() {
        assert_eq!(MAXIMUM_SQRT_BASE, PreciseNumber::new(u128::MAX).unwrap().value);
    }
    
    #[test]
    fn test_u256_precision_constant() {
        assert_eq!(PRECISION, U256::from(100u128)); // 1e-10
    }
    
}
