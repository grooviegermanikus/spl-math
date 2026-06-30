//! Test-only reference implementations of alternate algorithms.
//!
//! These are intentionally NOT part of the shipped library. They exist purely as
//! independent oracles for differential testing: the optimized production
//! algorithms (`sqrt_newton`, `sqrt_cordic`, `checked_pow`, `mul_div_*`) are
//! checked against these slower, simpler reference versions to prove the
//! optimizations preserve correctness.
//!
//! Each macro emits an extra `impl` block on a type already produced by
//! `define_precise_number!` / `define_muldiv!`. They are invoked only from the
//! `#[cfg(test)]` modules that need them. A given test exercises a subset of the
//! methods, so the impl blocks carry `#[allow(dead_code)]`.

/// Reference sqrt and pow algorithms for a `define_precise_number!` type.
#[macro_export]
macro_rules! define_precise_number_oracles {
    ($Precise:ident, $FPInner:ty) => {
        #[allow(dead_code)]
        impl $Precise {
            /// Minimum base (excl) allowed by the experimental pow approximation.
            /// This simply avoids 0 as a base.
            fn min_pow_base_excl() -> $FPInner {
                Self::FP_ZERO
            }

            /// Maximum base allowed by the experimental pow approximation.  The
            /// calculation uses a Taylor Series approximation around 1, which
            /// converges for bases between 0 and 2.  See
            /// https://en.wikipedia.org/wiki/Binomial_series#Conditions_for_convergence
            /// for more information.
            fn max_pow_base() -> $FPInner {
                Self::FP_ONE + Self::FP_ONE
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
            /// NOTE: experimental; its accurate range and precision have not been
            /// established, hence kept as a test-only reference.
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
            /// (numerator / denominator).
            /// NOTE: experimental; its accurate range and precision have not been
            /// established, hence kept as a test-only reference.
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

            /// Approximate the nth root of a number using Newton's method.
            /// General n-th root reference; the production code specializes to n=2
            /// (`newtonian_sqrt_approximation_fast`) and is validated against this.
            /// Adoption of python example in https://en.wikipedia.org/wiki/Newton%27s_method#Code
            fn newtonian_sqrt_approximation_generic(
                &self,
                nth_root: &Self,
                mut guess: Self,
                // safety valve to avoid infinite loops
                max_iterations: u32,
            ) -> Option<Self> {
                let zero = Self::zero();
                if *self == zero || *self == Self::one() {
                    return Some(*self);
                }
                if *nth_root == zero {
                    return None;
                }
                let one = Self::one();
                let nth_root_minus_one = nth_root.checked_sub(&one)?;
                let nth_root_minus_one_whole = nth_root_minus_one.to_imprecise()?;
                let mut last_guess = guess.clone();
                for _ in 0..max_iterations {
                    // x_k+1 = ((n - 1) * x_k + A / (x_k ^ (n - 1))) / n
                    let first_term = nth_root_minus_one.checked_mul(&guess)?;
                    let power = guess.checked_pow(nth_root_minus_one_whole.try_into().ok()?);
                    let second_term = match power {
                        Some(num) => self.checked_div(&num)?,
                        None => Self::zero(),
                    };
                    guess = first_term
                        .checked_add(&second_term)?
                        .checked_div(nth_root)?;
                    if last_guess.almost_eq(&guess, Self::PRECISION) {
                        break;
                    } else {
                        last_guess = guess.clone();
                    }
                }
                Some(guess)
            }

            fn mul2(&self) -> Option<Self> {
                let value = self.value.checked_add(self.value)?;
                Some(Self { value })
            }

            /// port of this https://github.com/sebcrozet/cordic/blob/0cb0773e879721ad8c72cd36dcb7eb27bd2f83a4/cordic/src/lib.rs#L204
            /// Naive CORDIC sqrt; the production `cordic_sqrt_approximation_fast` is
            /// validated against this.
            fn cordic_sqrt_approximation_naive(&self) -> Option<Self> {
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

                // original algo used NUM_BITS
                for _ in 0..Self::MAX_APPROXIMATION_ITERATIONS {
                    pow2 = pow2.div2();
                    let next_result = result.checked_add(&pow2)?;
                    if next_result.checked_pow(2)?.value <= x.value {
                        if result.almost_eq(&next_result, Self::PRECISION) {
                            result = next_result;
                            break;
                        } else {
                            result = next_result;
                        }
                    }
                }

                Some(result)
            }
        }
    };
}

/// Naive `mul_div_*` baselines for a `define_muldiv!` type. Always take the
/// wide-integer path; the production `mul_div_*` use a fast no-overflow path
/// and are validated against these.
#[macro_export]
macro_rules! define_muldiv_oracles {
    ($Precise:ident) => {
        #[allow(dead_code)]
        impl $Precise {
            fn mul_div_floor_naive(self, num: Self, denom: Self) -> Option<Self> {
                if denom.value == Self::FP_ZERO {
                    return None;
                }
                let r = (Self::extend_precision(self.value) * Self::extend_precision(num.value))
                    / Self::extend_precision(denom.value);

                Self::trunc_precision(r).map(|v| $Precise { value: v })
            }

            #[allow(clippy::manual_div_ceil)]
            fn mul_div_ceil_naive(self, num: Self, denom: Self) -> Option<Self> {
                if denom.value == Self::FP_ZERO {
                    return None;
                }
                let r = (Self::extend_precision(self.value) * Self::extend_precision(num.value)
                    + (Self::extend_precision(denom.value) - 1))
                    / Self::extend_precision(denom.value);

                Self::trunc_precision(r).map(|v| $Precise { value: v })
            }
        }
    };
}
