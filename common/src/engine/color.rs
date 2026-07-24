extern crate alloc;
use alloc::{vec, vec::Vec};

#[derive(PartialEq, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn red() -> Color {
        Color::new(255, 0, 0, 255)
    }

    pub fn green() -> Color {
        Color::new(0, 255, 0, 255)
    }

    pub fn blue() -> Color {
        Color::new(0, 0, 255, 255)
    }

    pub fn yellow() -> Color {
        Color::new(255, 255, 0, 255)
    }

    pub fn cyan() -> Color {
        Color::new(0, 255, 255, 255)
    }

    pub fn magenta() -> Color {
        Color::new(255, 0, 255, 255)
    }

    pub fn brown() -> Color {
        Color::new(139, 69, 19, 255)
    }

    pub fn white() -> Color {
        Color::new(255, 255, 255, 255)
    }

    pub fn black() -> Color {
        Color::new(0, 0, 0, 255)
    }

    pub const fn none() -> Color {
        Color::new(0, 0, 0, 0)
    }

    pub fn a(&mut self, alpha: u8) -> &Color {
        self.a = alpha;
        self
    }

    pub fn is_none(&self) -> bool {
        self.r == 0 && self.g == 0 && self.b == 0 && self.a == 0
    }

    pub fn blend_colors(self, input_colors: &[Color]) -> Color {
        let mut colors: Vec<Color> = vec![];
        for a in input_colors {
            if a.is_none() {
                colors.push(*a);
            }
        }

        if colors.is_empty() {
            return Color::none();
        }

        if colors.len() == 1 {
            return *colors.first().unwrap();
        }

        Color::additive_blending(&colors)
    }

    fn additive_blending(colors: &[Color]) -> Color {
        let mut r: u16 = 0;
        let mut g: u16 = 0;
        let mut b: u16 = 0;
        let mut a: u16 = 0;
        for c in colors {
            r += (c.r as u16 * c.a as u16) / 255;
            g += (c.g as u16 * c.a as u16) / 255;
            b += (c.b as u16 * c.a as u16) / 255;
            a += c.a as u16;
        }

        Color::new(
            r.clamp(0, 255) as u8,
            g.clamp(0, 255) as u8,
            b.clamp(0, 255) as u8,
            a.clamp(0, 255) as u8,
        )
    }

    pub fn lerp(from: &Color, to: &Color, step: f32) -> Color {
        let r = (from.r as f32 * (1.0 - step) + to.r as f32 * step) as u8;
        let g = (from.g as f32 * (1.0 - step) + to.g as f32 * step) as u8;
        let b = (from.b as f32 * (1.0 - step) + to.b as f32 * step) as u8;
        let a = (from.a as f32 * (1.0 - step) + to.a as f32 * step) as u8;
        Color::new(r, g, b, a)
    }
}

impl Default for Color {
    fn default() -> Self {
        Color::none()
    }
}

use core::{fmt::{self}, write};

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let return_value = if self.is_none() { " " } else { "o" };

        write!(f, "{return_value}")
    }
}
