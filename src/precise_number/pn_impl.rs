#![allow(clippy::arithmetic_side_effects)]
//! Defines PreciseNumber, a U256 wrapper with float-like operations

#[macro_export]
macro_rules! define_precise_number {
    ($Precise:ident, $TOuter:ty, $FPInner:ty, $FP_ONE:expr, $FP_ZERO:expr, $ROUNDING_CORRECTION:expr, $PRECISION:expr, $MAXIMUM_SQRT_BASE:expr) => {
        /// Struct encapsulating a fixed-point number that allows for decimal
        /// calculations
        #[derive(Clone, Debug, PartialEq)]
        pub struct $Precise {
            /// Wrapper over the inner value, which is multiplied by ONE
            pub value: $FPInner,
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

            /// Create a precise number from an imprecise outer type, should always succeed
            pub fn new(int_val: $TOuter) -> Self {
                let int_value: $FPInner = int_val.into();
                let value: $FPInner = int_value.checked_mul(Self::FP_ONE).unwrap();
                Self { value }
            }
            /// Convert a precise number back to outer type
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
