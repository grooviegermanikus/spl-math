use crate::precise_number::convert_from_f64::u256_from_f64_bits;
use crate::uint::{U256, U512};
/// Decimal fix-point number with 18 decimal places backed by U256
/// 18 decimal places are recommended for most DeFi applications
use crate::{define_muldiv, define_precise_number, define_sqrt_tests};

const ONE_CONST: U256 = U256([1000000000000000000, 0, 0, 0]);
const ROUNDING_CORRECTION: U256 = U256([1000000000000000000 / 2, 0, 0, 0]);
const PRECISION: U256 = U256([10000000000000000000,1000000000, 0, 0]); // TODO little-endian
                                              // TODO
const MAXIMUM_SQRT_BASE: U256 = U256([18446743073709551616, 18446744073709551615, 999999999999, 0]); // u128::MAX
define_precise_number!(
    PreciseNumber,
    u128,
    U256,
    ONE_CONST,
    1e18f64,
    U256::zero(),
    ROUNDING_CORRECTION,
    PRECISION,
    MAXIMUM_SQRT_BASE,
    |value| u256_from_f64_bits(value)
);
define_muldiv!(PreciseNumber, u128, U256, U512);
define_sqrt_tests!(PreciseNumber, u128, U256, U512);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u256_one_constant() {
        let one = U256::from(1_000000_000000_000000u128);
        assert_eq!(ONE_CONST, one);
    }

    #[test]
    fn test_u256_rounding_correction_constant() {
        let rounding = U256::from(1_000000_000000_000000u128) / 2;
        assert_eq!(ROUNDING_CORRECTION, rounding);
    }

    #[test]
    fn test_u256_maximum_sqrt_base_constant() {
        // TODO
    }

    #[test]
    fn test_u256_precision_constant() {
        assert_eq!(PRECISION, U256::from(100u128)); // 1e-10
    }

    use crate::precise_number::pn_256_128_d18::PreciseNumber;
    use crate::uint::U256;

    type InnerUint = U256;

    // returns 10**(-digits) in InnerUint
    // for testing only, neither fast not beautiful
    fn precision(digits: u8) -> InnerUint {
        let ten = InnerUint::from(10);
        let mut result = ONE_CONST;
        for _ in 0..digits {
            result = result.checked_div(ten).unwrap();
        }
        assert!(!result.is_zero(), "precision underflow");
        result
    }

    #[test]
    fn test_from_f64() {
        let pn_from_inner = PreciseNumber::new_from_inner_f64(1e17).unwrap();
        assert_eq!(pn_from_inner.to_str_pretty(), "0.1");

        let pn = PreciseNumber::new_from_f64(1e-6).unwrap();
        assert_eq!(pn.to_str_pretty(), "0.000001");
    }

    #[test]
    fn test_precision() {
        // 10^-9
        let precision_9 = precision(9);
        assert_eq!(precision_9, InnerUint::from(1000000000));
    }

    // adopted from token-bonding-curve -> dfs_precise_number.rs
    #[test]
    fn test_sqrt_cordic_precision() {
        // TODO we need to tune down this parameter to make algo fast and precise enough

        // number below 1 (with uneven number of bits) 1.23456789e-9
        let number = PreciseNumber::new(123456789)
            .unwrap()
            .checked_div(&(PreciseNumber::new(10u128.pow(17)).unwrap()))
            .unwrap();
        // sqrt is 3.51364182864446216-5
        let expected_sqrt = PreciseNumber::new(351364182864446216)
            .unwrap()
            .checked_div(&(PreciseNumber::new(10u128.pow(22)).unwrap()))
            .unwrap();
        let cordic_sqrt = number
            .cordic_root_approximation_fast(PreciseNumber::NUM_BITS)
            .unwrap();
        assert!(
            cordic_sqrt
                // precise to first 9 decimals
                .almost_eq(&expected_sqrt, precision(9)),
            "sqrt {:?} not equal to expected {:?}",
            cordic_sqrt,
            expected_sqrt,
        );

        // exactly max_bits 18446744073709551615e-18 (this is 64 bits of 1, then divided by ONE)
        let number = PreciseNumber::new(18446744073709551615)
            .unwrap()
            .checked_div(&(PreciseNumber::new(10u128.pow(18)).unwrap()))
            .unwrap();
        assert_eq!(number.value.bits(), 64);
        // sqrt is 4.29496729599999999988
        let expected_sqrt = PreciseNumber::new(4294967295999999999)
            .unwrap()
            .checked_div(&(PreciseNumber::new(10u128.pow(18)).unwrap()))
            .unwrap();
        // TODO replace speed_factor with something better
        let cordic_sqrt = number
            .cordic_root_approximation_fast(PreciseNumber::NUM_BITS)
            .unwrap();
        assert!(
            cordic_sqrt
                // precise to first 9 decimals
                .almost_eq(&expected_sqrt, precision(9)),
            "sqrt {:?} not equal to expected {:?}",
            cordic_sqrt,
            expected_sqrt,
        );

        // 1 exactly
        let number = PreciseNumber::new(1).unwrap();
        // sqrt is 1
        let expected_sqrt = PreciseNumber::new(1).unwrap();
        let cordic_sqrt = number.sqrt_cordic().unwrap();
        assert!(
            cordic_sqrt
                // precise to first 12 decimals
                .almost_eq(&expected_sqrt, precision(12)),
            "sqrt {:?} not equal to expected {:?}",
            cordic_sqrt,
            expected_sqrt,
        );
    }
}
