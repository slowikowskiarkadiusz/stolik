#![cfg_attr(not(feature = "std"), no_std)]

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

    pub fn set(&mut self, x: u8, y: u8, to: T) -> &mut Self {
        if x < self.width && y < self.height {
            self.data[(y * self.width + x) as usize] = to;
        }
        self
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
        let sin_abs = rad.sin().abs();
        let cos_abs = rad.cos().abs();

        let old_width = self.width as f32;
        let old_height = self.height as f32;
        let new_width = (old_width * cos_abs + old_width * sin_abs).ceil();
        let new_height = (old_height * cos_abs + old_height * sin_abs).ceil();

        let mut rotated = Matrix::<T>::new(new_width as u8, new_height as u8, background);

        let old_cx = old_width / 2.0;
        let old_cy = old_height / 2.0;
        let new_cx = new_width / 2.0;
        let new_cy = new_height / 2.0;

        for x in 0..(old_width as u8) {
            for y in 0..(old_height as u8) {
                let dx = x as f32 - old_cx;
                let dy = y as f32 - old_cy;

                let rx = (rad.cos() * dx - rad.sin() * dx + new_cx).round();
                let ry = (rad.cos() * dy - rad.sin() * dy + new_cy).round();

                if rx >= 0.0 && rx < new_width && ry >= 0.0 && ry < new_height {
                    rotated.set(x, y, self.get(x, y).clone());
                }
            }
        }

        self.data = rotated.data;
    }

    // TODO: do in-place. start writing from right bottom corner if the non-scaled is in left-top corner
    pub fn scale(&mut self, factor: f32, background: T) {
        let old_width = self.width;
        let old_height = self.height;
        let new_width = (self.width as f32 * factor).round() as u8;
        let new_height = (self.height as f32 * factor).round() as u8;

        let mut scaled = Matrix::<T>::new(new_width, new_height, background);

        for x in 0..new_width {
            for y in 0..new_height {
                let src_x = (x as f32 / factor).floor() as u8;
                let src_y = (y as f32 / factor).floor() as u8;

                if src_x < old_width && src_y < old_height{
                    scaled.set(x, y, self.get(src_x, src_y).clone());
                }
            }
        }

        self.data = scaled.data
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

use core::fmt::{self, Display, Write};
use std::f32::consts::PI;

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
                    return_value.push_str(".")
                }
            }
            return_value.push_str("|\n");
        }

        return_value = return_value.replace(".", &".".repeat(longest));

        write!(f, "{return_value}")
    }
}
