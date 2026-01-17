#![cfg_attr(not(feature = "std"), no_std)]

mod v2;
use crate::v2::V2;

pub fn lib() -> V2 {
    let v1 = V2::new(1.0, 2.0);
    let v2 = V2::new(3.0, 2.0);
    &v1 / v2.x
}
