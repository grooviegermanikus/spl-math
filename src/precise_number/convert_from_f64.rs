use num_traits::Zero;
use crate::uint::U256;

// Converts from the integer part of f64 to U256, returns None on overflow or negative input
pub(crate) fn u256_from_f64_bits(value: f64) -> Option<U256> {
    use std::num::FpCategory;

    if value.is_sign_negative() && !value.is_zero() {
        return None;
    }

    match value.classify() {
        FpCategory::Nan => return None,
        FpCategory::Infinite => return None,
        FpCategory::Zero => return Some(U256::zero()),
        FpCategory::Subnormal => return Some(U256::zero()),
        FpCategory::Normal => {}
    }

    if value < 1.0 {
        return Some(U256::zero());
    }

    const EXP_MASK: u64 = 0x7ff0_0000_0000_0000;
    const MAN_MASK: u64 = 0x000f_ffff_ffff_ffff;
    // bias - see https://en.wikipedia.org/wiki/IEEE_754
    const EXP_BIAS: i32 = 1023;

    let bits = value.to_bits();
    // exponent ranges from -1022 to 1023 (0/-1023 has special meaning)
    let exponent: i32 = (((bits & EXP_MASK) >> 52) as i32) - EXP_BIAS;
    let mantissa = (1u64 << 52) | (bits & MAN_MASK); // 53-bit value

    // bit_range_start is the position of the lowest bit of the mantissa in the final U256
    // we are exploiting the fact that exponent is base2
    let bit_range_start = exponent - 52; // may be negative

    if bit_range_start >= 0 {
        // highest bit (inclusive)
        let bit_range_end = bit_range_start as usize + 52usize;
        if bit_range_end >= 256 {
            // overflow highest bit
            return None;
        }

        let first_word = (bit_range_start as usize) / 64;
        let second_word = first_word + 1;
        let offset_in_word = (bit_range_start as usize) % 64;

        if first_word > 3 {
            // overflow highest word
            return None;
        }

        // shift the 53-bit mantissa which might span two words
        let mantissa_shifted = (mantissa as u128) << offset_in_word;
        let low_mantissa_bits = mantissa_shifted as u64;
        let high_mantissa_bits = (mantissa_shifted >> 64) as u64;

        let mut out = [0u64; 4];
        out[first_word] = low_mantissa_bits;

        if second_word <= 3 {
            out[second_word] = high_mantissa_bits;
        } else if high_mantissa_bits != 0 {
            // high would spill past the highest word
            return None;
        }

        Some(U256(out))
    } else {
        // right shift the mantissa will never use more than the lowest word
        let rs = (-bit_range_start) as u32;
        if rs >= 64 {
            // mantissa is 53 bits; shifting >=64 clears it
            return Some(U256::zero());
        }
        let shifted = mantissa >> rs;
        Some(U256([shifted, 0, 0, 0]))
    }
}

#[cfg(test)]
mod tests {
    use num_traits::{ToPrimitive};
    use proptest::proptest;
    use crate::define_precise_number;
    use crate::precise_number::convert_from_f64::{u256_from_f64_bits};
    use crate::precise_number::PreciseNumber;
    use crate::uint::U256;
    
    define_precise_number!(TestPreciseNumber8, u8, u8, 10u8, 1e1f64, 0u8, 5u8, 1u8, 10u8, |value| value.to_u8());

    #[test]
    fn test_u256_small() {
        // U256 is little-endian
        assert_eq!(U256([1, 0, 0, 0]).as_u128(), 1u128);
    }

    #[test]
    fn test_pn_from_f64() {
        let a = TestPreciseNumber8::new_from_f64(12.3f64).unwrap();
        assert_eq!(a.value, 123);

        let b = TestPreciseNumber8::new_from_f64(0.1f64).unwrap();
        assert_eq!(b.value, 1);

        let c = TestPreciseNumber8::new_from_f64(0.07f64).unwrap();
        assert_eq!(c.value, 0);

        assert!(TestPreciseNumber8::new_from_f64(25.5f64).is_some());
        assert!(TestPreciseNumber8::new_from_f64(25.59f64).is_some());
        assert!(TestPreciseNumber8::new_from_f64(25.6f64).is_none());

        assert!(TestPreciseNumber8::new_from_f64(-1.0f64).is_none());
    }

    #[test]
    fn test_pn_raw_from_f64() {
        let a = TestPreciseNumber8::new_from_inner_f64(12.3f64).unwrap();
        assert_eq!(a.value, 12);

        let b = TestPreciseNumber8::new_from_inner_f64(0.1f64).unwrap();
        assert_eq!(b.value, 0);

        let c = TestPreciseNumber8::new_from_inner_f64(0.9f64).unwrap();
        assert_eq!(c.value, 0);

        assert!(TestPreciseNumber8::new_from_inner_f64(255.9f64).is_some());
        // assert!(TestPreciseNumber8::new_from_inner_f64(256.0f64).is_err());
    }

    #[test]
    fn test_u256_from_f64() {
        let value: f64 = 3e12 + 0.123456789;
        let combined = PreciseNumber::new_from_f64(value).unwrap();

        // test is vague and should be rewritten
        assert_eq!(combined.value.as_u128(), 3000000000000123429978112);
    }

    #[test]
    fn test_u256_from_f64_block0() {
        // will underflow
        let u256 = u256_from_f64_bits(2f64.powi(20));
        assert_eq!(u256.unwrap().as_u128(), 2u128.pow(20));
    }

