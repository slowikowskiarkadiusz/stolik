use crate::{
    engine::{color::Color, color_matrix::ColorMatrix, v2::V2},
    scenes::tetris::{block::Block, shape::Shape},
};

pub struct HoldLogic {
    pub center: V2,
    color_matrix: ColorMatrix,
    held_piece: Option<Shape>,
}

impl HoldLogic {
    pub const SIZE: V2 = V2::new(4.0, 4.0);

    pub fn new(center: V2) -> Self {
        Self {
            center,
            held_piece: None,
            color_matrix: ColorMatrix::new(HoldLogic::SIZE.x as u8, HoldLogic::SIZE.y as u8, Color::none()),
        }
    }

    pub fn swap(&mut self, shape: Shape) -> Option<Shape> {
        let previous_piece = self.held_piece.clone();
        self.held_piece = Some(shape.clone());
        self.color_matrix = Block::generate_shape(&shape, 0, false);

        previous_piece
    }

    pub fn render(&self) -> &ColorMatrix {
        &self.color_matrix
    }
}
