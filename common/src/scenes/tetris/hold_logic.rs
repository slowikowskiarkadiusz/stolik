use crate::{
    engine::{color::Color, color_matrix::ColorMatrix, v2::V2},
    scenes::tetris::{block::Block, board::SCALE, shape::Shape},
};

pub struct HoldLogic {
    pub center: V2,
    /// Pre-scaled at SCALE× — render() returns directly, no per-frame scaling.
    color_matrix: ColorMatrix,
    held_piece: Option<Shape>,
}

impl HoldLogic {
    pub const SIZE: V2 = V2::new(4.0, 4.0);

    pub fn new(center: V2) -> Self {
        let s = SCALE as u8;
        Self {
            center,
            held_piece: None,
            color_matrix: ColorMatrix::new(
                HoldLogic::SIZE.x as u8 * s,
                HoldLogic::SIZE.y as u8 * s,
                Color::none(),
            ),
        }
    }

    pub fn swap(&mut self, shape: Shape) -> Option<Shape> {
        let previous_piece = self.held_piece.clone();
        self.held_piece = Some(shape.clone());
        // render_scaled() already returns the SCALE× matrix — no per-frame scaling needed.
        let block = Block::new(V2::zero(), shape, false);
        let scaled = block.render_scaled();
        let off_x = (self.color_matrix.width.saturating_sub(scaled.width)) / 2;
        let off_y = (self.color_matrix.height.saturating_sub(scaled.height)) / 2;
        for y in 0..self.color_matrix.height {
            for x in 0..self.color_matrix.width {
                self.color_matrix.set(x, y, Color::none());
            }
        }
        for y in 0..scaled.height {
            for x in 0..scaled.width {
                self.color_matrix.set(off_x + x, off_y + y, *scaled.get(x, y));
            }
        }
        previous_piece
    }

    pub fn render(&self) -> &ColorMatrix {
        &self.color_matrix
    }
}
