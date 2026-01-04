// Re-export methods to comply with prev structure

#[inline]
pub fn f32_normal_cdf(argument: f32) -> f32 {
    super::distribution_math::f32_normal_cdf(argument)
}

#[inline]
pub fn sqrt(radicand: u128) -> Option<u128> {
    super::sqrt_math::sqrt_binary_system(radicand)
}

