extern crate alloc;
use alloc::{string::String, string::ToString, vec, vec::Vec};
use libm::{ceilf, cosf, roundf, sinf};


#[derive(Clone)]
pub struct Matrix<T: Clone> {
    pub width: u8,
    pub height: u8,
    pub data: Vec<T>,
}

impl<T: Clone> Matrix<T> {
    pub fn new(width: u8, height: u8, init: T) -> Self {
        assert!(width > 0 || height > 0, "Matrix::new zero: {}x{}", width, height);
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

    pub fn fill(&mut self, to: T)
    where
        T: Copy,
    {
        self.data.fill(to);
    }

    // TODO: do in-place. swapping pixels
    pub fn rotate(&mut self, degrees: f32, background: T) {
        let rad = (degrees * PI) / 180.0;
        let sin_abs = sinf(rad).abs();
        let cos_abs = cosf(rad).abs();

        let old_width = self.width as f32;
        let old_height = self.height as f32;
        let new_width = ceilf(old_width * cos_abs + old_width * sin_abs);
        let new_height = ceilf(old_height * cos_abs + old_height * sin_abs);

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

    pub fn scale(&mut self, factor: f32, background: T, resize: bool) {
        let old_width = self.width;
        let old_height = self.height;
        let scaled_width = (old_width as f32 * factor) as usize;
        let scaled_height = (old_height as f32 * factor) as usize;
        let new_width = if resize { scaled_width as u8 } else { old_width };
        let new_height = if resize { scaled_height as u8 } else { old_height };

        // #[cfg(feature = "esp")]
        // println!("free: {}", esp_alloc::HEAP.free());

        if new_width <= 0 || new_height <= 0 {
            return;
        }

        if factor >= 1.0 {
            let f = factor as usize;
            let oy = old_height as usize;
            let ox = old_width as usize;
            self.data.resize(new_width as usize * new_height as usize, background.clone());
            for y in (0..oy).rev() {
                for x in (0..ox).rev() {
                    let src_idx = y * ox + x;
                    let pixel = self.data[src_idx].clone();
                    for dy in 0..f {
                        for dx in 0..f {
                            let nx = x * f + dx;
                            let ny = y * f + dy;
                            if nx < new_width as usize && ny < new_height as usize {
                                self.data[ny * new_width as usize + nx] = pixel.clone();
                            }
                        }
                    }
                }
            }
            if !resize {
                for y in 0..new_height as usize {
                    for x in 0..new_width as usize {
                        if x >= scaled_width || y >= scaled_height {
                            self.data[y * new_width as usize + x] = background.clone();
                        }
                    }
                }
            }
            self.width = new_width;
            self.height = new_height;
        } else {
            // println!("scale {}, {}", new_width, new_height);
            for y in 0..scaled_height.min(new_height as usize) {
                for x in 0..scaled_width.min(new_width as usize) {
                    let src_x = (x as f32 / factor) as u8;
                    let src_y = (y as f32 / factor) as u8;
                    if src_x < old_width && src_y < old_height {
                        self.data[y * old_width as usize + x] = self.data[src_y as usize * old_width as usize + src_x as usize].clone();
                    }
                }
            }
            if resize {
                self.data.truncate(new_width as usize * new_height as usize);
                self.data.shrink_to_fit();
            }
            self.width = new_width;
            self.height = new_height;
        }
    }

    pub fn scale_into(&self, factor: usize, dst: &mut Matrix<T>, _background: T) {
        for y in 0..self.height as usize {
            for x in 0..self.width as usize {
                let pixel = self.get(x as u8, y as u8).clone();
                for dy in 0..factor {
                    for dx in 0..factor {
                        dst.set((x * factor + dx) as u8, (y * factor + dy) as u8, pixel.clone());
                    }
                }
            }
        }
    }

    pub fn snippet(&self, from: &V2, to: &V2) -> Matrix<T> {
        let mut result = Matrix::new((to.x - from.x) as u8, (to.y - from.y) as u8, self.data.get(0).unwrap().clone());
        for x in from.x as u8..to.x as u8 {
            for y in from.y as u8..to.y as u8 {
                let t: T = self.get(x, y).clone();
                result.set(x - from.x as u8, y - from.y as u8, t);
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
use core::write;

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
