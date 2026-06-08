extern crate alloc;

use crate::{
    engine::{color::Color, color_matrix::ColorMatrix, v2::V2},
    scenes::tetris::{
        board::{BLOCKS_COLORS, SCALE},
        shape::{Shape},
    },
};

pub struct Block {
    pub center: V2,
    pub shape: Shape,
    pub is_shadow: bool,
    pub rotation: u16,
    /// 1× logical matrix — used for collision detection and get_taken_spots().
    pub matrix: ColorMatrix,
    /// Pre-scaled SCALE× render matrix — updated at creation and after each rotation.
    matrix_scaled: ColorMatrix,
}

fn upscale(src: &ColorMatrix) -> ColorMatrix {
    let s = SCALE as u8;
    let mut dst = ColorMatrix::new(src.width * s, src.height * s, Color::none());
    for y in 0..src.height {
        for x in 0..src.width {
            let c = *src.get(x, y);
            for dy in 0..s {
                for dx in 0..s {
                    dst.set(x * s + dx, y * s + dy, c);
                }
            }
        }
    }
    dst
}

impl Block {
    pub fn new(center: V2, shape: Shape, is_shadow: bool) -> Self {
        let matrix = Block::generate_shape(&shape, 0, is_shadow);
        let matrix_scaled = upscale(&matrix);
        Self {
            center,
            matrix_scaled,
            matrix,
            shape,
            is_shadow,
            rotation: 0,
        }
    }

    pub fn generate_shape(shape: &Shape, rotation: i32, is_shadow: bool) -> ColorMatrix {
        let size = match shape {
            Shape::I => V2::new(4.0, 4.0),
            Shape::O => V2::new(4.0, 3.0),
            _ => V2::new(3.0, 3.0),
        };

        let mut result = ColorMatrix::new(size.x as u8, size.y as u8, Color::none());

        let color = if is_shadow {
            Color::white().a(204).clone()
        } else {
            BLOCKS_COLORS[*shape as usize]
        };

        match shape {
            Shape::I => {
                for i in 0..size.x as u8 {
                    result.set(i, 1, color);
                }
            }
            Shape::O => {
                for x in 0..2 {
                    for y in 0..2 {
                        result.set(x + 1, y, color);
                    }
                }
            }
            Shape::T => {
                result.set(0, 1, color);
                result.set(1, 1, color);
                result.set(2, 1, color);
                result.set(1, 0, color);
            }
            Shape::S => {
                result.set(0, 1, color);
                result.set(1, 1, color);
                result.set(1, 0, color);
                result.set(2, 0, color);
            }
            Shape::Z => {
                result.set(0, 0, color);
                result.set(1, 0, color);
                result.set(1, 1, color);
                result.set(2, 1, color);
            }
            Shape::J => {
                for i in 0..3 {
                    result.set(i, 1, color);
                }
                result.set(0, 0, color);
            }
            Shape::L => {
                for i in 0..3 {
                    result.set(i, 1, color);
                }
                result.set(2, 0, color);
            }
        }

        result.rotate(rotation as f32, Color::none());
        result
    }

    /// Returns the 4 world-space positions occupied by this block.
    /// A tetromino always has exactly 4 cells so the array is always full.
    pub fn get_taken_spots(&self) -> [V2; 4] {
        let start = &self.center.floor() - &V2::new((self.matrix.width / 2) as f32, (self.matrix.height / 2) as f32);
        let mut coords = [V2::zero(); 4];
        let mut count = 0usize;

        'outer: for x in start.x as i16..start.x as i16 + self.matrix.width as i16 {
            for y in start.y as i16..start.y as i16 + self.matrix.height as i16 {
                if !self.matrix.get((x - start.x as i16) as u8, (y - start.y as i16) as u8).is_none() {
                    coords[count] = V2::new(x as f32, y as f32);
                    count += 1;
                    if count == 4 {
                        break 'outer;
                    }
                }
            }
        }

        coords
    }

    /// Returns wall-kick offsets for the given rotation transition.
    /// Returns a static slice so no heap allocation is needed.
    pub fn get_kicks(&self, to: i32) -> &'static [V2] {
        let mut to_value = to;
        if to_value < 0 {
            to_value = 360 - to_value;
        }

        if self.shape == Shape::O {
            return &[];
        }

        let piece_key = if self.shape == Shape::I { "I" } else { "Others" };
        let transition = (to_value as u16 % 360, self.rotation % 360);

        let kick_key: &str = match transition {
            (0, 90)   => "0->90",
            (90, 0)   => "90->0",
            (90, 180) => "90->180",
            (180, 90) => "180->90",
            (180, 270) => "180->270",
            (270, 180) => "270->180",
            (270, 0)  => "270->0",
            (0, 270)  => "0->270",
            _ => return &[],
        };

        get_kicks_static(piece_key, kick_key).unwrap_or(&[])
    }

    pub fn rotate_block(&mut self, by: i32) {
        let mut by_value = by;
        if by_value < 0 {
            by_value = 360 + by_value;
        }
        self.rotation = (self.rotation + by_value as u16) % 360;
        let old = &self.matrix;
        let mut new_matrix = if by_value == 90 || by_value == 270 {
            ColorMatrix::new(old.height, old.width, Color::none())
        } else {
            ColorMatrix::new(old.width, old.height, Color::none())
        };
        for y in 0..old.height {
            for x in 0..old.width {
                let pixel = old.get(x, y);
                if by_value == 90 {
                    new_matrix.set(y, old.width - 1 - x, *pixel);
                } else if by_value == 180 {
                    new_matrix.set(old.width - 1 - x, old.height - 1 - y, *pixel);
                } else if by_value == 270 {
                    new_matrix.set(old.height - 1 - y, x, *pixel);
                } else {
                    new_matrix.set(x, y, *pixel);
                }
            }
        }
        self.matrix = new_matrix;
        self.matrix_scaled = upscale(&self.matrix);
    }

    pub fn render(&self) -> &ColorMatrix {
        &self.matrix
    }

    /// Returns the pre-scaled SCALE× render matrix.
    pub fn render_scaled(&self) -> &ColorMatrix {
        &self.matrix_scaled
    }
}

