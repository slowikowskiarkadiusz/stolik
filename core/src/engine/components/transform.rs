use crate::engine::v2::V2;

pub struct Transform {
    pub center: V2,
    pub size: V2,
    pub anchor_offset: V2,
    pub original_size: V2,
    pub rotation: f32,
}

impl Transform {
    pub fn new(center: V2, size: V2) -> Self {
        Self {
            center,
            size,
            anchor_offset: V2::zero(),
            original_size: V2::zero(),
            rotation: 0.0,
        }
    }
}
