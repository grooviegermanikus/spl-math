#[cfg(test)]
mod tests_pn_256_128_d12 {
    use num_traits::ToPrimitive;
    use crate::define_precise_number;

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
    
}