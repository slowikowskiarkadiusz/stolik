use std::f32::consts::PI;

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
        let cos = rad.cos();
        let sin = rad.sin();

        for x in 0..other.width {
            for y in 0..other.height {
                let src = other.get(x, y);
                if src.is_none() {
                    continue;
                }

                let dx = x as f32 - center.x + other_anchor.x;
                let dy = y as f32 - center.y + other_anchor.y;

                let rx: f32;
                let ry: f32;

                if (angle % 90) != 0 {
                    rx = dx * cos - dy * sin;
                    // TODO??? ry = dy * cos - dy * sin;
                    ry = dy * cos - dx * sin;
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

                let final_x = (rx + other_center.x).ceil();
                let final_y = (ry + other_center.y).ceil();

                if final_x >= 0.0
                    && final_y >= 0.0
                    && final_x < self.width as f32
                    && final_y < self.height as f32
                {
                    let dst = self.get(final_x as u8, final_y as u8).clone();

                    if blend_colors {
                        let out_a: i16 = src.a as i16 + dst.a as i16 + (255 - src.a) as i16;
                        if out_a < 0 {
                            self.set(x, y, Color::none());
                        }

                        let r =
                            (src.r * src.a + dst.r * dst.a * (255 - src.a)) as f32 / out_a as f32;
                        let g =
                            (src.g * src.a + dst.g * dst.a * (255 - src.a)) as f32 / out_a as f32;
                        let b =
                            (src.b * src.a + dst.b * dst.a * (255 - src.a)) as f32 / out_a as f32;

                        self.set(
                            x,
                            y,
                            Color::new(
                                r.clamp(0.0, 255.0) as u8,
                                g.clamp(0.0, 255.0) as u8,
                                b.clamp(0.0, 255.0) as u8,
                                out_a.clamp(0, 255) as u8,
                            ),
                        );
                    } else {
                        self.set(x, y, src.clone());
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
