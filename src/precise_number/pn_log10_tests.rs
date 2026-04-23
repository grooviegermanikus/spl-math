#[cfg(test)]
mod tests {
    use crate::precise_number::convert_from_f64::u256_from_f64_bits;
    use crate::uint::U256;
    use crate::{define_log10, define_precise_number};
    use bigdecimal_rs::BigDecimal;
    use proptest::prelude::ProptestConfig;
    use proptest::proptest;
    use std::ops::Div;
    use std::str::FromStr;

    type InnerUint = U256;

    const ONE_CONST: U256 = U256([1000000000000, 0, 0, 0]);
    const ROUNDING_CORRECTION: U256 = U256([1000000000000 / 2, 0, 0, 0]);
    const PRECISION: U256 = U256([100, 0, 0, 0]);
    const MAXIMUM_SQRT_BASE: U256 =
        U256([18446743073709551616, 18446744073709551615, 999999999999, 0]); // u128::MAX
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
    define_log10!(PreciseNumber, U256, U256([301029995664, 0, 0, 0]));

    #[test]
    fn test_log10_powers_of_ten() {
        for exp in 1..=18u32 {
            let x = PreciseNumber::new(10u128.pow(exp)).unwrap();
            let result = x.log10().unwrap();
            let expected = PreciseNumber::new(exp as u128).unwrap();
            assert!(
                result.almost_eq(&expected, PreciseNumber::PRECISION),
                "log10(10^{}) = {} expected {}",
                exp,
                pretty_string(&result),
                pretty_string(&expected)
            );
        }
    }

    #[test]
    fn test_log2_powers_of_two() {
        for exp in 1..=40u32 {
            let x = PreciseNumber::new(1u128 << exp).unwrap();
            let result = x.log2().unwrap();
            let expected = PreciseNumber::new(exp as u128).unwrap();
            assert!(
                result.almost_eq(&expected, PreciseNumber::PRECISION),
                "log2(2^{}) = {} expected {}",
                exp,
                pretty_string(&result),
                pretty_string(&expected)
            );
        }
    }

    #[test]
    fn test_log10_known_values() {
        // log10(2) ≈ 0.30102999566
        let two = PreciseNumber::new(2).unwrap();
        let result = two.log10().unwrap();
        let expected_str = "0.30102999566";
        let result_bd = BigDecimal::from_str(&pretty_string(&result)).unwrap();
        let expected_bd = BigDecimal::from_str(expected_str).unwrap();
        let diff = (&result_bd - &expected_bd).abs();
        assert!(
            diff < BigDecimal::from_str("1e-10").unwrap(),
            "log10(2) = {} expected ~{}",
            pretty_string(&result),
            expected_str
        );

        // log10(3) ≈ 0.47712125472
        let three = PreciseNumber::new(3).unwrap();
        let result = three.log10().unwrap();
        let result_bd = BigDecimal::from_str(&pretty_string(&result)).unwrap();
        let expected_bd = BigDecimal::from_str("0.47712125472").unwrap();
        let diff = (&result_bd - &expected_bd).abs();
        assert!(
            diff < BigDecimal::from_str("1e-10").unwrap(),
            "log10(3) = {} expected ~0.47712125472",
            pretty_string(&result)
        );
    }

    #[test]
    fn test_log10_large_values() {
        // log10(u128::MAX) ≈ 38.53
        let max = PreciseNumber::new(u128::MAX).unwrap();
        let result = max.log10().unwrap();
        let result_f64: f64 = BigDecimal::from_str(&pretty_string(&result))
            .unwrap()
            .to_string()
            .parse()
            .unwrap();
        let expected = (u128::MAX as f64).log10();
        assert!(
            (result_f64 - expected).abs() < 0.001,
            "log10(u128::MAX) = {} expected ~{}",
            result_f64,
            expected
        );
    }

    #[test]
    fn test_log10_precision_across_range() {
        let test_values: Vec<u128> = vec![2, 3, 5, 7, 42, 100, 1000, 99999, 123456789];

        for &val in &test_values {
            let x = PreciseNumber::new(val).unwrap();
            let result = x.log10().unwrap();
            let result_bd = BigDecimal::from_str(&pretty_string(&result)).unwrap();
            let expected = (val as f64).log10();
            let expected_bd = BigDecimal::from_str(&format!("{:.15}", expected)).unwrap();
            let diff = (&result_bd - &expected_bd).abs();
            let precision = find_precision_digits(&diff);
            assert!(
                precision >= 10,
                "log10({}) precision {} < 10, result={} expected={}",
                val,
                precision,
                pretty_string(&result),
                expected
            );
        }
    }

