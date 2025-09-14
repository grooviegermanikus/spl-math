
#[inline(always)]
pub fn slow(a: i32, b: i32, c: i32) -> bool {
    let sgn = a.signum() * b.signum() * c.signum();
    sgn < 0
}

#[inline(always)]
pub fn fast(a: i32, b: i32, c: i32) -> bool {
    // let sgn = (a ^ b ^ c).signum();
    // don't need to check for zero or positive, just need to check for negative
    let sgn =  (a ^ b ^ c) & (1 << 31);
    // TODO for multdiv we can assume that c != 0 and also shortcut for the  two others
    let any_zero = a == 0 || b == 0 || c == 0;
    !any_zero && sgn < 0
    // !any_zero
}

#[cfg(test)]
mod tests {
    use quickcheck_macros::quickcheck;
    use super::*;

    #[quickcheck]
    fn fast_slow_signum3(a: i32, b: i32, c: i32) -> bool {
        fast(a,b,c) == slow(a,b,c)
    }
}
