use crate::{
    engine::{color_matrix::ColorMatrix, v2::V2},
    scenes::tetris::shape::Shape,
};

pub struct HoldLogic {
    pub center: V2,
}

impl HoldLogic {
    pub const SIZE: V2 = V2::new(4.0, 4.0);

    pub fn new(center: V2) -> Self {
        Self {}
    }

    pub fn swap(&mut self, shape: Shape) -> Option<Shape> {
        todo!()
    }

    pub fn render(&self) -> ColorMatrix {
        todo!()
    }
}
