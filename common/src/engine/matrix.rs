extern crate alloc;
use alloc::{string::String, string::ToString, vec, vec::Vec};
use libm::{ceilf, cosf, floorf, roundf, sinf};

#[derive(Clone)]
pub struct Matrix<T: Clone> {
    pub width: u8,
    pub height: u8,
    pub data: Vec<T>,
}

impl<T: Clone> Matrix<T> {
    pub fn new(width: u8, height: u8, init: T) -> Self {
        Self {
            width,
            height,
            data: vec![init; (width as usize * height as usize) as usize],
        }
    }

    pub fn get(&self, x: u8, y: u8) -> &T {
        if x > self.width {
            panic!("Matrix::at: x outside of (0, {}): {}", self.width, x)
        }
        if y > self.height {
            panic!("Matrix::at: y outside of (0, {}): {}", self.height, y)
        }
        &self.data[(y as usize * self.width as usize + x as usize) as usize]
    }

    pub fn set(&mut self, x: u8, y: u8, to: T) {
        if x < self.width && y < self.height {
            self.data[(y as u16 * self.width as u16 + x as u16) as usize] = to;
        }
    }

    pub fn get_size(&self) -> V2 {
        V2::new(self.width as f32, self.height as f32)
    }

    pub fn data(&self) -> &Vec<T> {
        &self.data
    }

    pub fn fill(&mut self, to: T) {
        self.data = vec![to; (self.width * self.height) as usize];
    }

    // TODO: do in-place. swapping pixels
    pub fn rotate(&mut self, degrees: f32, background: T) {
        let rad = (degrees * PI) / 180.0;
        let sin_abs = sinf(rad).abs();
        let cos_abs = cosf(rad).abs();

        let old_width = self.width as f32;
        let old_height = self.height as f32;
        let new_width = ceilf(old_width * cos_abs + old_width * sin_abs) + 1.0;
        let new_height = ceilf(old_height * cos_abs + old_height * sin_abs) + 1.0;

        let mut rotated = Matrix::<T>::new(new_width as u8, new_height as u8, background);

        let old_cx = old_width / 2.0;
        let old_cy = old_height / 2.0;
        let new_cx = new_width / 2.0;
        let new_cy = new_height / 2.0;

        for x in 0..(old_width as u8) {
            for y in 0..(old_height as u8) {
                let dx = x as f32 - old_cx;
                let dy = y as f32 - old_cy;
                let rx = roundf(cosf(rad) * dx - sinf(rad) * dy + new_cx);
                let ry = roundf(sinf(rad) * dx + cosf(rad) * dy + new_cy);

                if rx >= 0.0 && rx < new_width && ry >= 0.0 && ry < new_height {
                    rotated.set(rx as u8, ry as u8, self.get(x, y).clone());
                }
            }
        }

        self.data = rotated.data;
        self.width = new_width as u8;
        self.height = new_height as u8;
    }

    // TODO: do in-place. start writing from right bottom corner if the non-scaled is in left-top corner
    pub fn scale(&mut self, factor: f32, background: T) {
        let old_width = self.width;
        let old_height = self.height;
        let new_width = roundf(self.width as f32 * factor) as u8;
        let new_height = roundf(self.height as f32 * factor) as u8;

        let mut scaled = Matrix::<T>::new(new_width, new_height, background);

        for x in 0..new_width {
            for y in 0..new_height {
                let src_x = floorf(x as f32 / factor) as u8;
                let src_y = floorf(y as f32 / factor) as u8;

                if src_x < old_width && src_y < old_height {
                    scaled.set(x, y, self.get(src_x, src_y).clone());
                }
            }
        }

        self.data = scaled.data
    }

    pub fn snippet(&self, from: &V2, to: &V2) -> Matrix<T> {
        let mut result = Matrix::new((to.x - from.x) as u8, (to.y - from.y) as u8, self.data.get(0).unwrap().clone());
        for x in from.x as u8..to.x as u8 {
            for y in from.y as u8..to.y as u8 {
                let t: T = self.get(x, y).clone();
                result.set(x - from.x as u8, y - to.x as u8, t);
            }
        }

        result
    }
}

impl<T: Default + Clone> Matrix<T> {
    pub fn clear(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.set(x, y, T::default());
            }
        }
    }
}

use core::f32::consts::PI;
use core::fmt::{self, Display, Write};

use crate::engine::v2::V2;

impl<T: Default + Clone + PartialEq + Display> fmt::Display for Matrix<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut return_value = String::new();
        let mut longest: usize = 1;

        for y in 0..self.height {
            return_value.push('|');

            for x in 0..self.width {
                if self.get(x, y) != &T::default() {
                    let val = self.get(x, y).to_string();
                    write!(return_value, "{}", val)?;
                    let len = val.len();
                    if len > longest {
                        longest = len;
                    }
                } else {
                    return_value.push_str(" ")
                }
            }
            return_value.push_str("|\n");
        }

        return_value = return_value.replace(" ", &" ".repeat(longest));

        write!(f, "{return_value}")
    }
}