const KICKS: &[(&str, &[(&str, &[V2])])] = &[
    (
        "I",
        &[
            (
                "0->90",
                &[
                    V2::new(0.0, 0.0),
                    V2::new(-2.0, 0.0),
                    V2::new(1.0, 0.0),
                    V2::new(-2.0, -1.0),
                    V2::new(1.0, 2.0),
                ],
            ),
            (
                "90->0",
                &[
                    V2::new(0.0, 0.0),
                    V2::new(2.0, 0.0),
                    V2::new(-1.0, 0.0),
                    V2::new(2.0, 1.0),
                    V2::new(-1.0, -2.0),
                ],
            ),
            (
                "90->180",
                &[
                    V2::new(0.0, 0.0),
                    V2::new(-1.0, 0.0),
                    V2::new(2.0, 0.0),
                    V2::new(-1.0, 2.0),
                    V2::new(2.0, -1.0),
                ],
            ),
            (
                "180->90",
                &[
                    V2::new(0.0, 0.0),
                    V2::new(1.0, 0.0),
                    V2::new(-2.0, 0.0),
                    V2::new(1.0, -2.0),
                    V2::new(-2.0, 1.0),
                ],
            ),
            (
                "180->270",
                &[
                    V2::new(0.0, 0.0),
                    V2::new(2.0, 0.0),
                    V2::new(-1.0, 0.0),
                    V2::new(2.0, 1.0),
                    V2::new(-1.0, -2.0),
                ],
            ),
            (
                "270->180",
                &[
                    V2::new(0.0, 0.0),
                    V2::new(-2.0, 0.0),
                    V2::new(1.0, 0.0),
                    V2::new(-2.0, -1.0),
                    V2::new(1.0, 2.0),
                ],
            ),
            (
                "270->0",
                &[
                    V2::new(0.0, 0.0),
                    V2::new(1.0, 0.0),
                    V2::new(-2.0, 0.0),
                    V2::new(1.0, -2.0),
                    V2::new(-2.0, 1.0),
                ],
            ),
            (
                "0->270",
                &[
                    V2::new(0.0, 0.0),
                    V2::new(-1.0, 0.0),
                    V2::new(2.0, 0.0),
                    V2::new(-1.0, 2.0),
                    V2::new(2.0, -1.0),
                ],
            ),
        ],
    ),
    (
        "Others",
        &[
            (
                "0->90",
                &[
                    V2::new(0.0, 0.0),
                    V2::new(-1.0, 0.0),
                    V2::new(-1.0, 1.0),
                    V2::new(0.0, -2.0),
                    V2::new(-1.0, -2.0),
                ],
            ),
            (
                "90->0",
                &[
                    V2::new(0.0, 0.0),
                    V2::new(1.0, 0.0),
                    V2::new(1.0, -1.0),
                    V2::new(0.0, 2.0),
                    V2::new(1.0, 2.0),
                ],
            ),
            (
                "90->180",
                &[
                    V2::new(0.0, 0.0),
                    V2::new(1.0, 0.0),
                    V2::new(1.0, -1.0),
                    V2::new(0.0, 2.0),
                    V2::new(1.0, 2.0),
                ],
            ),
            (
                "180->90",
                &[
                    V2::new(0.0, 0.0),
                    V2::new(-1.0, 0.0),
                    V2::new(-1.0, 1.0),
                    V2::new(0.0, -2.0),
                    V2::new(-1.0, -2.0),
                ],
            ),
            (
                "180->270",
                &[
                    V2::new(0.0, 0.0),
                    V2::new(1.0, 0.0),
                    V2::new(1.0, 1.0),
                    V2::new(0.0, -2.0),
                    V2::new(1.0, -2.0),
                ],
            ),
            (
                "270->180",
                &[
                    V2::new(0.0, 0.0),
                    V2::new(-1.0, 0.0),
                    V2::new(-1.0, -1.0),
                    V2::new(0.0, 2.0),
                    V2::new(-1.0, 2.0),
                ],
            ),
            (
                "270->0",
                &[
                    V2::new(0.0, 0.0),
                    V2::new(-1.0, 0.0),
                    V2::new(-1.0, -1.0),
                    V2::new(0.0, 2.0),
                    V2::new(-1.0, 2.0),
                ],
            ),
            (
                "0->270",
                &[
                    V2::new(0.0, 0.0),
                    V2::new(1.0, 0.0),
                    V2::new(1.0, 1.0),
                    V2::new(0.0, -2.0),
                    V2::new(1.0, -2.0),
                ],
            ),
        ],
    ),
];

fn get_kicks_static(piece: &str, transition: &str) -> Option<&'static [V2]> {
    KICKS
        .iter()
        .find(|(p, _)| *p == piece)
        .and_then(|(_, transitions)| transitions.iter().find(|(t, _)| *t == transition))
        .map(|(_, kicks)| *kicks)
}