    #[test]
    fn test_u256_from_f64_block0and1() {
        let u256 = u256_from_f64_bits(2f64.powi(80));
        assert_eq!(u256.unwrap().as_u128(), 2u128.pow(80));
    }

    #[test]
    fn test_u256_from_f64_max() {

        let bits = 0f64.to_bits();
        // = largest mantissa
        const MAN_MASK: u64 = 0x000f_ffff_ffff_ffff;
        const EXP_MASK: u64 = 0x7ff0_0000_0000_0000;

        let exponent = 255+1023;
        // max exponent is largest minus 1 = (2^11-1) - 1 = 2046
        let max_supported = f64::from_bits(bits | MAN_MASK | (exponent << 52 & EXP_MASK));
        // https://float.exposed/0x4fefffffffffffff
        assert_eq!(max_supported, 1.15792089237316182568e+77);
        // bits: 0100111111101111111111111111111111111111111111111111111111111111
        // mantissa: (1.)1111111111111111111111111111111111111111111111111111


        let u256 = u256_from_f64_bits(max_supported).unwrap();
        // note: bit 53 is implicit from the mantissa interpretation as 1.xxxxx
        assert_eq!(u256.0, [0, 0, 0, 0xffff_ffff_ffff_f800]);


        let exponentplus1 = exponent + 1;
        let overflow_value2 = f64::from_bits((bits & !MAN_MASK + 1) | (exponentplus1 << 52 & EXP_MASK));

        let overflow_value = max_supported + 1.2855504354071922204335696738729300820177623950262342682411008e61;
        // bits: 0100111111110000000000000000000000000000000000000000000000000000
        assert_eq!(overflow_value2, overflow_value);

        assert_eq!(u256_from_f64_bits(overflow_value), None);

    }



    #[test]
    fn test_u256_from_f64_one() {
        let one: f64 = 1.0;
        let u256 = u256_from_f64_bits(one).unwrap();
        assert_eq!(u256.0, [1, 0, 0, 0]);
    }

    #[test]
    fn test_u256_from_min() {

        let min_value: f64 = 3.7921553222237964e-231;

        let u256 = u256_from_f64_bits(min_value).unwrap();

        assert_eq!(u256.0, [0, 0, 0, 0]);
    }

    #[test]
    fn test_u256_from_f64_min() {

        let min_value: f64 = f64::MIN_POSITIVE;

        let u256 = u256_from_f64_bits(min_value).unwrap();

        assert_eq!(u256.0, [0, 0, 0, 0]);
    }

    #[test]
    fn test_u256_from_negative_f64() {

        let neg_value: f64 = -42.0;

        let u256 = u256_from_f64_bits(neg_value);

        assert_eq!(u256, None);
    }

    #[test]
    fn test_u256_from_negative_zer0() {
        let u256 = u256_from_f64_bits(-0.0).unwrap();

        assert_eq!(u256.0, [0, 0, 0, 0]);
    }



    #[test]
    fn test_u256_from_f64_zero() {

        let min_value = 0.0f64;

        let u256 = u256_from_f64_bits(min_value).unwrap();

        assert_eq!(u256.0, [0, 0, 0, 0]);

        let underflow_value = -1e-100;
        assert_eq!(u256_from_f64_bits(underflow_value), None);
    }

    #[test]
    fn test_u256_from_f64_block3() {
        // 2^256 => 1.15e77
        // U256 is little-endian
        // not that the mantissa is only 52 bits and fits in the highest block
        assert_eq!(u256_from_f64_bits(1.15e77).unwrap().0, [0, 0, 0, 18320556978023200768]);
    }

    #[test]
    fn test_u256_from_f64_block2and3() {
        // U256 is little-endian
        assert_eq!(u256_from_f64_bits(2.0f64.powi(222) * 1.1111).unwrap().0, [0, 0, 11923974904812142592, 1193034540]);
    }


    #[test]
    fn test_u256_from_f64_overflow() {
        // 2^256 => apprx 1.16e77
        assert_eq!(u256_from_f64_bits(1.16e77), None);
    }

    #[test]
    fn test_u256_from_f64_bits_zero() {
        let u256 = u256_from_f64_bits(0.0);
        assert_eq!(u256.unwrap().as_u128(), 0u128);
    }

    #[test]
    fn test_u256_from_f64_bits_subnormal() {
        let bits = 0f64.to_bits();
        const MAN_MASK: u64 = 0x000f_ffff_ffff_ffff;
        let subnormal = f64::from_bits(bits | MAN_MASK);

        // subnormals are very small numbers and are guaranteed to be smaller than one
        assert_eq!(u256_from_f64_bits(subnormal), U256::zero().into());
    }

    #[test]
    fn test_from_f64() {
        let pn = TestPreciseNumber8::new_from_f64(12.3f64).unwrap();
        assert_eq!(pn.value, 123);
    }

    proptest! {

        #[test]
        fn test_truncated_prop(value: f64) { // TODO

            if value >= 0.0 && value < 1.15792089237316182568e+77 {
                let original = u256_from_f64_bits(value).unwrap();
                let truncated = u256_from_f64_bits(value.trunc()).unwrap();
                assert_eq!(original, truncated);
            }

        }

        #[test]
        fn test_u256_from_f64_prop(value: f64) { // TODO

            if value >= 0.0 && value < 1.15792089237316182568e+77 {
                u256_from_f64_bits(value).unwrap();
            }

        }
    }
}