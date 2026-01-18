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
        if x < 0 || x > self.width {
            panic!("x outside of (0, {}): {}", self.width, x)
        }
        if y < 0 || y > self.width {
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

use std::fmt::{self, Display, Write};

impl<T: Default + Clone + PartialEq + Display> fmt::Display for Matrix<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut return_value = String::new();

        for y in 0..self.height {
            return_value.push('|');

            for x in 0..self.width {
                if self.at(x, y) != &T::default() {
                    write!(return_value, "{}", self.at(x, y))?;
                } else {
                    return_value.push(' ')
                }
            }
        }

        write!(f, "{return_value}")
    }
}
