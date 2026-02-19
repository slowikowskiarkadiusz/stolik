extern crate alloc;
use alloc::vec::Vec;

use crate::{engine::v2::V2, scenes::tetris::shape::Shape};

pub struct Block {
    pub center: V2,
    pub shape: Shape,
    pub is_shadow: bool,
    pub rotation: u16,
}

impl Block {
    pub fn new(center: V2, shape: Shape, is_shadow: bool) -> Self {
        Self {
            center,
            shape,
            is_shadow,
            rotation: 0,
        }
    }

    pub fn get_kicks(&self, to: i32) -> Vec<V2> {
        todo!()
    }

    pub fn get_taken_spots(&self) -> Vec<V2> {
        todo!()
    }

    pub fn rotate_block(&self, by: i32) {
        todo!()
    }

    pub fn reset(&self) {
        todo!()
    }
}
