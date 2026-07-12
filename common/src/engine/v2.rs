extern crate alloc;
use core::f32::consts::PI;
use core::ops::{AddAssign, SubAssign};
use libm::{ceilf, cosf, floorf, powf, roundf, sinf, sqrtf};

#[derive(PartialEq, Default, Clone, Copy)]
pub struct V2 {
    pub x: f32,
    pub y: f32,
}

pub fn v2(x: f32, y: f32) -> V2 {
    V2::new(x, y)
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
        sqrtf(powf(&self.x - to.x, 2.0) + powf(&self.y - to.y, 2.0))
    }

    pub fn mag(&self) -> f32 {
        sqrtf(powf(self.x, 2.0) + powf(self.y, 2.0))
    }

    pub fn floor(&self) -> Self {
        Self {
            x: floorf(self.x),
            y: floorf(self.y),
        }
    }

    pub fn round(&self) -> Self {
        Self {
            x: roundf(self.x),
            y: roundf(self.y),
        }
    }

    pub fn ceil(&self) -> Self {
        Self {
            x: ceilf(self.x),
            y: ceilf(self.y),
        }
    }

    pub fn norm(&self) -> Self {
        self / self.mag()
    }

    pub fn rotate_around(&self, pivot: &V2, degrees: &f32) -> Self {
        let rad = (degrees * PI) / 180.0;
        let dx = self.x - pivot.x;
        let dy = self.y - pivot.y;
        let cos = cosf(rad);
        let sin = sinf(rad);

        let rx = cos * dx - sin * dy + pivot.x;
        let ry = sin * dx + cos * dy + pivot.y;

        Self { x: rx, y: ry }
    }

    pub fn dot(&self, other: &V2) -> f32 {
        self.x * other.x + self.y * other.y
    }

    /// 2D scalar cross product: self × other = self.x*other.y − self.y*other.x
    /// Positive = other is counterclockwise from self (in math coords).
    pub fn cross(&self, other: &V2) -> f32 {
        self.x * other.y - self.y * other.x
    }

    /// Rotate this vector around the origin by `degrees` (positive = CCW in math / CW on screen).
    pub fn rotate(&self, degrees: f32) -> Self {
        self.rotate_around(&V2::new(0.0, 0.0), &degrees)
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

impl Add for V2 {
    type Output = V2;
    fn add(self, rhs: Self) -> Self::Output {
        V2 { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}

impl AddAssign for V2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for V2 {
    type Output = V2;
    fn sub(self, rhs: Self) -> Self::Output {
        V2 { x: self.x - rhs.x, y: self.y - rhs.y }
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

impl SubAssign for V2 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
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

impl Mul<f32> for V2 {
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

impl Div<f32> for V2 {
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

impl fmt::Display for V2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "v2({}, {})", self.x, self.y)
    }
}
