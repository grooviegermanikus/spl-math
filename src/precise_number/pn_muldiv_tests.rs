#[cfg(test)]
mod tests {
    use crate::{define_muldiv, define_precise_number};
    use num_traits::ToPrimitive;
    use proptest::prelude::*;
    use proptest::test_runner::TestCaseResult;

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

    define_muldiv!(TestPreciseNumber8, u8, u8, u16);

    #[test]
    fn test_call_muldiv_floor() {
        let a = TestPreciseNumber8 { value: 100 };
        let b = TestPreciseNumber8 { value: 50 };
        let c = TestPreciseNumber8 { value: 25 };

        // (10 * 5) / 2 = 25
        let result = a.mul_div_floor(b, c).unwrap();
        assert_eq!(result.value, 200);
    }

    #[test]
    fn test_call_muldiv_ceil() {
        let a = TestPreciseNumber8 { value: 100 };
        let b = TestPreciseNumber8 { value: 50 };
        let c = TestPreciseNumber8 { value: 33 };

        let result = a.mul_div_ceil_naive(b, c).unwrap();
        assert_eq!(result.value, 152);
    }

    // more tests for ceil edge cases
    #[test]
    fn test_mul_div_ceil_edge_cases() {
        let a = TestPreciseNumber8 { value: 10 };
        let b = TestPreciseNumber8 { value: 10 };

        // exact division
        let c_exact = TestPreciseNumber8 { value: 5 };
        let r_exact = a.mul_div_ceil_naive(b, c_exact).unwrap();
        assert_eq!(r_exact.value, 20);

        // inexact division
        let c_inexact = TestPreciseNumber8 { value: 6 };
        let r_inexact = a.mul_div_ceil_naive(b, c_inexact).unwrap();
        assert_eq!(r_inexact.value, 17); // (10*10 + 5)/6 = 16.66.. -> 17

        // another inexact division
        let c_inexact2 = TestPreciseNumber8 { value: 7 };
        let r_inexact2 = a.mul_div_ceil_naive(b, c_inexact2).unwrap();
        assert_eq!(r_inexact2.value, 15); // (10*10 + 6)/7 = 14.28.. -> 15
    }

    proptest! {
        #[test]
        fn test_check_mul_div(a: u8, b: u8, c in 0..u8::MAX) {
            let aa = TestPreciseNumber8 { value: a };
            let bb = TestPreciseNumber8 { value: b };
            let cc = TestPreciseNumber8 { value: c };
            let r = aa.mul_div_floor(bb, cc);

            let expected = aa.mul_div_floor_naive(bb, cc);
            assert_eq!(r, expected);
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
            let r_ceil = aa.mul_div_ceil_naive(bb, denom);
            let r_floor = aa.mul_div_floor_naive(bb, denom);
            let (Some(ceil_val), Some(floor_val)) = (r_ceil, r_floor) else {
                // don't care about overflow cases
                return TestCaseResult::Ok(());
            };

            assert!(floor_val.value <= ceil_val.value);
            if floor_val.value == ceil_val.value {
                // exact division
                // floor(x) <= x <= ceil(x)
                let raw_muldiv = (aa.value as u32 * bb.value as u32) / denom.value as u32;
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
