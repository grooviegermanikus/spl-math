#[cfg(test)]
mod tests {
    use crate::define_precise_number;
    use crate::precise_number::convert_from_f64::u256_from_f64_bits;
    use crate::uint::{U256, U512};
    use bigdecimal_rs::BigDecimal;
    use num_traits::ToPrimitive;
    use proptest::prelude::*;
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
    define_precise_number!(
        TestPreciseNumber32,
        u32,
        u32,
        1_000u32,
        1e3f64,
        0u32,
        500u32,
        1u32,
        1_000u32,
        |value| value.to_u32()
    ); // MAXIMUM_SQRT_BASE is likely incorrect

    fn check_pow_approximation(base: InnerUint, exponent: InnerUint, expected: InnerUint) {
        let precision = InnerUint::from(5_000_000); // correct to at least 3 decimal places
        let base = PreciseNumber { value: base };
        let exponent = PreciseNumber { value: exponent };
        let root = base
            .checked_pow_approximation(&exponent, PreciseNumber::MAX_APPROXIMATION_ITERATIONS)
            .unwrap();
        let expected = PreciseNumber { value: expected };
        assert!(root.almost_eq(&expected, precision));
    }

    #[test]
    fn test_max_number_to_u128() {
        // 3.4e38
        // let a = PreciseNumber::new(300282366920938463463374607431768211455).unwrap();
        // let b = PreciseNumber::new(300282366920938463463374607431768211455).unwrap();

        let a = PreciseNumber::new(3.4e32 as u128).unwrap();
        let b = PreciseNumber::new(3.4e32 as u128).unwrap();
        // max 3,4028236692×10³²

        a.checked_mul(&b).unwrap();
    }

    #[test]
    fn test_max_int_val() {
        // 2^32 / 1000 // 4294967296 / 1000 = 4294967.296
        let _ = TestPreciseNumber32::new(4294967);
    }

    #[test]
    fn test_to_imprecise_rounding() {
        fn calc(a: u8, b: u8) -> u8 {
            let a = TestPreciseNumber8::new(a).unwrap();
            // println!("a: {}", a.value);
            let b = TestPreciseNumber8::new(b).unwrap();
            // println!("b: {}", b.value);
            let c = a.checked_div(&b).unwrap();
            // println!("c: {}", c.value);
            c.to_imprecise().unwrap()
        }

        // rounding mode HALF_DOWN
        assert_eq!(calc(11, 2), 5);
        assert_eq!(calc(5, 2), 2);
        assert_eq!(calc(4, 3), 1);
    }

    #[test]
    fn test_root_approximation() {
        let one = PreciseNumber::FP_ONE;
        // square root
        check_pow_approximation(one / 4, one / 2, one / 2); // 1/2
        check_pow_approximation(one * 11 / 10, one / 2, InnerUint::from(1_048808848161u128)); // 1.048808848161

        // 5th root
        check_pow_approximation(one * 4 / 5, one * 2 / 5, InnerUint::from(914610103850u128));
        // 0.91461010385

        // 10th root
        check_pow_approximation(one / 2, one * 4 / 50, InnerUint::from(946057646730u128));
        // 0.94605764673
    }

    fn check_pow_fraction(
        base: InnerUint,
        exponent: InnerUint,
        expected: InnerUint,
        precision: InnerUint,
    ) {
        let base = PreciseNumber { value: base };
        let exponent = PreciseNumber { value: exponent };
        let power = base.checked_pow_fraction(&exponent).unwrap();
        let expected = PreciseNumber { value: expected };
        assert!(power.almost_eq(&expected, precision));
    }

    #[test]
    fn test_pow_fraction() {
        let one = PreciseNumber::FP_ONE;
        let precision = InnerUint::from(50_000_000); // correct to at least 3 decimal places
        let less_precision = precision * 1_000; // correct to at least 1 decimal place
        check_pow_fraction(one, one, one, precision);
        check_pow_fraction(
            one * 20 / 13,
            one * 50 / 3,
            InnerUint::from(1312_534484739100u128),
            precision,
        ); // 1312.5344847391
        check_pow_fraction(one * 2 / 7, one * 49 / 4, InnerUint::from(2163), precision);
        check_pow_fraction(
            one * 5000 / 5100,
            one / 9,
            InnerUint::from(997802126900u128),
            precision,
        ); // 0.99780212695
           // results get less accurate as the base gets further from 1, so allow
           // for a greater margin of error
        check_pow_fraction(
            one * 2,
            one * 27 / 5,
            InnerUint::from(42_224253144700u128),
            less_precision,
        ); // 42.2242531447
        check_pow_fraction(
            one * 18 / 10,
            one * 11 / 3,
            InnerUint::from(8_629769290500u128),
            less_precision,
        ); // 8.629769290
    }

