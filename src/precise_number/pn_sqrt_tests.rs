#[cfg(test)]
mod tests {
    use std::ops::Div;
    use std::str::FromStr;
    use bigdecimal_rs::BigDecimal;
    use num_traits::ToPrimitive;
    use proptest::prelude::ProptestConfig;
    use proptest::proptest;
    use crate::define_precise_number;
    use crate::precise_number::convert_from_f64::u256_from_f64_bits;
    use crate::uint::U256;

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

    #[test]
    fn test_square_root_min_max() {
        let test_roots = [
            PreciseNumber::minimum_sqrt_base(),
            PreciseNumber::maximum_sqrt_base(),
        ];
        for i in test_roots.iter() {
            check_square_root(i);
        }
    }

    #[test]
    fn test_cordic_approximation() {
        // square root 0+1
        let test = PreciseNumber::new(0).unwrap();
        let root = test
            .sqrt()
            .unwrap()
            .to_imprecise()
            .unwrap();
        assert_eq!(root, 0);

        let test = PreciseNumber::new(1).unwrap();
        let root = test
            .sqrt()
            .unwrap()
            .to_imprecise()
            .unwrap();
        assert_eq!(root, 1);

        // square root
        let test = PreciseNumber::new(9).unwrap();
        let root = test
            .sqrt()
            .unwrap()
            .to_imprecise()
            .unwrap();
        assert_eq!(root, 3); // actually 3

        let test = PreciseNumber::new(101).unwrap();
        let root = test
            .sqrt()
            .unwrap()
            .to_imprecise()
            .unwrap();
        assert_eq!(root, 10); // actually 10.049875
    }


    // adopted from token-bonding-curve -> dfs_precise_number.rs
    #[test]
    fn test_square_root_precision() {
        // TODO we need to tune down this parameter to make algo fast and precise enough
        const SPEED_FACTOR: u32 = crate::precise_number::pn_256_128_d18::PreciseNumber::NUM_BITS;

        // number below 1 (with uneven number of bits) 1.23456789e-9
        let number = PreciseNumber::new(123456789)
            .unwrap()
            .checked_div(&(PreciseNumber::new(10u128.pow(17)).unwrap()))
            .unwrap();
        assert_eq!(number.value.as_u128(), 1234u128);
        // sqrt(1234e-12) = 3,512833614e-5
        let expected_sqrt = PreciseNumber::new(3_512_833_614)
            .unwrap()
            .checked_div(&(PreciseNumber::new(10u128.pow(14)).unwrap()))
            .unwrap();
        let cordic_sqrt = number.sqrt_cordic(SPEED_FACTOR).unwrap();
        assert!(
            cordic_sqrt
                // precise to first 9 decimals
                .almost_eq(&expected_sqrt, precision(9)),
            "sqrt {:?} not equal to expected {:?}",
            number.sqrt().unwrap(),
            expected_sqrt,
        );

        // exactly max_bits 18446744073709551615e-18 (this is 64 bits of 1, then divided by ONE)
        let number = PreciseNumber::new(18446744073709551615)
            .unwrap()
            .checked_div(&(PreciseNumber::new(10u128.pow(18)).unwrap()))
            .unwrap();
        // assert_eq!(number.value.bits(), 64);
        assert_eq!(number.value.bits(), 45);
        // sqrt is 4.29496729599999999988
        let expected_sqrt = PreciseNumber::new(4294967295999999999)
            .unwrap()
            .checked_div(&(PreciseNumber::new(10u128.pow(18)).unwrap()))
            .unwrap();
        let cordic_sqrt = number.sqrt_cordic(SPEED_FACTOR).unwrap();
        assert!(
            cordic_sqrt
                // precise to first 9 decimals
                .almost_eq(&expected_sqrt, precision(9)),
            "sqrt {:?} not equal to expected {:?}",
            number.sqrt().unwrap(),
            expected_sqrt,
        );

        // 1 exactly
        let number = PreciseNumber::new(1).unwrap();
        // sqrt is 1
        let expected_sqrt = PreciseNumber::new(1).unwrap();
        let cordic_sqrt = number.sqrt_cordic(SPEED_FACTOR).unwrap();
        assert!(
            cordic_sqrt
                // precise to first 12 decimals
                .almost_eq(&expected_sqrt, precision(12)),
            "sqrt {:?} not equal to expected {:?}",
            number.sqrt().unwrap(),
            expected_sqrt,
        );
    }


    fn check_square_root(check: &PreciseNumber) {
        let approximate_root = check.sqrt_cordic(PreciseNumber::NUM_BITS).unwrap();
        let (lower_bound, upper_bound) = calc_square_root_bounds(&approximate_root);
        assert!(check.less_than_or_equal(&upper_bound));
        assert!(check.greater_than_or_equal(&lower_bound));
    }

