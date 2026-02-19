use crate::engine::{color::Color, v2::V2};

pub struct GarbageBar {}

impl GarbageBar {
    pub fn new(center: V2, size: V2, color: Color) -> Self {
        Self {}
    }

    pub fn decrease_and_get_left(&self, count: u8) -> u8 {
        todo!()
    }

    pub fn add_lines(&self, count: u8) {
        todo!()
    }

    pub fn pop(&self) -> bool {
        todo!()
    }

    pub fn tick(&self, delta_time: f32) {
        todo!()
    }
}