    #[test]
    fn test_newtonian_approximation() {
        let test = PreciseNumber::new(0).unwrap();
        let nth_root = PreciseNumber::new(0).unwrap();
        let guess = test.checked_div(&nth_root);
        assert_eq!(guess, Option::None);

        // square root 0+1
        let test = PreciseNumber::new(0).unwrap();
        let nth_root = PreciseNumber::new(2).unwrap();
        let guess = test.checked_div(&nth_root).unwrap();
        let root = test
            .newtonian_root_approximation_fast(guess, PreciseNumber::MAX_APPROXIMATION_ITERATIONS)
            .unwrap()
            .to_imprecise()
            .unwrap();
        assert_eq!(root, 0);

        let test = PreciseNumber::new(1).unwrap();
        let nth_root = PreciseNumber::new(2).unwrap();
        let guess = test.checked_div(&nth_root).unwrap();
        let root = test
            .newtonian_root_approximation_fast(guess, PreciseNumber::MAX_APPROXIMATION_ITERATIONS)
            .unwrap()
            .to_imprecise()
            .unwrap();
        assert_eq!(root, 1);

        // square root
        let test = PreciseNumber::new(9).unwrap();
        let nth_root = PreciseNumber::new(2).unwrap();
        let guess = test.checked_div(&nth_root).unwrap();
        let root = test
            .newtonian_root_approximation_fast(guess, PreciseNumber::MAX_APPROXIMATION_ITERATIONS)
            .unwrap()
            .to_imprecise()
            .unwrap();
        assert_eq!(root, 3); // actually 3

        let test = PreciseNumber::new(101).unwrap();
        let guess = test.checked_div(&nth_root).unwrap();
        let root = test
            .newtonian_root_approximation_fast(guess, PreciseNumber::MAX_APPROXIMATION_ITERATIONS)
            .unwrap()
            .to_imprecise()
            .unwrap();
        assert_eq!(root, 10); // actually 10.049875

        let test = PreciseNumber::new(1_000_000_000).unwrap();
        let nth_root = PreciseNumber::new(2).unwrap();
        let guess = test.checked_div(&nth_root).unwrap();
        let root = test
            .newtonian_root_approximation_generic(
                &nth_root,
                guess,
                PreciseNumber::MAX_APPROXIMATION_ITERATIONS,
            )
            .unwrap()
            .to_imprecise()
            .unwrap();
        assert_eq!(root, 31_623); // actually 31622.7766

        // 5th root
        let test = PreciseNumber::new(500).unwrap();
        let nth_root = PreciseNumber::new(5).unwrap();
        let guess = test.checked_div(&nth_root).unwrap();
        let root = test
            .newtonian_root_approximation_generic(
                &nth_root,
                guess,
                PreciseNumber::MAX_APPROXIMATION_ITERATIONS,
            )
            .unwrap()
            .to_imprecise()
            .unwrap();
        assert_eq!(root, 3); // actually 3.46572422
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

    #[test]
    fn test_checked_div() {
        let one_tenth = PreciseNumber::new(1)
            .unwrap()
            .checked_div(&PreciseNumber::new(10).unwrap())
            .unwrap();
        let two = PreciseNumber::new(2).unwrap();
        let c = one_tenth.checked_div(&one_tenth).unwrap();
        let e = PreciseNumber::new(1).unwrap().checked_div(&c).unwrap();
        let d = c.checked_mul(&two).unwrap();
        assert_eq!(e.to_imprecise().unwrap(), 1);
        assert_eq!(d.to_imprecise().unwrap(), 2);
    }

    #[test]
    fn test_checked_mul() {
        let number_one = PreciseNumber::new(0).unwrap();
        let number_two = PreciseNumber::new(0).unwrap();
        let result = number_one.checked_mul(&number_two);
        assert_eq!(
            result,
            Option::Some(PreciseNumber {
                value: U256::from(0)
            })
        );

        let number_one = PreciseNumber::new(2).unwrap();
        let number_two = PreciseNumber::new(2).unwrap();
        let result = number_one.checked_mul(&number_two).unwrap();
        assert_eq!(result, PreciseNumber::new(2 * 2).unwrap());

        let number_one = PreciseNumber { value: U256::MAX };
        let number_two = PreciseNumber::new(1).unwrap();
        let result = number_one.checked_mul(&number_two).unwrap();
        assert_eq!(
            result.value,
            U256::MAX / PreciseNumber::FP_ONE * PreciseNumber::FP_ONE
        );

        let number_one = PreciseNumber { value: U256::MAX };
        let mut number_two = PreciseNumber::new(1).unwrap();
        number_two.value += U256::from(1);
        let result = number_one.checked_mul(&number_two);
        assert_eq!(result, Option::None);
    }

    fn check_square_root(check: &PreciseNumber) {
        let (lower_bound, upper_bound) = calc_square_root_bounds(check);
        assert!(check.less_than_or_equal(&upper_bound));
        assert!(check.greater_than_or_equal(&lower_bound));
    }

    fn calc_square_root_bounds(check: &PreciseNumber) -> (PreciseNumber, PreciseNumber) {
        let epsilon = PreciseNumber {
            value: InnerUint::from(10),
        }; // correct within 11 decimals
        let one = PreciseNumber::one();
        let one_plus_epsilon = one.checked_add(&epsilon).unwrap();
        let one_minus_epsilon = one.checked_sub(&epsilon).unwrap();
        let approximate_root = check.sqrt_cordic(PreciseNumber::NUM_BITS).unwrap();
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

    #[test]
    fn test_floor() {
        let whole_number = PreciseNumber::new(2).unwrap();
        let mut decimal_number = PreciseNumber::new(2).unwrap();
        decimal_number.value += InnerUint::from(1);
        let floor = decimal_number.floor().unwrap();
        let floor_again = floor.floor().unwrap();
        assert_eq!(whole_number.value, floor.value);
        assert_eq!(whole_number.value, floor_again.value);
    }

    #[test]
    fn test_ceiling() {
        // 1.999999999999
        let mut decimal_number = PreciseNumber::new(2).unwrap();
        decimal_number.value -= InnerUint::from(1);
        let ceiling = decimal_number.ceiling().unwrap();
        let ceiling_again = ceiling.ceiling().unwrap();

        let expected_fp2: InnerUint = PreciseNumber::new(2).unwrap().value;
        assert_eq!(ceiling.value, expected_fp2);
        assert_eq!(ceiling_again.value, expected_fp2);
    }

    #[test]
    fn test_ceiling_all() {
        for value in 0..=246 {
            let a = TestPreciseNumber8 { value };

            let ceil_expected = (value as f64 / 10.0).ceil() as u8;
            let ceiling = a.ceiling().unwrap();

            assert_eq!(ceiling.value, ceil_expected * 10);
        }

        for value in 247..=255 {
            let a = TestPreciseNumber8 { value };
            assert!(a.ceiling().is_none(), "will overflow");
        }
    }

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
    fn test_overflow_u256() {
        let ten = U256::from_dec_str("10").unwrap();
        let a = ten.pow(U256::from(50u32));
        let b = ten.pow(U256::from(50u32));
        // u256 overflows at 1e77
        assert_eq!(a.checked_mul(b), None);
    }

    // adopted from token-bonding-curve -> dfs_precise_number.rs
    #[test]
    fn test_square_root_precision() {
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
        assert!(
            number
                .sqrt_cordic(PreciseNumber::NUM_BITS) // TODO replace speed_factor with something better
                .unwrap()
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
            let (lower_bound, upper_bound) = calc_square_root_bounds(&radicand);
            assert!(radicand.less_than_or_equal(&upper_bound));
            assert!(radicand.greater_than_or_equal(&lower_bound));
        }
    }

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
}
