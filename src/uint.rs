#![allow(clippy::arithmetic_side_effects)]
//! Large uint types

// required for clippy
#![allow(clippy::assign_op_pattern)]
#![allow(clippy::ptr_offset_with_cast)]
#![allow(clippy::manual_range_contains)]
#![allow(missing_docs)]

use uint::construct_uint;

construct_uint! {
    pub struct U256(4);
}
construct_uint! {
    pub struct U192(3);
}

impl TryFrom<f64> for U256 {
    type Error = ();

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        // let bytes: &[u64] = value.as_ref();
        // let len = bytes.len();
        // let lower = bytes[0..4].to_vec();
        // let upper = bytes[4..len].to_vec();
        // if upper.iter().any(|&x| x != 0) {
        //     return Err(());
        // }
        // let u256 = U256(lower.try_into().unwrap());
        // Ok(u256)
        todo!()
    }
}
