/// Decimal fix-point number with 18 decimal places backed by U256
/// 18 decimal places are recommended for most DeFi applications

use crate::define_precise_number;
use crate::uint::U256;

const ONE_CONST: U256 = U256([1000000000000000000, 0, 0, 0]);
const ROUNDING_CORRECTION: U256 = U256([1000000000000000000 / 2, 0, 0, 0]);
const PRECISION: U256 = U256([100, 0, 0, 0]); // TODO
// TODO
const MAXIMUM_SQRT_BASE: U256 = U256([18446743073709551616, 18446744073709551615, 999999999999, 0]);
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


    use crate::uint::U256;
    use crate::precise_number::pn_256_128_d18::PreciseNumber;

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
    fn test_precision() {
        // 10^-9
        let precision_9 = precision(9);
        assert_eq!(precision_9, InnerUint::from(1000000000));
    }

    // adopted from token-bonding-curve -> dfs_precise_number.rs
    #[test]
    fn test_square_root_precision() {
        // number below 1 (with uneven number of bits) 1.23456789e-9
        let number = PreciseNumber::new(123456789)
            .checked_div(&(PreciseNumber::new(10u128.pow(17))))
            .unwrap();
        // sqrt is 3.51364182864446216-5
        let expected_sqrt = PreciseNumber::new(351364182864446216)
            .checked_div(&(PreciseNumber::new(10u128.pow(22))))
            .unwrap();
        assert!(
            number
                .sqrt()
                .unwrap()
                // precise to first 9 decimals
                .almost_eq(&expected_sqrt, precision(9)),
            "sqrt {:?} not equal to expected {:?}",
            number.sqrt().unwrap(),
            expected_sqrt,
        );

        // exactly max_bits 18446744073709551615e-18 (this is 64 bits of 1, then divided by ONE)
        let number = PreciseNumber::new(18446744073709551615)
            .checked_div(&(PreciseNumber::new(10u128.pow(18))))
            .unwrap();
        assert_eq!(number.value.bits(), 64);
        // sqrt is 4.29496729599999999988
        let expected_sqrt = PreciseNumber::new(4294967295999999999)
            .checked_div(&(PreciseNumber::new(10u128.pow(18))))
            .unwrap();
        assert!(
            number
                .sqrt()
                .unwrap()
                // precise to first 9 decimals
                .almost_eq(&expected_sqrt, precision(9)),
            "sqrt {:?} not equal to expected {:?}",
            number.sqrt().unwrap(),
            expected_sqrt,
        );

        // 1 exactly
        let number = PreciseNumber::new(1);
        // sqrt is 1
        let expected_sqrt = PreciseNumber::new(1);
        assert!(
            number
                .sqrt()
                .unwrap()
                // precise to first 12 decimals
                .almost_eq(&expected_sqrt, precision(12)),
            "sqrt {:?} not equal to expected {:?}",
            number.sqrt().unwrap(),
            expected_sqrt,
        );

        // TODO decide what to do
        // small perfect square (4e-12), should_round_up=false
        // let number = PreciseNumber::new(4)
        //     .checked_div(&(PreciseNumber::new(10u128.pow(12))))
        //     .unwrap();
        // // 2e-6, shouldn't do any rounding
        // let expected_sqrt = PreciseNumber::new(2)
        //     .checked_div(&(PreciseNumber::new(10u128.pow(6))))
        //     .unwrap();
        // assert!(
        //     number.sqrt().unwrap().eq(&expected_sqrt),
        //     "sqrt {:?} not equal to expected {:?}",
        //     number.sqrt().unwrap(),
        //     expected_sqrt,
        // );

    }



}
