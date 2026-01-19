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
                        self.set(x, y, other.at(x, y).clone());
                    }
                }
            }
        }

        self
    }
}