    fn find_precision_digits(diff: &BigDecimal) -> u32 {
        for digits in 0..20 {
            let eps = BigDecimal::from_str(&format!("1e-{}", digits)).unwrap();
            if *diff >= eps {
                return if digits > 0 { digits - 1 } else { 0 };
            }
        }
        20
    }

    fn compare_log10_pn_vs_f64(i: u128) -> f64 {
        let pn = PreciseNumber {
            value: InnerUint::from(i),
        };
        // i represents i / ONE in real value
        if pn.value < PreciseNumber::FP_ONE {
            return 0.0; // skip values < 1
        }

        let pn_log10 = pn.log10().unwrap();

        let fx_one = BigDecimal::from_str("1000000000000").unwrap();
        let fx_x = BigDecimal::from_str(&i.to_string()).unwrap() / fx_one.clone();
        let fx_x_f64: f64 = fx_x.to_string().parse().unwrap();
        let fx_log10 = fx_x_f64.log10();

        let pn_log10_bd = BigDecimal::from_str(&format!("{}", pn_log10.value.as_u128()))
            .unwrap()
            .div(fx_one);

        
        (BigDecimal::from_str(&format!("{:.20}", fx_log10)).unwrap() - pn_log10_bd)
            .abs()
            .to_string()
            .parse::<f64>()
            .unwrap_or(1.0)
    }

    #[test]
    fn test_log10_fixed_range() {
        // values around 1 (scaled by 1e12)
        let around_one = (1_000_000_000_000u128..1_100_000_000_000u128).step_by(10_000_000_000);
        // medium values
        let medium = (10_000_000_000_000u128..20_000_000_000_000u128).step_by(1_000_000_000_000);
        // large values
        let large = ((u128::MAX - 1_000_000)..u128::MAX).step_by(100_000);

        for i in around_one.chain(medium).chain(large) {
            let diff = compare_log10_pn_vs_f64(i);
            assert!(
                diff < 0.000_000_01,
                "Difference ({}) too large for i={}",
                diff,
                i
            );
        }
    }

    #[test]
    fn test_signed_log10_fractional_known_values() {
        // log10(0.5) = log10(1/2) = -log10(2) ≈ -0.30102999566
        let half = PreciseNumber::one().div2();
        let (result, negative) = half.signed_log10().unwrap();
        assert!(negative);
        let result_bd = BigDecimal::from_str(&pretty_string(&result)).unwrap();
        let expected_bd = BigDecimal::from_str("0.30102999566").unwrap();
        let diff = (&result_bd - &expected_bd).abs();
        assert!(
            diff < BigDecimal::from_str("1e-8").unwrap(),
            "signed_log10(0.5) magnitude = {} expected ~0.30102999566",
            pretty_string(&result)
        );

        // log10(0.01) = -2
        let hundredth = PreciseNumber::one().div10().div10();
        let (result, negative) = hundredth.signed_log10().unwrap();
        assert!(negative);
        let expected_2 = PreciseNumber::one()
            .checked_add(&PreciseNumber::one())
            .unwrap();
        assert!(
            result.almost_eq(&expected_2, PreciseNumber::PRECISION),
            "signed_log10(0.01) magnitude = {} expected 2.0",
            pretty_string(&result)
        );
    }

    #[test]
    fn test_signed_log10_symmetry() {
        // For any x > 1: signed_log10(x) and signed_log10(1/x) should have
        // same magnitude but opposite signs
        let test_values: Vec<u128> = vec![2, 5, 10, 100, 1000];

        for &val in &test_values {
            let x = PreciseNumber::new(val).unwrap();
            let reciprocal = PreciseNumber::one().checked_div(&x).unwrap();

            let (mag_x, neg_x) = x.signed_log10().unwrap();
            let (mag_r, neg_r) = reciprocal.signed_log10().unwrap();

            assert!(!neg_x, "log10({}) should be positive", val);
            assert!(neg_r, "log10(1/{}) should be negative", val);

            // magnitudes should be approximately equal
            assert!(
                mag_x.almost_eq(&mag_r, PreciseNumber::PRECISION),
                "log10({}) magnitude {} != log10(1/{}) magnitude {}",
                val,
                pretty_string(&mag_x),
                val,
                pretty_string(&mag_r)
            );
        }
    }

