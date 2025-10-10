
mod tests {
    use crate::{define_muldiv, define_precise_number};

    define_precise_number!(TestPreciseNumber8, u8, u8, 10u8, 0u8, 5u8, 1u8, 10u8);
    define_muldiv!(TestPreciseNumber8, u8, u8, u16);


    #[test]
    fn test_call_muldiv() {
        let a = TestPreciseNumber8 { value: 100 };
        let b = TestPreciseNumber8 { value: 50 };
        let c = TestPreciseNumber8 { value: 25 };

        // (10 * 5) / 2 = 25
        let result = a.mul_div_floor(b, c).unwrap();
        assert_eq!(result.value, 200);
    }

}