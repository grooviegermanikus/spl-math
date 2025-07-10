#![allow(clippy::arithmetic_side_effects)]
//! Defines PreciseNumber, a U256 wrapper with float-like operations

use crate::uint::U256;

macro_rules! define_precise_number {
    ($Precise:ident, $TOuter:ty, $FPInner:ty, $FP_ONE:expr, $FP_ZERO:expr, $ROUNDING_CORRECTION:expr, $PRECISION:expr, $MAXIMUM_SQRT_BASE:expr) => {
        /// Struct encapsulating a fixed-point number that allows for decimal
        /// calculations
        #[derive(Clone, Debug, PartialEq)]
        pub struct $Precise {
            /// Wrapper over the inner value, which is multiplied by ONE
            pub(crate) value: $FPInner,
        }

        #[allow(dead_code)]
        impl $Precise {
            const FP_ONE: $FPInner = $FP_ONE;
            const FP_ZERO: $FPInner = $FP_ZERO;

            /// Correction to apply to avoid truncation errors on division.  Since
            /// integer operations will always floor the result, we artificially bump it
            /// up by one half to get the expect result.
            const ROUNDING_CORRECTION: $FPInner = $ROUNDING_CORRECTION;

            /// Desired precision for the correction factor applied during each
            /// iteration of checked_pow_approximation.  Once the correction factor is
            /// smaller than this number, or we reach the maximum number of iterations,
            /// the calculation ends.
            const PRECISION: $FPInner = $PRECISION;

            const MAXIMUM_SQRT_BASE: $FPInner = $MAXIMUM_SQRT_BASE;

            // workaround to be compatible with all types used in tests
            const SMALLEST_POSITIVE: u8 = 1;

            fn zero() -> Self {
                Self {
                    value: Self::FP_ZERO,
                }
            }

            fn one() -> Self {
                Self {
                    value: Self::FP_ONE,
                }
            }

            /// Maximum number iterations to apply on checked_pow_approximation.
            const MAX_APPROXIMATION_ITERATIONS: u32 = 100;

            /// Minimum base (excl) allowed when calculating exponents in checked_pow_fraction
            /// and checked_pow_approximation.  This simply avoids 0 as a base.
            fn min_pow_base_excl() -> $FPInner {
                Self::FP_ZERO
            }

            /// Maximum base allowed when calculating exponents in checked_pow_fraction
            /// and checked_pow_approximation.  The calculation use a Taylor Series
            /// approximation around 1, which converges for bases between 0 and 2.  See
            /// https://en.wikipedia.org/wiki/Binomial_series#Conditions_for_convergence
            /// for more information.
            fn max_pow_base() -> $FPInner {
                Self::FP_ONE + Self::FP_ONE
            }

            /// Create a precise number from an imprecise u128, should always succeed
            pub fn new(int_val: $TOuter) -> Option<Self> {
                let int_value: $FPInner = int_val.into();
                let value: $FPInner = int_value.checked_mul(Self::FP_ONE)?;
                Some(Self { value })
            }

            /// Convert a precise number back to u128
            pub fn to_imprecise(&self) -> Option<$TOuter> {
                self.value
                    .checked_add(Self::ROUNDING_CORRECTION)?
                    .checked_div(Self::FP_ONE)
                    .and_then(|v| <$TOuter>::try_from(v).ok())
            }

            /// Checks that two PreciseNumbers are equal within some tolerance
            pub fn almost_eq(&self, rhs: &Self, precision: $FPInner) -> bool {
                let (difference, _) = self.unsigned_sub(rhs);
                difference.value < precision
            }

            /// Checks that a number is less than another
            pub fn less_than(&self, rhs: &Self) -> bool {
                self.value < rhs.value
            }

            /// Checks that a number is greater than another
            pub fn greater_than(&self, rhs: &Self) -> bool {
                self.value > rhs.value
            }

            /// Checks that a number is less than another
            pub fn less_than_or_equal(&self, rhs: &Self) -> bool {
                self.value <= rhs.value
            }

            /// Checks that a number is greater than another
            pub fn greater_than_or_equal(&self, rhs: &Self) -> bool {
                self.value >= rhs.value
            }

            /// Floors a precise value to a precision of ONE
            pub fn floor(&self) -> Option<Self> {
                let value = self
                    .value
                    .checked_div(Self::FP_ONE)?
                    .checked_mul(Self::FP_ONE)?;
                Some(Self { value })
            }

            /// Ceiling a precise value to a precision of ONE
            pub fn ceiling(&self) -> Option<Self> {
                let value = self
                    .value
                    .checked_add(Self::FP_ONE.checked_sub(Self::SMALLEST_POSITIVE.into())?)?
                    .checked_div(Self::FP_ONE)?
                    .checked_mul(Self::FP_ONE)?;
                Some(Self { value })
            }

            /// Performs a checked division on two precise numbers
            pub fn checked_div(&self, rhs: &Self) -> Option<Self> {
                if *rhs == Self::zero() {
                    return None;
                }
                match self.value.checked_mul(Self::FP_ONE) {
                    Some(v) => {
                        let value = v
                            .checked_add(Self::ROUNDING_CORRECTION)?
                            .checked_div(rhs.value)?;
                        Some(Self { value })
                    }
                    None => {
                        let value = self
                            .value
                            .checked_add(Self::ROUNDING_CORRECTION)?
                            .checked_div(rhs.value)?
                            .checked_mul(Self::FP_ONE)?;
                        Some(Self { value })
                    }
                }
            }

            /// Performs a multiplication on two precise numbers
            pub fn checked_mul(&self, rhs: &Self) -> Option<Self> {
                match self.value.checked_mul(rhs.value) {
                    Some(v) => {
                        let value = v
                            .checked_add(Self::ROUNDING_CORRECTION)?
                            .checked_div(Self::FP_ONE)?;
                        Some(Self { value })
                    }
                    None => {
                        let value = if self.value >= rhs.value {
                            self.value
                                .checked_div(Self::FP_ONE)?
                                .checked_mul(rhs.value)?
                        } else {
                            rhs.value
                                .checked_div(Self::FP_ONE)?
                                .checked_mul(self.value)?
                        };
                        Some(Self { value })
                    }
                }
            }

            /// Performs addition of two precise numbers
            pub fn checked_add(&self, rhs: &Self) -> Option<Self> {
                let value = self.value.checked_add(rhs.value)?;
                Some(Self { value })
            }

            /// Subtracts the argument from self
            pub fn checked_sub(&self, rhs: &Self) -> Option<Self> {
                let value = self.value.checked_sub(rhs.value)?;
                Some(Self { value })
            }

            /// Performs a subtraction, returning the result and whether the result is
            /// negative
            pub fn unsigned_sub(&self, rhs: &Self) -> (Self, bool) {
                match self.value.checked_sub(rhs.value) {
                    None => {
                        let value = rhs.value.checked_sub(self.value).unwrap();
                        (Self { value }, true)
                    }
                    Some(value) => (Self { value }, false),
                }
            }

            /// Performs pow on a precise number
            pub fn checked_pow(&self, exponent: u32) -> Option<Self> {
                // For odd powers, start with a multiplication by base since we halve the
                // exponent at the start
                let value = if exponent.checked_rem(2)? == 0 {
                    Self::FP_ONE
                } else {
                    self.value
                };
                let mut result = Self { value };

                // To minimize the number of operations, we keep squaring the base, and
                // only push to the result on odd exponents, like a binary decomposition
                // of the exponent.
                let mut squared_base = self.clone();
                let mut current_exponent = exponent.checked_div(2)?;
                while current_exponent != 0 {
                    squared_base = squared_base.checked_mul(&squared_base)?;

                    // For odd exponents, "push" the base onto the value
                    if current_exponent.checked_rem(2)? != 0 {
                        result = result.checked_mul(&squared_base)?;
                    }

                    current_exponent = current_exponent.checked_div(2)?;
                }
                Some(result)
            }

            /// Approximate the nth root of a number using a Taylor Series around 1 on
            /// x ^ n, where 0 < n < 1, result is a precise number.
            /// Refine the guess for each term, using:
            ///                                  1                    2
            /// f(x) = f(a) + f'(a) * (x - a) + --- * f''(a) * (x - a)  + ...
            ///                                  2!
            /// For x ^ n, this gives:
            ///  n    n         n-1           1                  n-2        2
            /// x  = a  + n * a    (x - a) + --- * n * (n - 1) a     (x - a)  + ...
            ///                               2!
            ///
            /// More simply, this means refining the term at each iteration with:
            ///
            /// t_k+1 = t_k * (x - a) * (n + 1 - k) / k
            ///
            /// where a = 1, n = power, x = precise_num
            /// NOTE: this function is private because its accurate range and precision
            /// have not been estbalished.
            fn checked_pow_approximation(
                &self,
                exponent: &Self,
                max_iterations: u32,
            ) -> Option<Self> {
                assert!(self.value > Self::min_pow_base_excl());
                assert!(self.value <= Self::max_pow_base());
                let one = Self::one();
                if *exponent == Self::zero() {
                    return Some(one);
                }
                let mut precise_guess = one.clone();
                let mut term = precise_guess.clone();
                let (x_minus_a, x_minus_a_negative) = self.unsigned_sub(&precise_guess);
                let exponent_plus_one = exponent.checked_add(&one)?;
                let mut negative = false;
                let mut k = Self::zero();
                for _ in 1..max_iterations {
                    // start with 1
                    k = k.checked_add(&one)?;
                    let (current_exponent, current_exponent_negative) =
                        exponent_plus_one.unsigned_sub(&k);
                    term = term.checked_mul(&current_exponent)?;
                    term = term.checked_mul(&x_minus_a)?;
                    term = term.checked_div(&k)?;
                    if term.value < Self::PRECISION {
                        break;
                    }
                    if x_minus_a_negative {
                        negative = !negative;
                    }
                    if current_exponent_negative {
                        negative = !negative;
                    }
                    if negative {
                        precise_guess = precise_guess.checked_sub(&term)?;
                    } else {
                        precise_guess = precise_guess.checked_add(&term)?;
                    }
                }
                Some(precise_guess)
            }

            /// Get the power of a number, where the exponent is expressed as a fraction
            /// (numerator / denominator)
            /// NOTE: this function is private because its accurate range and precision
            /// have not been estbalished.
            #[allow(dead_code)]
            fn checked_pow_fraction(&self, exponent: &Self) -> Option<Self> {
                assert!(self.value > Self::min_pow_base_excl());
                assert!(self.value <= Self::max_pow_base());
                let whole_exponent = exponent.floor()?;
                let precise_whole =
                    self.checked_pow(whole_exponent.to_imprecise()?.try_into().ok()?)?;
                let (remainder_exponent, negative) = exponent.unsigned_sub(&whole_exponent);
                assert!(!negative);
                if remainder_exponent.value == Self::FP_ZERO {
                    return Some(precise_whole);
                }
                let precise_remainder = self.checked_pow_approximation(
                    &remainder_exponent,
                    Self::MAX_APPROXIMATION_ITERATIONS,
                )?;
                precise_whole.checked_mul(&precise_remainder)
            }

            /// Approximate the nth root of a number using Newton's method
            /// https://en.wikipedia.org/wiki/Newton%27s_method
            /// NOTE: this function is private because its accurate range and precision
            /// have not been established.
            fn newtonian_root_approximation(
                &self,
                root: &Self,
                mut guess: Self,
                iterations: u32,
            ) -> Option<Self> {
                let zero = Self::zero();
                if *self == zero {
                    return Some(zero);
                }
                if *root == zero {
                    return None;
                }
                let one = Self::one();
                let root_minus_one = root.checked_sub(&one)?;
                let root_minus_one_whole = root_minus_one.to_imprecise()?;
                let mut last_guess = guess.clone();
                for _ in 0..iterations {
                    // x_k+1 = ((n - 1) * x_k + A / (x_k ^ (n - 1))) / n
                    let first_term = root_minus_one.checked_mul(&guess)?;
                    let power = guess.checked_pow(root_minus_one_whole.try_into().ok()?);
                    let second_term = match power {
                        Some(num) => self.checked_div(&num)?,
                        None => Self::zero(),
                    };
                    guess = first_term.checked_add(&second_term)?.checked_div(root)?;
                    if last_guess.almost_eq(&guess, Self::PRECISION) {
                        break;
                    } else {
                        last_guess = guess.clone();
                    }
                }
                Some(guess)
            }

            /// Based on testing around the limits, this base is the smallest value that
            /// provides an epsilon 11 digits
            fn minimum_sqrt_base() -> Self {
                Self::zero()
            }

            /// Based on testing around the limits, this base is the smallest value that
            /// provides an epsilon of 11 digits
            fn maximum_sqrt_base() -> Self {
                Self {
                    value: Self::MAXIMUM_SQRT_BASE,
                }
            }

            /// Approximate the square root using Newton's method.  Based on testing,
            /// this provides a precision of 11 digits for inputs between 0 and
            /// u128::MAX
            pub fn sqrt(&self) -> Option<Self> {
                if self.less_than(&Self::minimum_sqrt_base())
                    || self.greater_than(&Self::maximum_sqrt_base())
                {
                    return None;
                }
                let one = Self::one();
                let two = one.checked_add(&one)?;
                // A good initial guess is the average of the interval that contains the
                // input number.  For all numbers, that will be between 1 and the given number.
                let guess = self.checked_add(&one)?.checked_div(&two)?;
                self.newtonian_root_approximation(&two, guess, Self::MAX_APPROXIMATION_ITERATIONS)
            }
        }
    };
} // -- macro

