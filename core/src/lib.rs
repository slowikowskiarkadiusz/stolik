#![cfg_attr(not(feature = "std"), no_std)]

pub mod engine;
use crate::engine::matrix::Matrix;

pub fn lib() -> String {
    let mut matrix = Matrix::new(8, 8, 0);
    matrix.set(1, 2, 17);
    matrix.to_string()
}
