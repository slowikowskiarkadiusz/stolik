use core::f32::consts::PI;

use libm::{ceilf, cosf, roundf, sinf};

use crate::engine::{color::Color, matrix::Matrix, v2::V2};

pub type ColorMatrix = Matrix<Color>;

impl Matrix<Color> {
    pub fn write_at_origin(&mut self, other: &ColorMatrix, origin: &V2) -> &ColorMatrix {
        if origin.x < other.width as f32 && origin.y < other.height as f32 {
            for x in 0..other.width {
                for y in 0..other.height {
                    let tx = x + origin.x as u8;
                    let ty = y + origin.y as u8;

                    if tx < self.width && ty < self.height {
                        self.set(x, y, other.get(x, y).clone());
                    }
                }
            }
        }

        self
    }

    pub fn write(
        &mut self,
        other: &ColorMatrix,
        other_center: &V2,
        other_rotation: Option<f32>,
        other_anchor: Option<V2>,
        blend_colors: Option<bool>,
    ) -> &ColorMatrix {
        let other_rotation = other_rotation.unwrap_or(0.0);
        let other_anchor = other_anchor.unwrap_or(V2::zero());
        let blend_colors = blend_colors.unwrap_or(true);

        if other.width == 0 || other.height == 0 {
            return self;
        }

        let center = V2::new(other.width as f32 / 2.0, other.height as f32 / 2.0);
        let mut angle: i16 = (other_rotation % 360.0) as i16;
        if angle < 0 {
            angle += 360;
        }
        let rad = other_rotation * PI / 180.0;
        let cos = cosf(rad);
        let sin = sinf(rad);

        for x in 0..other.width {
            for y in 0..other.height {
                let src = other.get(x, y);

                // Pomijaj tylko całkowicie przezroczyste piksele
                if src.a == 0 {
                    continue;
                }

                let dx = x as f32 - center.x + other_anchor.x;
                let dy = y as f32 - center.y + other_anchor.y;

                let rx: f32;
                let ry: f32;

                if (angle % 90) != 0 {
                    rx = dx * cos - dy * sin;
                    ry = dx * sin + dy * cos;
                } else {
                    if angle == 90 {
                        rx = -dy;
                        ry = dx;
                    } else if angle == 180 {
                        rx = -dx;
                        ry = -dy;
                    } else if angle == 270 {
                        rx = dy;
                        ry = -dx;
                    } else {
                        rx = dx;
                        ry = dy;
                    }
                }

                let final_x = ceilf(rx + other_center.x);
                let final_y = ceilf(ry + other_center.y);

                if final_x >= 0.0 && final_y >= 0.0 && final_x < self.width as f32 && final_y < self.height as f32 {
                    if blend_colors {
                        let dst = self.get(final_x as u8, final_y as u8);

                        let sa = src.a as f32 / 255.0;
                        let da = dst.a as f32 / 255.0;
                        let out_a = sa + da * (1.0 - sa);

                        if out_a <= 0.0 {
                            self.set(final_x as u8, final_y as u8, Color::none());
                        } else {
                            let inv_sa = 1.0 - sa;
                            let r = (src.r as f32 * sa + dst.r as f32 * da * inv_sa) / out_a;
                            let g = (src.g as f32 * sa + dst.g as f32 * da * inv_sa) / out_a;
                            let b = (src.b as f32 * sa + dst.b as f32 * da * inv_sa) / out_a;

                            self.set(
                                final_x as u8,
                                final_y as u8,
                                Color::new(roundf(r) as u8, roundf(g) as u8, roundf(b) as u8, roundf(out_a * 255.0) as u8),
                            );
                        }
                    } else {
                        self.set(final_x as u8, final_y as u8, src.clone());
                    }
                }
            }
        }

        self
    }

    pub fn dim(&mut self, to_opacity: u8) {
        for x in 0..self.width {
            for y in 0..self.height {
                let mut color = self.get(x, y).clone();
                color.a *= to_opacity;
                self.set(x, y, color);
            }
        }
    }
}
