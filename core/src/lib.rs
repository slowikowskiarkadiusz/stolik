#![cfg_attr(not(feature = "std"), no_std)]

pub mod engine;

pub fn lib() -> String {
    let a: u16 = 300;
    (a.clamp(0, 255) as u8).to_string()
}
