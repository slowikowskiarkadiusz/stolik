extern crate alloc;
use alloc::{format, vec::Vec};

use crate::{
    engine::{color::Color, color_matrix::ColorMatrix, v2::V2},
    scenes::tetris::{
        board::BLOCKS_COLORS,
        shape::{self, Shape},
    },
};

pub struct Block {
    pub center: V2,
    pub shape: Shape,
    pub is_shadow: bool,
    pub rotation: u16,
    pub matrix: ColorMatrix,
}

impl Block {
    pub fn new(center: V2, shape: Shape, is_shadow: bool) -> Self {
        Self {
            center,
            matrix: Block::generate_shape(&shape, 0, is_shadow),
            shape,
            is_shadow,
            rotation: 0,
        }
    }

    fn generate_shape(shape: &Shape, rotation: i32, is_shadow: bool) -> ColorMatrix {
        let size = match shape {
            Shape::I => V2::new(4.0, 4.0),
            Shape::O => V2::new(4.0, 3.0),
            _ => V2::new(3.0, 3.0),
        };

        let mut result = ColorMatrix::new(size.x as u8, size.y as u8, Color::none());

        let color = if is_shadow {
            Color::white().a(204).clone()
        } else {
            BLOCKS_COLORS[shape.clone() as usize].clone()
        };

        match shape {
            Shape::I => {
                for i in 0..size.x as u8 {
                    result.set(i, 1, color.clone());
                }
            }
            Shape::O => {
                for x in 0..2 {
                    for y in 0..2 {
                        result.set(x + 1, y, color.clone());
                    }
                }
            }
            Shape::T => {
                result.set(0, 1, color.clone());
                result.set(1, 1, color.clone());
                result.set(2, 1, color.clone());
                result.set(1, 0, color.clone());
            }
            Shape::S => {
                result.set(0, 1, color.clone());
                result.set(1, 1, color.clone());
                result.set(1, 0, color.clone());
                result.set(2, 0, color.clone());
            }
            Shape::Z => {
                result.set(0, 0, color.clone());
                result.set(1, 0, color.clone());
                result.set(1, 1, color.clone());
                result.set(2, 1, color.clone());
            }
            Shape::J => {
                for i in 0..3 {
                    result.set(i, 1, color.clone());
                }
                result.set(0, 0, color.clone());
            }
            Shape::L => {
                for i in 0..3 {
                    result.set(i, 1, color.clone());
                }
                result.set(2, 0, color.clone());
            }
        }

        result.rotate(rotation as f32, Color::none());
        result
    }

    pub fn get_kicks(&self, to: i32) -> Vec<V2> {
        let mut to_value = to;
        if to_value < 0 {
            to_value = 360 - to_value;
        }

        let mut kick_key = format!("{}->{}", to_value % 360, self.rotation % 360);

        if self.shape == Shape::O {
            return Vec::new();
        }

        let piece_key = if self.shape == Shape::I { "I" } else { "Others" };
        let mut result = Vec::<V2>::new();

        if let Some(raw) = get_kicks(piece_key, &kick_key) {
            for v2 in raw {
                result.push(v2.clone());
            }
        }

        result
    }

    pub fn get_taken_spots(&self) -> Vec<V2> {
        let start = &self.center.floor() - &V2::new((self.matrix.width / 2) as f32, (self.matrix.height / 2) as f32);
        let mut coords = Vec::<V2>::new();

        for x in start.x as u8..start.x as u8 + self.matrix.width {
            for y in start.y as u8..start.y as u8 + self.matrix.height {
                if !self.matrix.get(x - start.x as u8, y - start.y as u8).is_none() {
                    coords.push(V2::new(x as f32, y as f32));
                }
            }
        }

        coords
    }

    pub fn rotate_block(&mut self, by: i32) {
        let mut by_value = by;
        if by_value < 0 {
            by_value = 360 + by_value;
        }
        self.rotation = (self.rotation + by_value as u16) % 360;
        self.matrix = if by_value == 90 || by_value == 270 {
            ColorMatrix::new(self.matrix.height, self.matrix.width, Color::none())
        } else {
            ColorMatrix::new(self.matrix.width, self.matrix.height, Color::none())
        };

        for y in 0..self.matrix.height {
            for x in 0..self.matrix.width {
                let pixel = self.matrix.get(x, y);

                if by_value == 90 {
                    self.matrix.set(y, self.matrix.width - 1 - x, pixel.clone());
                } else if by_value == 180 {
                    self.matrix.set(self.matrix.width - 1 - x, self.matrix.width - 1 - y, pixel.clone());
                } else if by_value == 270 {
                    self.matrix.set(self.matrix.height - 1 - y, x, pixel.clone());
                } else {
                    self.matrix.set(x, y, pixel.clone());
                }
            }
        }
    }

    pub fn render(&self) -> &ColorMatrix {
        &self.matrix
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

fn get_kicks(piece: &str, transition: &str) -> Option<&'static [V2]> {
    KICKS
        .iter()
        .find(|(p, _)| *p == piece)
        .and_then(|(_, transitions)| transitions.iter().find(|(t, _)| *t == transition))
        .map(|(_, kicks)| *kicks)
}
