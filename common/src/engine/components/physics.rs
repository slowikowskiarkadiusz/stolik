use crate::engine::v2::V2;

#[allow(dead_code)]
pub struct Physics {
    velocity: V2,
    angular_velocity: f32,
    is_fixed: bool,
    mass: f32,
    drag: f32,
    angular_drag: f32,
    rotation: f32,
    inertia: f32,
    bounciness: f32,
    max_distance_from_center: f32,
}

impl Physics {
    pub fn new() -> Self {
        Self {
            velocity: V2::zero(),
            angular_velocity: 0.0,
            is_fixed: false,
            mass: 0.0,
            drag: 0.0,
            angular_drag: 0.0,
            rotation: 0.0,
            inertia: 0.0,
            bounciness: 0.0,
            max_distance_from_center: 0.0,
        }
    }

    pub fn with_is_fixed(&mut self, is_fixed: bool) -> &Self {
        self.is_fixed = is_fixed;
        self
    }
}
