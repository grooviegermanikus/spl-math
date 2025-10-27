#[cfg(test)]
mod tests {
    use proptest::test_runner::TestCaseResult;
    use {proptest::prelude::*};
    use crate::{define_muldiv, define_precise_number};

    define_precise_number!(TestPreciseNumber8, u8, u8, 10u8, 0u8, 5u8, 1u8, 10u8);
    define_muldiv!(TestPreciseNumber8, u8, u8, u16);


    #[test]
    fn test_call_muldiv_floor() {
        let a = TestPreciseNumber8 { value: 100 };
        let b = TestPreciseNumber8 { value: 50 };
        let c = TestPreciseNumber8 { value: 25 };

        // (10 * 5) / 2 = 25
        let result = mul_div_floor_hacking(a, b, c).unwrap();
        assert_eq!(result.value, 200);
    }

    #[test]
    fn test_call_muldiv_ceil() {
        let a = TestPreciseNumber8 { value: 100 };
        let b = TestPreciseNumber8 { value: 50 };
        let c = TestPreciseNumber8 { value: 33 };

        let result = mul_div_ceil_naiv(a, b, c).unwrap();
        assert_eq!(result.value, 152);
    }

    fn mul_div_floor_hacking(base: TestPreciseNumber8, num: TestPreciseNumber8, denom: TestPreciseNumber8) -> Option<TestPreciseNumber8> {
        if denom.value == 0 {
            return None;
        }
        if base.value.leading_zeros() + num.value.leading_zeros() >= u8::BITS {
            // small number, no overflow
            let r = base.value.checked_mul(num.value).expect("no overflow")
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
        if denom.value == 0 {
            return None;
        }
        let raw_muldiv = base.value as u32 * num.value as u32 / denom.value as u32;
        u8::try_from(raw_muldiv)
            .map(|v| TestPreciseNumber8 { value: v })
            .ok()
    }

    fn mul_div_ceil_naiv(base: TestPreciseNumber8, num: TestPreciseNumber8, denom: TestPreciseNumber8) -> Option<TestPreciseNumber8> {
        if denom.value == 0 {
            return None;
        }
        let raw_muldiv = (base.value as u32 * num.value as u32 + (denom.value as u32 - 1) )/ denom.value as u32; // why u32?
        u8::try_from(raw_muldiv)
            .map(|v| TestPreciseNumber8 { value: v })
            .ok()
    }

    // more tests for ceil edge cases
    #[test]
    fn test_mul_div_ceil_edge_cases() {
        let a = TestPreciseNumber8 { value: 10 };
        let b = TestPreciseNumber8 { value: 10 };

        // exact division
        let c_exact = TestPreciseNumber8 { value: 5 };
        let r_exact = mul_div_ceil_naiv(a.clone(), b.clone(), c_exact).unwrap();
        assert_eq!(r_exact.value, 20);

        // inexact division
        let c_inexact = TestPreciseNumber8 { value: 6 };
        let r_inexact = mul_div_ceil_naiv(a.clone(), b.clone(), c_inexact).unwrap();
        assert_eq!(r_inexact.value, 17); // (10*10 + 5)/6 = 16.66.. -> 17

        // another inexact division
        let c_inexact2 = TestPreciseNumber8 { value: 7 };
        let r_inexact2 = mul_div_ceil_naiv(a.clone(), b.clone(), c_inexact2).unwrap();
        assert_eq!(r_inexact2.value, 15); // (10*10 + 6)/7 = 14.28.. -> 15
    }

    proptest! {
        #[test]
        fn test_check_mul_div(a: u8, b: u8, c in 0..u8::MAX) {
            let aa = TestPreciseNumber8 { value: a };
            let bb = TestPreciseNumber8 { value: b };
            let cc = TestPreciseNumber8 { value: c };
            let r = mul_div_floor_hacking(aa.clone(), bb.clone(), cc.clone());

            let expected = mul_div_floor_naiv(aa, bb, cc);
            assert_eq!(r, expected);
            // 128, b = 222, c = 1 OVERFLOW
        }

            // ceil(x) = floor(x + denom)
            // ....x... 10 ........ 20
            // floor(x) <= x <= ceil(x)
            // x == floor(x) <=> x == ceil(x)
        #[test]
        fn test_check_mul_div_invariants(a: u8, b: u8, c in 1..u8::MAX) {
            let aa = TestPreciseNumber8 { value: a };
            let bb = TestPreciseNumber8 { value: b };
            let denom = TestPreciseNumber8 { value: c };
            let r_ceil = mul_div_ceil_naiv(aa.clone(), bb.clone(), denom.clone());
            let r_floor = mul_div_floor_naiv(aa.clone(), bb.clone(), denom.clone());
            let (Some(ceil_val), Some(floor_val)) = (r_ceil, r_floor) else {
                // don't care about overflow cases
                return TestCaseResult::Ok(());
            };

            assert!(floor_val.value <= ceil_val.value);
            if floor_val.value == ceil_val.value {
                // exact division
                // floor(x) <= x <= ceil(x)
                let raw_muldiv = (aa.value as u32 * bb.value as u32) / denom.value as u32; // TODO u16
                assert_eq!(floor_val.value as u32, raw_muldiv);
            } else {
                // not exact division
                // floor(x) < x < ceil(x)
                let raw_muldiv = (aa.value as u32 * bb.value as u32) / denom.value as u32;
                assert_eq!(floor_val.value as u32, raw_muldiv);
                assert_eq!(ceil_val.value as u32, raw_muldiv + 1);
            }

        }
    }
}