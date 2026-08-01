/// Decimal fix-point number with 12 decimal places backed by U256
/// backward-compatible with spl-math's PreciseNumber (12 decimal places)
use crate::precise_number::convert_from_f64::u256_from_f64_bits;
use crate::uint::{U256, U512};
use crate::{
    define_log10, define_log10_tests, define_muldiv, define_precise_number, define_sqrt_tests,
};

const ONE_CONST: U256 = U256([1000000000000, 0, 0, 0]);
const ROUNDING_CORRECTION: U256 = U256([1000000000000 / 2, 0, 0, 0]);
const PRECISION: U256 = U256([100, 0, 0, 0]);
const MAXIMUM_SQRT_BASE: U256 = U256([18446743073709551616, 18446744073709551615, 999999999999, 0]); // u128::MAX
define_precise_number!(
    PreciseNumber,
    u128,
    U256,
    ONE_CONST,
    1e12f64,
    U256::zero(),
    ROUNDING_CORRECTION,
    PRECISION,
    MAXIMUM_SQRT_BASE,
    |value| u256_from_f64_bits(value)
);
define_muldiv!(PreciseNumber, u128, U256, U512);
define_log10!(PreciseNumber, U256, U256([301029995664, 0, 0, 0]));
define_sqrt_tests!(PreciseNumber, u128, U256, U512, (12, 11));
define_log10_tests!(PreciseNumber, u128, U256, 11);

#[cfg(test)]
mod tests {
    use super::*;


    /// `checked_div` keeps the full fixed-point precision, so rounding only
    /// happens on the last representable digit (1e-12). These are the edge
    /// cases around that digit: the quotient goes to the nearer neighbour and
    /// an exact half is broken downwards.
    #[test]
    fn test_roundmode_half_down() {
        // type InnerUint = U256;
        // pub const ONE: u128 = 1_000_000_000_000;
        const ONE: u128 = 1_000_000_000_000;
        // operate on raw fixed-point values so the tie sits on the last digit
        fn div(dividend: u128, divisor: u128) -> u128 {
            let dividend = PreciseNumber {
                value: U256::from(dividend),
            };
            let divisor = PreciseNumber {
                value: U256::from(divisor),
            };
            dividend.checked_div(&divisor).unwrap().value.as_u128()
        }

        // 1.0 / 4096 = 0.000244140625 is exactly representable, nothing to round
        assert_eq!(div(ONE, 4096 * ONE), 244_140_625);

        // 1.0 / 8192 = 0.0001220703125 falls exactly between two representable
        // values (122070312.5e-12), the tie must round down
        assert_eq!(div(ONE, 8192 * ONE), 122_070_312);
        // one ulp of dividend above that tie rounds up ...
        assert_eq!(div(ONE + 1, 8192 * ONE), 122_070_313);
        // ... and one ulp below it rounds down
        assert_eq!(div(ONE - 1, 8192 * ONE), 122_070_312);

        // remainders away from the tie go to the nearer neighbour
        assert_eq!(div(ONE, 3 * ONE), 333_333_333_333); // 0.333... down
        assert_eq!(div(2 * ONE, 3 * ONE), 666_666_666_667); // 0.666... up

        // a divisor below 1.0 must not inflate the quotient:
        // 1e-12 / 0.4 = 2.5e-12, again a tie that rounds down
        assert_eq!(div(1, 4 * ONE / 10), 2);
    }

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
        assert_eq!(
            MAXIMUM_SQRT_BASE,
            PreciseNumber::new(u128::MAX).unwrap().value
        );
    }

    #[test]
    fn test_u256_precision_constant() {
        assert_eq!(PRECISION, U256::from(100u128)); // 1e-10
    }

    #[test]
    fn test_call_muldiv() {
        let a = PreciseNumber::new(10).unwrap();
        let b = PreciseNumber::new(5).unwrap();
        let c = PreciseNumber::new(2).unwrap();

        // (10 * 5) / 2 = 25
        let result = a.mul_div_floor(b, c).unwrap();
        assert_eq!(result, PreciseNumber::new(25).unwrap());
    }

    #[test]
    fn test_precompute_log10_of_2() {
        // round(log10(2) * 10^12)
        let log10_of_2 = 2.0f64.log10();
        let computed = (log10_of_2 * 1e12).round() as u128;
        assert_eq!(computed, PreciseNumber::LOG10_OF_2.as_u128());
    }

    use super::PreciseNumber;
    use crate::uint::U256;

    /**
     * fixed test for POC of SPLM-11: `checked_div` inflates exact sub-unit ratios
     * @notice Proof Statement: Prove that the public `checked_div` fast path returns
     * a quotient strictly greater than `1.0` for reachable inputs `0 < x <= 0.5`
     * even when dividing a value by itself, and that the same path biases sub-unit
     * reciprocals consumed by higher-level callers such as `signed_log10`.
     */
    #[test]
    fn test_poc_checked_div_inflates_exact_sub_unit_quotients() {
        let six_tenths = PreciseNumber::new(6)
            .unwrap()
            .checked_div(&PreciseNumber::new(10).unwrap())
            .unwrap();
        assert_eq!(
            six_tenths.checked_div(&six_tenths).unwrap(),
            PreciseNumber::one(),
            "values above 0.5 stay exact"
        );

        let half = PreciseNumber::new(1)
            .unwrap()
            .checked_div(&PreciseNumber::new(2).unwrap())
            .unwrap();
        assert_eq!(
            half.checked_div(&half).unwrap().value,
            PreciseNumber::FP_ONE,
            "0.5 / 0.5 should be exact"
        );

        let one_tenth = PreciseNumber::new(1)
            .unwrap()
            .checked_div(&PreciseNumber::new(10).unwrap())
            .unwrap();
        assert_eq!(
            one_tenth.checked_div(&one_tenth).unwrap().value,
            PreciseNumber::FP_ONE,
            "0.1 / 0.1 should equal 1.0"
        );

        let minimum_unit = PreciseNumber::new_from_f64(1e-12).unwrap();
        assert_eq!(minimum_unit.value, U256::from(1u8));
        assert_eq!(
            minimum_unit.checked_div(&minimum_unit).unwrap().value,
            PreciseNumber::FP_ONE,
            "the minimum positive unit must not inflated"
        );

        assert_eq!(
            PreciseNumber::one().checked_div(&one_tenth).unwrap().value,
            U256::from(10_000_000_000_000u128),
            "reciprocals below one inherit the same upward bias"
        );

    }


    /**
     * Part of SPLM-11 findings: the log2 is imprecise due to the upward bias of `checked_div` for sub-unit ratios, which is consumed by `signed_log10` and other higher-level functions.
     * Fix is possible but very complex - so we accept the deviation.
     */
    #[test]
    fn test_inprecise_log2() {
        let one_tenth = PreciseNumber::new(1)
            .unwrap()
            .checked_div(&PreciseNumber::new(10).unwrap())
            .unwrap();

        let (signed_log10_tenth, negative) = one_tenth.signed_log10().unwrap();
        assert!(negative, "log10(0.1) must be negative");
        assert_eq!(
            signed_log10_tenth.value,
            U256::from(999_999_999_998u128), // should be 1.0
            "signed_log10(0.1) misses the exact decade because it consumes the biased reciprocal"
        );
    }

}