    fn compare_signed_log10_pn_vs_f64(i: u128) -> f64 {
        let pn = PreciseNumber {
            value: InnerUint::from(i),
        };
        if pn.value == PreciseNumber::FP_ZERO {
            return 0.0;
        }

        let (magnitude, negative) = pn.signed_log10().unwrap();

        let fx_one = BigDecimal::from_str("1000000000000").unwrap();
        let fx_x = BigDecimal::from_str(&i.to_string()).unwrap() / fx_one.clone();
        let fx_x_f64: f64 = fx_x.to_string().parse().unwrap();
        let fx_log10 = fx_x_f64.log10(); // negative for x < 1

        let pn_log10_bd = BigDecimal::from_str(&format!("{}", magnitude.value.as_u128()))
            .unwrap()
            .div(fx_one);
        let pn_signed = if negative { -pn_log10_bd } else { pn_log10_bd };

        
        (BigDecimal::from_str(&format!("{:.20}", fx_log10)).unwrap() - pn_signed)
            .abs()
            .to_string()
            .parse::<f64>()
            .unwrap_or(1.0)
    }

    #[test]
    fn test_signed_log10_fractional_range() {
        // values between 0.1 and 1.0 (scaled by 1e12)
        let near_one = (100_000_000_000u128..1_000_000_000_000u128).step_by(50_000_000_000);
        // small values around 0.001
        let small = (1_000_000_000u128..10_000_000_000u128).step_by(1_000_000_000);

        for i in near_one.chain(small) {
            let diff = compare_signed_log10_pn_vs_f64(i);
            assert!(
                diff < 0.000_000_01,
                "Difference ({}) too large for i={}",
                diff,
                i
            );
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 1_000,
            timeout: 30,
            ..ProptestConfig::default()
        })]

        #[test]
        fn test_log10_random(a in 1_000_000_000_000u128..u128::MAX) {
            // a is already in inner representation (>= FP_ONE, i.e., x >= 1)
            let pn = PreciseNumber { value: InnerUint::from(a) };
            let result = pn.log10();
            assert!(result.is_some(), "log10 should succeed for value >= 1");
        }

        #[test]
        fn test_log2_random(a in 1_000_000_000_000u128..u128::MAX) {
            let pn = PreciseNumber { value: InnerUint::from(a) };
            let result = pn.log2();
            assert!(result.is_some(), "log2 should succeed for value >= 1");
        }

        #[test]
        fn test_log10_monotonic(a in 1_000_000_000_000u128..u128::MAX/2) {
            let pn_a = PreciseNumber { value: InnerUint::from(a) };
            let pn_b = PreciseNumber { value: InnerUint::from(a + 1) };
            let log_a = pn_a.log10().unwrap();
            let log_b = pn_b.log10().unwrap();
            assert!(log_b.value >= log_a.value, "log10 must be monotonically increasing");
        }

        #[test]
        fn test_signed_log10_random(a in 1u128..u128::MAX) {
            // any value > 0 in inner representation should succeed
            let pn = PreciseNumber { value: InnerUint::from(a) };
            let result = pn.signed_log10();
            assert!(result.is_some(), "signed_log10 should succeed for value > 0, got None for {}", a);
        }

        #[test]
        fn test_signed_log10_monotonic_below_one(a in 1u128..999_999_999_999u128) {
            // for 0 < a < b < 1: log10(a) < log10(b) (both negative, a more negative)
            let pn_a = PreciseNumber { value: InnerUint::from(a) };
            let pn_b = PreciseNumber { value: InnerUint::from(a + 1) };
            let (mag_a, neg_a) = pn_a.signed_log10().unwrap();
            let (mag_b, neg_b) = pn_b.signed_log10().unwrap();
            // both should be negative (below FP_ONE)
            assert!(neg_a, "should be negative for value < FP_ONE");
            assert!(neg_b, "should be negative for value < FP_ONE");
            // magnitude of a should be >= magnitude of b (a is smaller, so more negative)
            assert!(mag_a.value >= mag_b.value,
                "signed_log10 magnitude must decrease as x increases toward 1");
        }
    }

    fn pretty_string(pn: &PreciseNumber) -> String {
        use bigdecimal_rs::BigDecimal;
        use std::ops::Div;
        use std::str::FromStr;
        let bd = BigDecimal::from_str(&format!("{}", pn.value))
            .unwrap()
            .div(BigDecimal::from_str(&format!("{}", PreciseNumber::FP_ONE)).unwrap());
        format!("{}", bd)
    }
}
