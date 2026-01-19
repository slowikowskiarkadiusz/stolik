#![cfg_attr(not(feature = "std"), no_std)]

pub struct Matrix<T> {
    pub width: u8,
    pub height: u8,
    pub data: Vec<T>,
}

impl<T: Clone> Matrix<T> {
    pub fn new(width: u8, height: u8, init: T) -> Self {
        Self {
            width,
            height,
            data: vec![init; (width * height) as usize],
        }
    }

    pub fn at(&self, x: u8, y: u8) -> &T {
        if x > self.width {
            panic!("x outside of (0, {}): {}", self.width, x)
        }
        if y > self.width {
            panic!("y outside of (0, {}): {}", self.height, y)
        }
        &self.data[(y * self.width + x) as usize]
    }

    pub fn set(&mut self, x: u8, y: u8, to: T) -> &mut Self {
        self.data[(y * self.width + x) as usize] = to;
        self
    }

    pub fn data(&self) -> &Vec<T> {
        &self.data
    }
}

use core::fmt::{self, Display, Write};

impl<T: Default + Clone + PartialEq + Display> fmt::Display for Matrix<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut return_value = String::new();
        let mut longest: usize = 0;

        for y in 0..self.height {
            return_value.push('|');

            for x in 0..self.width {
                if self.at(x, y) != &T::default() {
                    let val = self.at(x, y).to_string();
                    write!(return_value, "{}", val)?;
                    longest = val.len();
                } else {
                    return_value.push_str(".")
                }
            }
            return_value.push_str("|\n");
        }

        return_value = return_value.replace(".", &".".repeat(longest));

        write!(f, "{return_value}")
    }
}