    fn calc_square_root_bounds(approximate_root: &PreciseNumber) -> (PreciseNumber, PreciseNumber) {
        let epsilon = PreciseNumber {
            value: precision(11),
        };
        let one = PreciseNumber::one();
        let one_plus_epsilon = one.checked_add(&epsilon).unwrap();
        let one_minus_epsilon = one.checked_sub(&epsilon).unwrap();
        let lower_bound = approximate_root
            .checked_mul(&one_minus_epsilon)
            .unwrap()
            .checked_pow(2)
            .unwrap();
        let upper_bound = approximate_root
            .checked_mul(&one_plus_epsilon)
            .unwrap()
            .checked_pow(2)
            .unwrap();
        (lower_bound, upper_bound)
    }

    fn compare_pn_fixed(i: u128) -> f64 {
        let pn = PreciseNumber {
            value: InnerUint::from(i),
        };

        let pn_sqrt = pn.sqrt().unwrap();

        let fx_one = BigDecimal::from_str("1000000000000").unwrap(); // 1e12
        // need to convert via string as BigDecimal::from_u128 does not work
        let fx_x = BigDecimal::from_str(&i.to_string())
            .unwrap_or_else(|_| panic!("convert from {}", i))
            / fx_one.clone();
        let fx_sqrt = fx_x.sqrt().unwrap();
        let fx_sqrt_pn = fx_sqrt;

        let pn_sqrt_as_fixed = BigDecimal::from_str(&format!("{}", pn_sqrt.value.as_u128()))
            .unwrap()
            .div(fx_one);

        let float_sqrt_pn = fx_sqrt_pn.to_f64().unwrap();
        (fx_sqrt_pn - pn_sqrt_as_fixed).abs().to_f64().unwrap() / float_sqrt_pn
    }


    // BigDecimal will adjust the scale dynamically up to MAX_SCALE=100
    #[test]
    fn test_fixed_vs_pn() {
        let small_values = (1_000..2_000).step_by(100);
        let around2 = (1_800_000_000_000u128..2_200_000_000_000u128).step_by(10_000_000_000);
        let large_values = ((u128::MAX - 1_000_000)..u128::MAX).step_by(10_000);

        // i is in scaled by 1e12
        for i in small_values.chain(around2).chain(large_values) {
            let diff = compare_pn_fixed(i);
            assert!(
                diff < 0.001,
                "Difference ({}) is greater than epsilon for i={}",
                diff,
                i
            );

            let radicand = PreciseNumber {
                value: InnerUint::from(i),
            };
            let approximate_root = radicand.sqrt_cordic(PreciseNumber::NUM_BITS).unwrap();
            let (lower_bound, upper_bound) = calc_square_root_bounds(&approximate_root);
            assert!(radicand.less_than_or_equal(&upper_bound));
            assert!(radicand.greater_than_or_equal(&lower_bound));
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 10_000,
            timeout: 30,
            ..ProptestConfig::default()
        })]

        #[test]
        fn test_square_root(a in 0..u128::MAX) {
            let a = PreciseNumber { value: InnerUint::from(a) };
            check_square_root(&a);
        }

        #[test]
        fn test_newton_vs_cordic_vs_generic(a in 0..u128::MAX) {
            let a = PreciseNumber { value: InnerUint::from(a) };
            let two = PreciseNumber::new(2).unwrap();
            let guess = a.checked_add(&PreciseNumber::one()).unwrap().checked_div(&two).unwrap();
            let generic_version = a.newtonian_root_approximation_generic(&two, guess, 100).unwrap();
            let newton2_version = a.newtonian_root_approximation_fast(guess, 100).unwrap();
            let cordic2_version = a.cordic_root_approximation_fast(PreciseNumber::NUM_BITS).unwrap();

            assert!(newton2_version.value.abs_diff(generic_version.value).as_u128() < 10,
                "a={}, generic_version={}, newton2_version={}", a.value.as_u128(), generic_version.value.as_u128(), newton2_version.value.as_u128());
            assert!(cordic2_version.value.abs_diff(newton2_version.value).as_u128() < 10,
                "a={}, cordic2_version={}, newton2_version={}", a.value.as_u128(), cordic2_version.value.as_u128(), newton2_version.value.as_u128());

        }

        #[test]
        fn test_cordic_optimized_vs_naive(a in 0..u128::MAX) {
            let a = PreciseNumber { value: InnerUint::from(a) };
            let cordic_version = a.cordic_root_approximation_fast(PreciseNumber::NUM_BITS);
            let cordic_naiv_version = a.cordic_root_approximation_naiv();

            assert_eq!(cordic_version, cordic_naiv_version);
        }
    }

    // returns 10**(-digits) in InnerUint
    // for testing only, neither fast not beautiful
    pub fn precision(digits: u8) -> InnerUint {
        let ten = InnerUint::from(10);
        let mut result = ONE_CONST;
        for _ in 0..digits {
            result = result.checked_div(ten).unwrap();
        }
        assert!(!result.is_zero(), "precision underflow");
        result
    }

}