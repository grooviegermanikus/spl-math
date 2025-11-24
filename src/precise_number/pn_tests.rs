#[cfg(test)]
mod tests {
    use std::num::FpCategory;
    use std::ops::Shl;
    use crate::define_precise_number;
    use crate::uint::U256;
    use num_traits::ToPrimitive;
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
        1e12f64,
        U256::zero(),
        ROUNDING_CORRECTION,
        PRECISION,
        MAXIMUM_SQRT_BASE,
        |value| value.to_u128()
    );

    define_precise_number!(TestPreciseNumber8, u8, u8, 10u8, 1e1f64, 0u8, 5u8, 1u8, 10u8, |value| value.to_u8());
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
        let one_tenth = PreciseNumber::new(1).unwrap()
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
    fn test_pn_from_f64() {
        let a = TestPreciseNumber8::try_from(12.3f64).unwrap();
        assert_eq!(a.value, 123);

        let b = TestPreciseNumber8::try_from(0.1f64).unwrap();
        assert_eq!(b.value, 1);

        let c = TestPreciseNumber8::try_from(0.07f64).unwrap();
        assert_eq!(c.value, 0);

        assert!(TestPreciseNumber8::try_from(25.5f64).is_ok());
        assert!(TestPreciseNumber8::try_from(25.59f64).is_ok());
        assert!(TestPreciseNumber8::try_from(25.6f64).is_err());

        assert!(TestPreciseNumber8::try_from(-1.0f64).is_err());
    }

    #[test]
    fn test_u256_from_f64() {
        // 340282366920938463463374607431768211455 = 3.4e38
        let value: f64 = 3e12 + 0.123456789;
        let combined = crate::precise_number::pn_tests::tests::PreciseNumber::try_from(value).unwrap();

        // 3000000000000.123535156250
        assert_eq!(combined.value.as_u128(), 3000000000000123535156250);
    }

    #[test]
    fn test_u256_from_f64_bits() {

        // 2^20
        let u256 = u256_from_f64_bits(1048576f64);
        assert_eq!(u256.unwrap().as_u128(), 1048576u128);
    }

    #[test]
    fn test_u256_from_f64_bits_zero() {
        let u256 = u256_from_f64_bits(0.0);
        assert_eq!(u256.unwrap().as_u128(), 0u128);
    }

    fn u256_from_f64_bits(value: f64) -> Option<U256> {

        const EXP_MASK: u64 = 0x7ff0_0000_0000_0000;
        const MAN_MASK: u64 = 0x000f_ffff_ffff_ffff;

        // 1.111111111 (binary) * 2^-2 = 0.3 (decimal)
        // let value: f64 = 1048576f64; // 2^20

        match value.classify() {
            FpCategory::Nan => return None,
            FpCategory::Infinite => return None,
            FpCategory::Zero => return U256::zero().into(),
            FpCategory::Subnormal => {}
            FpCategory::Normal => {}
        }

        let mut bits = value.to_bits();
        // bits = bits | MAN_MASK;
        // let value = f64::from_bits(bits);

        let mantissa: u64 = bits & MAN_MASK;
        let exponent: i32 = ((bits & EXP_MASK) >> 52) as i32 - 1023;

        let mantissa_value = mantissa | (1u64 << 52);
        // bits 0..52
        // shift right by 52 and left by exponent
        // e.g. exponent 20 -> bit 20..72
        let bit_range_start = exponent - 52;
        let lower_block = bit_range_start / 64;
        let upper_block = lower_block + 1;
        let lower_shift = (bit_range_start + 256) % 64;
        let upper_shift = 64 - lower_shift;
        let (lower, _) = mantissa_value.overflowing_shr(lower_shift as u32);
        let (upper, _) = mantissa_value.overflowing_shl(upper_shift as u32);

        println!("bit_range_start: {}", bit_range_start);
        println!("lower_block: {}", lower_block);
        println!("upper_block: {}", upper_block);
        println!("lower_shift: {}", lower_shift);
        println!("upper_shift: {}", upper_shift);
        println!("lower: {:064b}", lower);
        println!("upper: {:064b}", upper);



        // shift to right position and project on 2 of the 4 u64s in U256
        // TODO add 2 more blocks
        let u256 = U256([lower, upper, 0, 0]);

        // 52 bits
        // mantissa: (1.)0000000000001111111111111111111111111111111111111111111111111111
        println!("value: {}", value);
        println!("bits: {:064b}", bits);
        println!("mantissa: (1.){:064b}", mantissa);
        println!("mantissa_value: {}", mantissa_value);
        println!("mantissa: {}", mantissa as u64);
        println!("exponent: {}", exponent);
        println!("u256: {:?}", u256);



        // value.classify();
        // u64::MAX
        // f64::MANTISSA_DIGITS
        // value.to_u64()
        // let bytes = [0u64; 4];
        // U256::(bytes);

        Some(u256)
    }


    #[test]
    fn test_from_f64() {
        let pn = TestPreciseNumber8::try_from(12.3f64).unwrap();
        assert_eq!(pn.value, 123);
    }

    proptest! {
        #[test]
        fn test_square_root(a in 0..u128::MAX) {
            let a = PreciseNumber { value: InnerUint::from(a) };
            check_square_root(&a);
        }

         #[test]
        fn test_f64(value: f64) { // TODO
            crate::precise_number::pn_tests::tests::PreciseNumber::try_from(value).unwrap();
        }
    }
}
