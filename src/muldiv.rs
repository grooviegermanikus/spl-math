#[inline(always)]
pub fn signum3_simple(a: i32, b: i32, c: i32) -> bool {
    let sgn = a.signum() * b.signum() * c.signum();
    sgn < 0
}

// TODO bake in assumptions that c is never zero
// 953 vs 1477
#[inline(always)]
pub fn signum3_fast(a: i32, b: i32, c: i32) -> bool {
    // let sgn = (a ^ b ^ c).signum();
    // don't need to check for zero or positive, just need to check for negative
    let sgn = (a ^ b ^ c) & (1 << 31);
    // TODO for muldiv we can assume that c != 0 and also shortcut for the  two others
    let any_zero = a == 0 || b == 0 || c == 0;
    !any_zero && sgn < 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck_macros::quickcheck;

    #[quickcheck]
    fn fast_slow_signum3(a: i32, b: i32, c: i32) -> bool {
        signum3_fast(a, b, c) == signum3_simple(a, b, c)
    }
}
