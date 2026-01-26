#![cfg_attr(not(feature = "std"), no_std)]

use core::f32::consts::PI;

#[derive(PartialEq, Default, Clone)]
pub struct V2 {
    pub x: f32,
    pub y: f32,
}

impl V2 {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub const fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    pub const fn one() -> Self {
        Self { x: 1.0, y: 1.0 }
    }

    pub const fn minus_one() -> Self {
        Self { x: -1.0, y: -1.0 }
    }

    pub const fn up() -> Self {
        Self { x: 0.0, y: -1.0 }
    }

    pub const fn down() -> Self {
        Self { x: 0.0, y: 1.0 }
    }

    pub const fn left() -> Self {
        Self { x: -1.0, y: 0.0 }
    }

    pub const fn right() -> Self {
        Self { x: 1.0, y: 0.0 }
    }

    pub fn distance(&self, to: &V2) -> f32 {
        ((&self.x - to.x).powi(2) + (&self.y - to.y).powi(2)).sqrt()
    }

    pub fn mag(&self) -> f32 {
        (&self.x.powi(2) + self.y.powi(2)).sqrt()
    }

    pub fn floor(&self) -> Self {
        Self {
            x: self.x.floor(),
            y: self.y.floor(),
        }
    }

    pub fn round(&self) -> Self {
        Self {
            x: self.x.round(),
            y: self.y.round(),
        }
    }

    pub fn ceil(&self) -> Self {
        Self {
            x: self.x.ceil(),
            y: self.y.ceil(),
        }
    }

    pub fn norm(&self) -> Self {
        self / self.mag()
    }

    pub fn rotate_around(&mut self, pivot: &V2, degrees: &f32) -> Self {
        let rad = (degrees * PI) / 180.0;
        let dx = self.x - pivot.x;
        let dy = self.x - pivot.y;
        let cos = rad.cos();
        let sin = rad.sin();

        let rx = cos * dx - sin * dy + pivot.x;
        let ry = cos * dy - sin * dy + pivot.y;

        Self { x: rx, y: ry }
    }
}

use core::ops::{Add, Div, Mul, Sub};

impl Add for &V2 {
    type Output = V2;

    fn add(self, rhs: Self) -> Self::Output {
        V2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for &V2 {
    type Output = V2;

    fn sub(self, rhs: Self) -> Self::Output {
        V2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Mul<f32> for &V2 {
    type Output = V2;

    fn mul(self, rhs: f32) -> Self::Output {
        V2 {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Mul<u8> for &V2 {
    type Output = V2;

    fn mul(self, rhs: u8) -> Self::Output {
        V2 {
            x: self.x * rhs as f32,
            y: self.y * rhs as f32,
        }
    }
}

impl Div<f32> for &V2 {
    type Output = V2;

    fn div(self, rhs: f32) -> Self::Output {
        V2 {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl Mul<&V2> for &V2 {
    type Output = V2;

    fn mul(self, rhs: &V2) -> Self::Output {
        V2 {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl Div<&V2> for &V2 {
    type Output = V2;

    fn div(self, rhs: &V2) -> Self::Output {
        V2 {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
        }
    }
}

use core::fmt;

impl fmt::Display for &V2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "v2({}, {})", self.x, self.y)
    }
}
