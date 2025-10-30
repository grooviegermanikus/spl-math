#[cfg(test)]
mod tests {
    use crate::uint::{U256, U512};
    use crate::{define_precise_number};
    use proptest::prelude::*;

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
        U256::zero(),
        ROUNDING_CORRECTION,
        PRECISION,
        MAXIMUM_SQRT_BASE
    );

    define_precise_number!(TestPreciseNumber8, u8, u8, 10u8, 0u8, 5u8, 1u8, 10u8);
    define_precise_number!(
        TestPreciseNumber32,
        u32,
        u32,
        1_000u32,
        0u32,
        500u32,
        1u32,
        1_000u32
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
    fn test_extend_precision() {
        let u256: U256 = U256::from(1_000_000_000_000u128);
        // cast to U512
        let bytes: &[u64] = u256.as_ref();
        let len = bytes.len();
        let dlen = len * 2;
        // extend to 8 bytes:
        let mut bytes8 = Vec::with_capacity(dlen);
        bytes8.extend_from_slice(bytes);
        bytes8.resize(dlen, 0);
        let u512 = U512(bytes8.try_into().unwrap());
        assert_eq!(u512.as_u128(), u256.as_u128());
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

        // square root
        let test = PreciseNumber::new(9).unwrap();
        let nth_root = PreciseNumber::new(2).unwrap();
        let guess = test.checked_div(&nth_root).unwrap();
        let root = test
            .newtonian_root_approximation(
                &nth_root,
                guess,
                PreciseNumber::MAX_APPROXIMATION_ITERATIONS,
            )
            .unwrap()
            .to_imprecise()
            .unwrap();
        assert_eq!(root, 3); // actually 3

        let test = PreciseNumber::new(101).unwrap();
        let nth_root = PreciseNumber::new(2).unwrap();
        let guess = test.checked_div(&nth_root).unwrap();
        let root = test
            .newtonian_root_approximation(
                &nth_root,
                guess,
                PreciseNumber::MAX_APPROXIMATION_ITERATIONS,
            )
            .unwrap()
            .to_imprecise()
            .unwrap();
        assert_eq!(root, 10); // actually 10.049875

        let test = PreciseNumber::new(1_000_000_000).unwrap();
        let nth_root = PreciseNumber::new(2).unwrap();
        let guess = test.checked_div(&nth_root).unwrap();
        let root = test
            .newtonian_root_approximation(
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
            .newtonian_root_approximation(
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
        let epsilon = PreciseNumber {
            value: InnerUint::from(10),
        }; // correct within 11 decimals
        let one = PreciseNumber::one();
        let one_plus_epsilon = one.checked_add(&epsilon).unwrap();
        let one_minus_epsilon = one.checked_sub(&epsilon).unwrap();
        let approximate_root = check.sqrt().unwrap();
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
        assert!(check.less_than_or_equal(&upper_bound));
        assert!(check.greater_than_or_equal(&lower_bound));
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

    #[test]
    fn test_overflow_u256() {
        let ten = U256::from_dec_str("10").unwrap();
        let a = ten.pow(U256::from(50u32));
        let b = ten.pow(U256::from(50u32));
        // u256 overflows at 1e77
        assert_eq!(a.checked_mul(b), None);
    }

    proptest! {
        #[test]
        fn test_square_root(a in 0..u128::MAX) {
            let a = PreciseNumber { value: InnerUint::from(a) };
            check_square_root(&a);
        }
    }
}
