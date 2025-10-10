
mod tests {
    use {super::*, proptest::prelude::*};
    use crate::{define_muldiv, define_precise_number};

    define_precise_number!(TestPreciseNumber8, u8, u8, 10u8, 0u8, 5u8, 1u8, 10u8);
    define_muldiv!(TestPreciseNumber8, u8, u8, u16);


    #[test]
    fn test_call_muldiv() {
        let a = TestPreciseNumber8 { value: 100 };
        let b = TestPreciseNumber8 { value: 50 };
        let c = TestPreciseNumber8 { value: 25 };

        // (10 * 5) / 2 = 25
        let result = mul_div_floor_hacking(a, b, c).unwrap();
        assert_eq!(result.value, 200);
    }

    fn mul_div_floor_hacking(base: TestPreciseNumber8, num: TestPreciseNumber8, denom: TestPreciseNumber8) -> Option<TestPreciseNumber8> {
        if denom.value == 0 {
            return None;
        }
        if base.value.leading_zeros() + num.value.leading_zeros() >= u8::BITS {
            // small number, no overflow
            let r = base.value * num.value
                / denom.value;
            Some(TestPreciseNumber8 { value: r })
        } else {
            let r = base.value as u16 * num.value as u16
                / denom.value as u16;
            u8::try_from(r)
                .map(|v| TestPreciseNumber8 { value: v })
                .ok()
        }
    }

    fn mul_div_floor_naiv(base: TestPreciseNumber8, num: TestPreciseNumber8, denom: TestPreciseNumber8) -> Option<TestPreciseNumber8> {
        let raw_muldiv = base.value as u32 * num.value as u32 / denom.value as u32;
        u8::try_from(raw_muldiv)
            .map(|v| TestPreciseNumber8 { value: v })
            .ok()
    }

    proptest! {
        #[test]
        fn test_check_mul_div(a: u8, b: u8, c in 1..u8::MAX) {
            let aa = TestPreciseNumber8 { value: a };
            let bb = TestPreciseNumber8 { value: b };
            let cc = TestPreciseNumber8 { value: c };
            let r = mul_div_floor_hacking(aa.clone(), bb.clone(), cc.clone());

            let expected = mul_div_floor_naiv(aa, bb, cc);
            assert_eq!(r, expected);
            // 128, b = 222, c = 1 OVERFLOW
        }
    }
}