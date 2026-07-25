#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(not(feature = "std"), not(feature = "esp")))]
macro_rules! println {
    ($($t:tt)*) => {};
}

pub mod engine;
pub mod scenes;
