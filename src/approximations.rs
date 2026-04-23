// Re-export methods to comply with prev structure

use num_traits::{CheckedShl, CheckedShr, PrimInt};

#[inline]
pub fn f32_normal_cdf(argument: f32) -> f32 {
    super::distribution_math::f32_normal_cdf(argument)
}

#[inline]
pub fn sqrt<T: PrimInt + CheckedShl + CheckedShr>(radicand: T) -> Option<T> {
    super::sqrt_math::sqrt_binary_system(radicand)
}

#[inline]
pub fn sqrt_binary_system_naive<T: PrimInt + CheckedShl + CheckedShr>(radicand: T) -> Option<T> {
    super::sqrt_math::sqrt_binary_system_naive(radicand)
}
