use crate::engine::{
    components::world::World,
    engine::{ActorId, SCREEN_SIZE},
    v2::V2,
};

pub struct Camera {
    viewport: (V2, V2),
}

impl Camera {
    pub fn new() -> Self {
        Self {
            viewport: (V2::zero(), &V2::one() * SCREEN_SIZE),
        }
    }

    pub fn get_center(&self) -> V2 {
        (&self.viewport.0 + &self.viewport.1) / 2.0
    }

    pub fn get_viewport_bounds(&self) -> (V2, V2) {
        self.viewport
    }

    pub fn get_viewport_size(&self) -> V2 {
        &self.viewport.1 - &self.viewport.0
    }

    pub fn set_center(&mut self, to: V2) {
        let size = &self.viewport.1 - &self.viewport.0;
        self.viewport.0 = &to - &(size / 2.0);
        self.viewport.1 = &to + &(size / 2.0);
    }

    pub fn set_x(&mut self, to: f32) {
        let size = &self.viewport.1.x - &self.viewport.0.x;
        self.viewport.0.x = to - (size / 2.0);
        self.viewport.1.x = to + (size / 2.0);
    }

    pub fn set_y(&mut self, to: f32) {
        let size = &self.viewport.1.y - &self.viewport.0.y;
        self.viewport.0.y = to - (size / 2.0);
        self.viewport.1.y = to + (size / 2.0);
    }

    pub fn get_viewport_size_relative_to_screen(&self) -> f32 {
        (self.viewport.1.x - self.viewport.0.x) / (SCREEN_SIZE as f32)
    }

    pub fn zoom(&mut self, multiplier: f32) {
        let center = self.get_center();

        self.viewport.0 = (&self.viewport.0 - &center) * multiplier;
        self.viewport.1 = (&self.viewport.1 - &center) * multiplier;
    }

    pub fn can_see_x(&self, x: f32) -> bool {
        (self.viewport.0.x..self.viewport.1.x).contains(&x)
    }

    pub fn can_see_y(&self, y: f32) -> bool {
        (self.viewport.0.y..self.viewport.1.y).contains(&y)
    }

    pub fn can_see(&self, point: V2) -> bool {
        self.can_see_x(point.x) && self.can_see_x(point.y)
    }

    pub fn can_see_actor(&self, actor_id: ActorId, world: &World) -> bool {
        if let Some(actor_transform) = world.get_transform(&actor_id) {
            return self.can_see(actor_transform.center);
        }
        false
    }
}
