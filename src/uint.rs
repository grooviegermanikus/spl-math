#![allow(clippy::arithmetic_side_effects)]
// note that uint crate does not (yet) support div_ceil; remove that and other clippy allow when it does
#![allow(clippy::manual_div_ceil)]
// required for clippy
#![allow(clippy::assign_op_pattern)]
#![allow(clippy::ptr_offset_with_cast)]
#![allow(clippy::manual_range_contains)]
#![allow(missing_docs)]

use uint::construct_uint;

construct_uint! {
    pub struct U512(8);
}
construct_uint! {
    pub struct U256(4);
}
construct_uint! {
    pub struct U192(3);
}

impl From<U256> for U512 {
    fn from(value: U256) -> Self {
        // cast to U512
        let bytes: &[u64] = value.as_ref();
        let dlen = bytes.len() * 2;
        // extend to 8 bytes:
        let mut bytes8 = Vec::with_capacity(dlen);
        bytes8.extend_from_slice(bytes);
        bytes8.resize(dlen, 0);
        U512(bytes8.try_into().unwrap())
    }
}

impl TryFrom<U512> for U256 {
    type Error = ();

    fn try_from(value: U512) -> Result<Self, Self::Error> {
        let bytes: &[u64] = value.as_ref();
        let len = bytes.len();
        let lower = bytes[0..4].to_vec();
        let upper = bytes[4..len].to_vec();
        if upper.iter().any(|&x| x != 0) {
            return Err(());
        }
        let u256 = U256(lower.try_into().unwrap());
        Ok(u256)
    }
}

#[test]
fn test_u256_to_u512() {
    let u256 = U256::from(1_000_000_000_000u128);
    let u512: U512 = U512::from(u256);
    let expected = U512::from(1_000_000_000_000u128);
    assert_eq!(u512, expected);
}

#[test]
fn test_u512_to_u256() {
    let u512 = U512::from(1_000_000_000_000u128);
    let u256: U256 = U256::try_from(u512).unwrap();
    let expected = U256::from(1_000_000_000_000u128);
    assert_eq!(u256, expected);
}

#[test]
fn test_u512_to_u256_overflow() {
    let u512 = U512::max_value();
    let u256: Result<U256, ()> = U256::try_from(u512);
    assert!(u256.is_err());
}
