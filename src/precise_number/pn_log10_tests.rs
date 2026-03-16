#[cfg(test)]
mod tests {
    use crate::{define_log10, define_precise_number};
    use crate::precise_number::convert_from_f64::u256_from_f64_bits;
    use crate::uint::U256;
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
                result.pretty_string(),
                expected.pretty_string()
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
                result.pretty_string(),
                expected.pretty_string()
            );
        }
    }

    #[test]
    fn test_log10_known_values() {
        // log10(2) ≈ 0.30102999566
        let two = PreciseNumber::new(2).unwrap();
        let result = two.log10().unwrap();
        let expected_str = "0.30102999566";
        let result_bd = BigDecimal::from_str(&result.pretty_string()).unwrap();
        let expected_bd = BigDecimal::from_str(expected_str).unwrap();
        let diff = (&result_bd - &expected_bd).abs();
        assert!(
            diff < BigDecimal::from_str("1e-10").unwrap(),
            "log10(2) = {} expected ~{}",
            result.pretty_string(),
            expected_str
        );

        // log10(3) ≈ 0.47712125472
        let three = PreciseNumber::new(3).unwrap();
        let result = three.log10().unwrap();
        let result_bd = BigDecimal::from_str(&result.pretty_string()).unwrap();
        let expected_bd = BigDecimal::from_str("0.47712125472").unwrap();
        let diff = (&result_bd - &expected_bd).abs();
        assert!(
            diff < BigDecimal::from_str("1e-10").unwrap(),
            "log10(3) = {} expected ~0.47712125472",
            result.pretty_string()
        );
    }

    #[test]
    fn test_log10_large_values() {
        // log10(u128::MAX) ≈ 38.53
        let max = PreciseNumber::new(u128::MAX).unwrap();
        let result = max.log10().unwrap();
        let result_f64: f64 = BigDecimal::from_str(&result.pretty_string())
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
            let result_bd = BigDecimal::from_str(&result.pretty_string()).unwrap();
            let expected = (val as f64).log10();
            let expected_bd = BigDecimal::from_str(&format!("{:.15}", expected)).unwrap();
            let diff = (&result_bd - &expected_bd).abs();
            let precision = find_precision_digits(&diff);
            assert!(
                precision >= 10,
                "log10({}) precision {} < 10, result={} expected={}",
                val,
                precision,
                result.pretty_string(),
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

        let diff = (BigDecimal::from_str(&format!("{:.20}", fx_log10)).unwrap() - pn_log10_bd)
            .abs()
            .to_string()
            .parse::<f64>()
            .unwrap_or(1.0);
        diff
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
    }

}
