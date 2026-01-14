#![allow(clippy::arithmetic_side_effects)]
//! Defines PreciseNumber, a U256 wrapper with float-like operations
//! Important: put this makro inside an unique module to avoid name clashes

#[macro_export]
macro_rules! define_precise_number {
    ($Precise:ident, $TOuter:ty, $FPInner:ty, $FP_ONE:expr, $FP_ONE_F64:expr, $FP_ZERO:expr, $ROUNDING_CORRECTION:expr, $PRECISION:expr, $MAXIMUM_SQRT_BASE:expr, $CONVERT_F64:expr) => {

        mod foobar {}

        /// Struct encapsulating a fixed-point number that allows for decimal
        /// calculations
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub struct $Precise {
            /// Wrapper over the inner value, which is multiplied by ONE
            pub value: $FPInner,
        }

        #[allow(dead_code)]
        impl $Precise {
            pub(crate) const FP_ONE: $FPInner = $FP_ONE;
            const FP_ONE_F64: f64 = $FP_ONE_F64;
            pub(crate) const FP_ZERO: $FPInner = $FP_ZERO;
            const CONVERT_FROM_F64: fn(f64) -> Option<$FPInner> = $CONVERT_F64;

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

            pub const NUM_BITS: u32 = size_of::<$FPInner>() as u32 * 8;

            pub const fn zero() -> Self {
                Self {
                    value: Self::FP_ZERO,
                }
            }

            pub const fn one() -> Self {
                Self {
                    value: Self::FP_ONE,
                }
            }

            /// Maximum number iterations to apply on checked_pow_approximation.
            /// use test_sqrt_precision_tuner to adjust this value
            pub(crate) const MAX_APPROXIMATION_ITERATIONS: u32 = 100;

            /// Limit the bitshifts in cordic
            /// use test_sqrt_precision_tuner to adjust this value
            const CORDIC_SPEED_FACTOR: u32 = 41; // 12 digits precision (same as newton)

            /// Minimum base (excl) allowed when calculating exponents in checked_pow_fraction
            /// and checked_pow_approximation.  This simply avoids 0 as a base.
            pub(crate) fn min_pow_base_excl() -> $FPInner {
                Self::FP_ZERO
            }

            /// Maximum base allowed when calculating exponents in checked_pow_fraction
            /// and checked_pow_approximation.  The calculation use a Taylor Series
            /// approximation around 1, which converges for bases between 0 and 2.  See
            /// https://en.wikipedia.org/wiki/Binomial_series#Conditions_for_convergence
            /// for more information.
            pub(crate) fn max_pow_base() -> $FPInner {
                Self::FP_ONE + Self::FP_ONE
            }

            /// Create a precise number from an imprecise outer type, should always succeed
            pub fn new(int_val: $TOuter) -> Option<Self> {
                let int_value: $FPInner = int_val.into();
                let value: $FPInner = int_value.checked_mul(Self::FP_ONE).unwrap();
                Some(Self { value })
            }

            /// Convert a precise number back to outer type
            pub fn to_imprecise(self) -> Option<$TOuter> {
                self.value
                    .checked_add(Self::ROUNDING_CORRECTION)?
                    .checked_div(Self::FP_ONE)
                    .and_then(|v| <$TOuter>::try_from(v).ok())
            }

            /// Checks that two PreciseNumbers are equal within some tolerance
            pub fn almost_eq(&self, rhs: &Self, precision: $FPInner) -> bool {
                let (difference, _) = self.unsigned_sub(rhs);
                difference.value <= precision
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

            /// divide PreciseNumber by inner type
            pub fn checked_div_inner(&self, rhs: &$FPInner) -> Option<Self> {
                if *rhs == $FP_ZERO {
                    return None;
                }

                self.value.checked_add(Self::ROUNDING_CORRECTION)?
                    .checked_div(*rhs)
                    .map(|value| Self { value })
            }

            pub(crate) fn div2(&self) -> Self {
                use std::ops::Shr;
                let value = self.value.shr(1);
                Self { value }
            }

            pub(crate) fn div10(&self) -> Self {
                let value = self.value.checked_div(10u8.into()).unwrap();
                Self { value }
            }

            pub(crate) fn mul2(&self) -> Option<Self> {
                let value = self.value.checked_add(self.value)?;
                Some(Self { value })
            }

            #[inline(always)]
            pub(crate) fn pow2(value: $FPInner) -> Option<$FPInner> {
                // 33% faster than checked_pow
                value.checked_mul(value)
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
            pub(crate) fn checked_pow_approximation(
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

            // note: not used anymore
            /// Approximate the nth root of a number using Newton's method
            /// Adoption of python example in https://en.wikipedia.org/wiki/Newton%27s_method#Code
            /// NOTE: this function is private because its accurate range and precision
            /// have not been established.
            fn newtonian_root_approximation_generic(
                &self,
                root: &Self,
                mut guess: Self,
                iterations: u32,
            ) -> Option<Self> {
                let zero = Self::zero();
                if *self == zero || *self == Self::one() {
                    return Some(*self);
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

            // optimized version for root==2
            fn newtonian_root_approximation_fast(
                &self,
                mut guess: Self,
                iterations: u32,
            ) -> Option<Self> {
                let a = self;
                let zero = Self::zero();
                if *a == zero || *a == Self::one() {
                    return Some(*a);
                }
                // precalc first part of checked_div
                let a_scaled = a
                    .value
                    .checked_mul(Self::FP_ONE)?
                    .checked_add(Self::ROUNDING_CORRECTION)?;
                for _ in 0..iterations {
                    // x_k+1 = ((n - 1) * x_k + A / (x_k ^ (n - 1))) / n
                    // .. with n=2
                    // x_k+1 = (x_k + A / x_k) / 2
                    // let next_guess = guess.checked_add(&a.checked_div(&guess)?)?.div2();
                    let next_guess_inner = (guess
                        .value
                        .checked_add(a_scaled.checked_div(guess.value)?)?)
                        / 2;
                    let next_guess = Self {
                        value: next_guess_inner,
                    };
                    // note: reference algo uses "<="
                    if guess.almost_eq(&next_guess, Self::PRECISION) {
                        guess = next_guess;
                        break;
                    } else {
                        guess = next_guess;
                    }
                }
                Some(guess)
            }



            // optimized version
            fn cordic_root_approximation_fast(
                &self, speed_factor: u32,
            ) -> Option<Self> {

                lazy_static::lazy_static! {
                    static ref POW2_TABLE: Vec<$FPInner> = {
                        use num_traits::{CheckedShl, CheckedShr};
                        let mut table = Vec::new();
                        for i in 0..=(2*$Precise::NUM_BITS+1) {
                            let shift = i as i32 - $Precise::NUM_BITS as i32;
                            let pow2 = if shift < 0 {
                                //$FP_ONE >> -shift
                                let Some(out) = $FP_ONE.checked_shr((-shift) as u32) else {
                                    continue;
                                };
                                out
                            } else {
                                //$FP_ONE << shift
                                let Some(out) = $FP_ONE.checked_shl(shift as u32) else {
                                    continue;
                                };
                                out
                            };
                            table.push(pow2);

                        }
                        table
                    };
                    static ref POW2_SQUARE_TABLE: Vec<$FPInner> = {
                        use num_traits::{CheckedShl, CheckedShr};
                        let mut table = Vec::new();
                        for i in 0..=(2*$Precise::NUM_BITS) {
                            let shift = i as i32 - $Precise::NUM_BITS as i32;

                            let Some(one_square) = $Precise::FP_ONE.checked_mul($Precise::FP_ONE) else {
                                panic!("error builing pow2_table");
                            };
                            let pow2 = if shift < 0 {
                                // one_square >> -2*shift
                                //<$Precise>::shr_inner(one_square, (-2*shift) as u32)
                                let Some(out) = one_square.checked_shr((-2*shift) as u32) else {
                                    panic!("error builing pow2_table");
                                };
                                out
                            } else {
                                // one_square << 2*shift
                                //<$Precise>::shl_inner(one_square, (2*shift) as u32)
                                let Some(out) = one_square.checked_shl((2*shift) as u32) else {
                                    panic!("error builing pow2_table");
                                };
                                out
                            };
                            table.push(pow2);

                        }
                        table
                    };
                }

                // calc FP_ONE * 2^n
                // #[inline(always)]
                fn one_pow2(n: i32) -> $FPInner {
                    // TODO change to debug_assert!
                    assert!(n <= $Precise::NUM_BITS as i32, "error in one_pow2");
                    assert!(n >= -($Precise::NUM_BITS as i32), "error in one_pow2");
                    let shift = n + $Precise::NUM_BITS as i32;
                    panic!("one_pow2({}) len={}", n, POW2_TABLE.len());
                    return POW2_TABLE[shift as usize];
                }

                // calc FP_ONE^2 * 2^(2n)
                // #[inline(always)]
                // TODO unused?
                // fn one_pow2_squared(n: i32) -> $FPInner {
                //     // TODO change to debug_assert!
                //     assert!(n <= $Precise::NUM_BITS as i32, "error2");
                //     assert!(n >= -($Precise::NUM_BITS as i32), "error3");
                //     let shift = n + $Precise::NUM_BITS as i32;
                //     return POW2_SQUARE_TABLE[shift as usize];
                // }

                // assert_eq!(one_pow2(0), Self::FP_ONE, "error at one_pow2 a");
                // assert_eq!(one_pow2(3), Self::FP_ONE * 8, "error at one_pow2 b");
                // assert_eq!(one_pow2(-3), Self::FP_ONE / 8, "error at one_pow2 c");
                // assert_eq!(one_pow2_squared(0), Self::FP_ONE * Self::FP_ONE, "error at one_pow2_squared a");
                // assert_eq!(one_pow2_squared(1), Self::FP_ONE.checked_mul(Self::FP_ONE).unwrap() * 4, "error at one_pow2_squared b");

                let x = *self;
                if x == Self::zero() || x == Self::one() {
                    return Some(x);
                }

                // let x_shifted = x.value.checked_mul(Self::FP_ONE)?;

                let mut pow2_inner_shift: i32 = 0;
                // let mut pow2_inner_squared_shift: i32 = 0;
                // let mut pow2_inner = Self::FP_ONE;
                // let mut pow2_inner_squared = Self::pow2(Self::FP_ONE)?;

                // panic!("CHECKPOINT2");
                // need to use bitshift instead of mul/div because it seems to make difference in performance with SBF
                let mut result_inner = if x.value < Self::FP_ONE {
                    panic!("CHECK BAR");
                    while x.value <= one_pow2(2*pow2_inner_shift) {
                        // pow2_inner >>= 1;
                        panic!("CHECKPOINTz pow2_inner_shift={}", pow2_inner_shift);
                        pow2_inner_shift -= 1;
                        // pow2_inner_squared >>= 2;
                        // pow2_inner_squared_shift -= 2;
                    }
                    panic!("CHECKPOINT pow2_inner_shift={}", pow2_inner_shift);
                    one_pow2(pow2_inner_shift)
                } else {
                    // panic!("CHECK FOO pow2_inner_shift={}", pow2_inner_shift);
                    let _ = one_pow2(2*pow2_inner_shift);
                    panic!("CHECK FOO_after");
                    // x >= 1
                    while one_pow2(2*pow2_inner_shift) <= x.value {
                        // pow2_inner <<= 1;
                         panic!("CHECKPOINTy pow2_inner_shift={}", pow2_inner_shift);
                        pow2_inner_shift += 1;
                        // pow2_inner_squared <<= 2;
                        // pow2_inner_squared_shift += 2;
                    }
                    // pow2_inner >> 1
                    panic!("CHECKPOINT pow2_inner_shift={}", pow2_inner_shift);
                    one_pow2(pow2_inner_shift - 1)
                };

                panic!("CHECKPOINT3");

                let x_shifted = x.value.checked_mul(Self::FP_ONE)?;

                // FIXME use a better value for max iterations
                // limit iterations, see https://github.com/Max-Gulda/Cordic-Math/blob/9309c134a220f63ed67358d8fb813c6d4f506ba5/lib/cordicMath/src/cordic-math.c#L443
                // const CORDIC_SPEED_FACTOR: u32 = 15;
                // let speed_factor: u32 = Self::NUM_BITS;

                // if speed_factor is larger than NUM_BITS, the loop will terminate automatically
                for _ in 0..speed_factor {
                   // pow2_inner >>= 1;
                   pow2_inner_shift -= 1;
                   let pow2_inner = one_pow2(pow2_inner_shift);
                    if pow2_inner == Self::FP_ZERO {
                        break;
                    }
                    // we can stop if pow2_inner is zero as further iterations won't change result
                    let next_result_inner = result_inner.checked_add(pow2_inner)?;
                    if Self::pow2(next_result_inner)? <= x_shifted {
                        result_inner = next_result_inner;
                    }
                }

                Some(Self { value: result_inner } )

            }

            // port of this https://github.com/sebcrozet/cordic/blob/0cb0773e879721ad8c72cd36dcb7eb27bd2f83a4/cordic/src/lib.rs#L204
            fn cordic_root_approximation_naiv(
                &self
            ) -> Option<Self> {
                let x = *self;
                if x == Self::zero() || x == Self::one() {
                    return Some(x);
                }

                let mut pow2 = Self::one();
                let mut result;

                if x.value < Self::FP_ONE {
                    while x.value <= pow2.checked_pow(2)?.value {
                        pow2 = pow2.div2();
                    }

                    result = pow2;
                } else {
                    // x >= T::one()
                    while pow2.checked_pow(2)?.value <= x.value {
                        pow2 = pow2.mul2()?;
                    }

                    result = pow2.div2();
                }


                for _ in 0..Self::NUM_BITS {
                    pow2 = pow2.div2();
                    let next_result = result.checked_add(&pow2)?;
                    if next_result.checked_pow(2)?.value <= x.value {
                        result = next_result;
                    }
                }

                Some(result)
            }


            /// Based on testing around the limits, this base is the smallest value that
            /// provides an epsilon 11 digits
            pub(crate) fn minimum_sqrt_base() -> Self {
                Self::zero()
            }

            /// Based on testing around the limits, this base is the smallest value that
            /// provides an epsilon of 11 digits
            pub(crate) fn maximum_sqrt_base() -> Self {
                Self {
                    value: Self::MAXIMUM_SQRT_BASE,
                }
            }

            /// Approximate the square root using recommended method.
            ///
            /// For specific needs use sqrt_newton or sqrt_cordic directly.
            pub fn sqrt(&self) -> Option<Self> {
                self.sqrt_cordic()
            }

            /// Approximate the square root using Newton's method.  Based on testing,
            /// this provides a precision of 11 digits for inputs between 0 and
            /// u128::MAX
            pub fn sqrt_newton(&self) -> Option<Self> {
                if self.less_than(&Self::minimum_sqrt_base())
                    || self.greater_than(&Self::maximum_sqrt_base())
                {
                    return None;
                }

                let one = Self::one();
                // A good initial guess is the average of the interval that contains the
                // input number.  For all numbers, that will be between 1 and the given number.
                let guess = self.checked_add(&one)?.div2();
                self.newtonian_root_approximation_fast(guess, Self::MAX_APPROXIMATION_ITERATIONS)
            }

            /// Approximate the square root using CORDIC's method.
            /// newton vs cordic: newton is faster on SBF but slower on ARM
            pub fn sqrt_cordic(&self) -> Option<Self> {
                if self.less_than(&Self::minimum_sqrt_base())
                    || self.greater_than(&Self::maximum_sqrt_base())
                {
                    return None;
                }
                self.cordic_root_approximation_fast(Self::CORDIC_SPEED_FACTOR)
            }

            #[cfg(feature = "from_f64")]
            pub fn new_from_f64(input_f64: f64) -> Option<Self> {
                let scaled_value = input_f64 * Self::FP_ONE_F64;
                Self::new_from_inner_f64(scaled_value)
            }

            #[cfg(feature = "from_f64")]
            pub fn new_from_inner_f64(inner_value: f64) -> Option<Self> {
                Self::CONVERT_FROM_F64(inner_value).map(|value| Self { value })
            }

            #[cfg(test)]
            // very hacky and slow implementation for testing purposes only
            pub fn to_str_pretty(&self) -> String {
                use bigdecimal_rs::BigDecimal;
                use std::str::FromStr;
                use std::ops::Div;
                let bd = BigDecimal::from_str(&format!("{}", self.value))
                .unwrap()
                .div(BigDecimal::from_str(&format!("{}", Self::FP_ONE)).unwrap());
                format!("{}", bd)
            }

        }
    };
} // -- macro

#[macro_export]
macro_rules! define_muldiv {
    // Struct, u128, U256, U512
    ($Precise:ident, $TOuter:ty, $FPInner:ty, $FPInnerDoublePrecision:ty) => {
        #[allow(dead_code)]
        impl $Precise {
            #[inline(always)]
            fn extend_precsion(val: $FPInner) -> $FPInnerDoublePrecision {
                <$FPInnerDoublePrecision>::from(val)
            }

            #[inline(always)]
            fn trunc_precision(val: $FPInnerDoublePrecision) -> Option<$FPInner> {
                <$FPInner>::try_from(val).ok()
            }

            pub fn mul_div_floor(self, num: Self, denom: Self) -> Option<Self> {
                if denom.value == Self::FP_ZERO {
                    return None;
                }

                if let Some(dividend) = self.value.checked_mul(num.value) {
                    // small number, no overflow
                    let r = dividend / denom.value;
                    Some($Precise { value: r })
                } else {
                    let r = (Self::extend_precsion(self.value) * Self::extend_precsion(num.value))
                        / Self::extend_precsion(denom.value);

                    Self::trunc_precision(r).map(|v| $Precise { value: v })
                }
            }

            pub(crate) fn mul_div_floor_naive(self, num: Self, denom: Self) -> Option<Self> {
                if denom.value == Self::FP_ZERO {
                    return None;
                }
                let r = (Self::extend_precsion(self.value) * Self::extend_precsion(num.value))
                    / Self::extend_precsion(denom.value);

                Self::trunc_precision(r).map(|v| $Precise { value: v })
            }

            pub fn mul_div_ceil(self, num: Self, denom: Self) -> Option<Self> {
                if denom.value == Self::FP_ZERO {
                    return None;
                }

                if let Some(dividend) = self
                    .value
                    .checked_mul(num.value)
                    .and_then(|x| x.checked_add(denom.value - 1))
                {
                    // small number, no overflow
                    let r = dividend / denom.value;
                    Some($Precise { value: r })
                } else {
                    let r = (Self::extend_precsion(self.value) * Self::extend_precsion(num.value))
                        / Self::extend_precsion(denom.value);

                    Self::trunc_precision(r).map(|v| $Precise { value: v })
                }
            }

            #[allow(clippy::manual_div_ceil)]
            pub(crate) fn mul_div_ceil_naive(self, num: Self, denom: Self) -> Option<Self> {
                if denom.value == Self::FP_ZERO {
                    return None;
                }
                let r = (Self::extend_precsion(self.value) * Self::extend_precsion(num.value)
                    + (Self::extend_precsion(denom.value) - 1))
                    / Self::extend_precsion(denom.value);

                Self::trunc_precision(r).map(|v| $Precise { value: v })
            }
        }
    };
}


#[macro_export]
macro_rules! define_sqrt_tests {
    // Struct, u128, U256, U512
    ($Precise:ident, $TOuter:ty, $FPInner:ty, $FPInnerDoublePrecision:ty) => {

        #[cfg(test)]
        mod sqrt_tests {
            use super::*;


            // makes sure that both sqrt methods have similar precision
            // see MAX_APPROXIMATION_ITERATIONS and CORDIC_SPEED_FACTOR for details
            #[test]
            fn test_sqrt_precision_tuner() {

                // newton, cordic
                const TARGET_PRECISION: (u32, u32) = (11, 11);

                assert_eq!(
                    compare_newton_vs_cordic_precision(<$Precise>::maximum_sqrt_base()),
                    TARGET_PRECISION, "precision at maximum_sqrt_base failed");

                assert_eq!(
                    compare_newton_vs_cordic_precision(<$Precise>::maximum_sqrt_base().div2()),
                    TARGET_PRECISION, "precision at maximum_sqrt_base/2 failed");

                assert_eq!(
                    compare_newton_vs_cordic_precision((<$Precise>::one().checked_add(&<$Precise>::one()).unwrap().checked_add(&<$Precise>::one()).unwrap()).div2()),
                    TARGET_PRECISION, "precision at 1.5 failed");

            }

            fn find_max_precision(approximate_root: $Precise, radicand: $Precise) -> u32 {
                let mut best_precision = 0u32;
                for (precision, eps) in precisions_enumerated() {
                    let (lower_bound, upper_bound) = calc_square_root_bounds(&approximate_root, precision);
                    if radicand.less_than_or_equal(&upper_bound)
                        && radicand.greater_than_or_equal(&lower_bound)
                    {
                        best_precision = precision;
                    } else {
                        break;
                    }
                }
                best_precision
            }

            fn precisions_enumerated() -> Vec<(u32, $FPInner)> {
                let mut out = Vec::new();
                let mut cur = $Precise::one();
                let zero = <$Precise>::zero();
                for precision in 0..1000 {
                    out.push((precision, cur.value));
                    cur = cur.div10();
                    if cur == zero {
                        break;
                    }
                }
                out
            }

            fn compare_newton_vs_cordic_precision(
                radicand: $Precise
            ) -> (u32, u32) {

                let precision_newton = find_max_precision(radicand.sqrt_newton().unwrap(), radicand);
                let precision_cordic = find_max_precision(radicand.sqrt_cordic().unwrap(), radicand);

                (precision_newton, precision_cordic)
            }

            // this accounts for the absolute error - in contract to relative error
            fn calc_square_root_bounds(
                approximate_root: &$Precise,
                precision: u32,
            ) -> ($Precise, $Precise) {
                let epsilon = $Precise {
                    value: precision_in_inner(precision),
                };
                let one = <$Precise>::one();
                let one_plus_epsilon = one.checked_add(&epsilon).unwrap();
                let one_minus_epsilon = one.checked_sub(&epsilon).unwrap();
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
                (lower_bound, upper_bound)
            }

            // returns 10**(-digits) in InnerUint
            // for testing only, neither fast not beautiful
            fn precision_in_inner(digits: u32) -> $FPInner {
                let mut result = ONE_CONST;
                let ten = <$FPInner>::from(10u8);
                for _ in 0..digits {
                    result = result.checked_div(ten).unwrap();
                }
                assert!(result != <$Precise>::FP_ZERO, "precision underflow, digits={}", digits);
                result
            }


        }

    }
}
