#![cfg_attr(not(feature = "std"), no_std)]

mod v2;
use crate::v2::V2;
mod matrix;
use crate::matrix::Matrix;

pub fn lib() -> String {
    let mut matrix = Matrix::new(8, 8, 0);
    matrix.set(1, 2, 17);
    matrix.to_string()
}
