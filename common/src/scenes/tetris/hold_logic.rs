use crate::engine::v2::V2;

pub struct HoldLogic {}

impl HoldLogic {
    pub const SIZE: V2 = V2::new(4.0, 4.0);

    pub fn new(center: V2) -> Self {
        Self {}
    }
}
