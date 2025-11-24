
#[cfg(test)]
mod tests {
    use std::num::FpCategory;
    use num_traits::{ToPrimitive, Zero};
    use proptest::proptest;
    use crate::define_precise_number;
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
        let combined = PreciseNumber::try_from(value).unwrap();

        // 3000000000000.123535156250
        assert_eq!(combined.value.as_u128(), 3000000000000123535156250);
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


        println!("best supported: {}", max_supported);
        let u256 = u256_from_f64_bits(max_supported).unwrap();
        println!("best supported: {:?}", u256.0);
        // note: bit 53 is implicit from the mantissa interpretation as 1.xxxxx
        assert_eq!(u256.0, [0, 0, 0, 0xffff_ffff_ffff_f800]);


        let exponentplus1 = exponent + 1;
        let overflow_value2 = f64::from_bits((bits & !MAN_MASK + 1) | (exponentplus1 << 52 & EXP_MASK));

        let overflow_value = max_supported + 1.2855504354071922204335696738729300820177623950262342682411008e61;
        // bits: 0100111111110000000000000000000000000000000000000000000000000000
        assert_eq!(overflow_value2, overflow_value);

        println!("overflow value: {}", overflow_value);
        println!("overflow value: {:064b}", overflow_value.to_bits());
        assert_eq!(u256_from_f64_bits(overflow_value), None);

    }



    #[test]
    fn test_u256_from_f64_one() {
        let one: f64 = 1.0;
        let u256 = u256_from_f64_bits(one).unwrap();
        assert_eq!(u256.0, [1, 0, 0, 0]);
    }

    #[test]
    fn test_u256_from_f64_min() {

        // TODO everyhting smaller than 1.0e52 needs special handing

        // 4.503.599.627.370.496 =
        // let min_value: f64 = 4_503_599_627_370_496u128 as f64;
        // let min_value: f64 = 3.7921553222237964e3;
        let min_value: f64 = 3.7921553222237964e-231;

        println!("min value: {}", min_value);
        println!("min value: {:064b}", min_value.to_bits());

        let u256 = u256_from_f64_bits(min_value).unwrap();

        // assert_eq!(u256.0, [1, 0, 0, 0]);
    }



    #[test]
    fn test_u256_from_f64_zero() {

        let min_value = 0.0f64;

        println!("min value: {}", min_value);
        println!("min value: {:064b}", min_value.to_bits());

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
        println!("subnormal: {}", subnormal);

        assert_eq!(u256_from_f64_bits(subnormal), U256::zero().into());
    }


    // will take the truncated value
    fn u256_from_f64_bits(value: f64) -> Option<U256> {

        if value.is_sign_negative() && !value.is_zero() {
            return None;
        }

        // FIXME
        let value = value.trunc();


        const EXP_MASK: u64 = 0x7ff0_0000_0000_0000;
        const MAN_MASK: u64 = 0x000f_ffff_ffff_ffff;

        // 1.111111111 (binary) * 2^-2 = 0.3 (decimal)
        // let value: f64 = 1048576f64; // 2^20

        match value.classify() {
            FpCategory::Nan => return None,
            FpCategory::Infinite => return None,
            FpCategory::Zero => return U256::zero().into(),
            // subnormal numbers not supported
            FpCategory::Subnormal => {
                println!("subnormal not supported");
                return None;

            },
            FpCategory::Normal => {}
        }






        let bits = value.to_bits();

        let mantissa: u64 = bits & MAN_MASK;
        let exponent: i32 = ((bits & EXP_MASK) >> 52) as i32 - 1023;

        let mantissa_value = mantissa | (1u64 << 52);
        // bits 0..52
        // shift right by 52 and left by exponent
        // e.g. exponent 20 -> bit 20..72
        let bit_range_start = exponent - 52;
        let lower_block = (1024 + bit_range_start) / 64 - 16;
        let upper_block = lower_block + 1;

        println!("bit_range_start: {}", bit_range_start);
        println!("lower_block: {}", lower_block);
        println!("upper_block: {}", upper_block);

        if lower_block > 3 {
            println!("overflow lower block");
            return None;
        }

        if upper_block < 0 {
            println!("underflow upper block");
            return None;
        }

        assert!(lower_block >= -1 && lower_block <= 3);

        println!("value: {}", value);
        println!("bits: {:064b}", bits);
        println!("mantissa: (1.){:052b}", mantissa);
        println!("mantissa_value: {}", mantissa_value);
        println!("mantissa: {}", mantissa as u64);
        println!("exponent: {}", exponent);

        let lower_shift = (bit_range_start + 1024) % 64; // add 1024 to avoid negative modulo
        let upper_shift = 64 - lower_shift;


        println!("lower_shift: {}", lower_shift);
        println!("upper_shift: {}", upper_shift);

        //                           v--- bit_range_start
        // ...................xxxxxxxx.....
        // 33333333222222221111111100000000
        let (lower, _) = mantissa_value.overflowing_shl(lower_shift as u32);
        let (upper, _) = mantissa_value.overflowing_shr(upper_shift as u32);


        println!("lower: {:064b}", lower);
        println!("upper: {:064b}", upper);


        let u256 = match lower_block {
            -1 => {
                if lower == 0 {
                    U256([upper, 0, 0, 0])
                } else {
                    println!("underflow lower block");
                    // TODO check if that's what we want
                    U256([upper, 0, 0, 0])
                    // return None;
                }
            },
            0 => U256([lower, upper, 0, 0]),
            1 => U256([0, lower, upper, 0]),
            2 => U256([0, 0, lower, upper]),
            3 => {
                if upper == 0 {
                    U256([0, 0, 0, lower])
                } else {
                    println!("overflow upper block");
                    return None;
                }
            },
            _ => {
                println!("overflow lower block index");
                return None;
            }
        };



        // shift to right position and project on 2 of the 4 u64s in U256
        // TODO add 2 more blocks


        // 52 bits
        // mantissa: (1.)0000000000001111111111111111111111111111111111111111111111111111

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