const ONE_CONST: U256 = U256([1000000000000, 0, 0, 0]);
const ROUNDING_CORRECTION: U256 = U256([1000000000000 / 2, 0, 0, 0]);
const PRECISION: U256 = U256([100, 0, 0, 0]);
const MAXIMUM_SQRT_BASE: U256 = U256([18446743073709551616, 18446744073709551615, 999999999999, 0]); // u128::MAX
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

#[cfg(test)]
mod tests {
    use {super::*, proptest::prelude::*};
    type InnerUint = U256;

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
    fn test_u256_one_constant() {
        let one = U256::from(1_000_000_000_000u128);
        assert_eq!(ONE_CONST, one);
    }

    #[test]
    fn test_u256_rounding_correction_constant() {
        let rounding = U256::from(1_000_000_000_000u128) / 2;
        assert_eq!(ROUNDING_CORRECTION, rounding);
    }

    #[test]
    fn test_u256_maximum_sqrt_base_constant() {
        assert_eq!(
            MAXIMUM_SQRT_BASE,
            PreciseNumber::new(u128::MAX).unwrap().value
        );
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
        assert!(TestPreciseNumber32::new(4294967).is_some());
        assert!(TestPreciseNumber32::new(4294967 + 1).is_none());
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

    proptest! {
        #[test]
        fn test_square_root(a in 0..u128::MAX) {
            let a = PreciseNumber { value: InnerUint::from(a) };
            check_square_root(&a);
        }
    }
}
