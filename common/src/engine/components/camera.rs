use crate::engine::{
    components::world::World,
    engine::{ActorId, SCREEN_SIZE},
    v2::V2,
};

#[derive(Clone, Copy)]
pub struct Viewport {
    pub from: V2,
    pub to: V2,
}

impl Viewport {
    pub fn new(from: V2, to: V2) -> Self {
        Self { from, to }
    }

    pub fn can_see_x(&self, x: f32) -> bool {
        (self.from.x..self.to.x).contains(&x)
    }

    pub fn can_see_y(&self, y: f32) -> bool {
        (self.from.y..self.to.y).contains(&y)
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

    pub fn get_size(&self) -> V2 {
        &self.to - &self.from
    }
}

#[derive(Clone, Copy)]
pub struct Camera {
    viewport: Viewport,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            viewport: Viewport::new(V2::zero(), &V2::one() * SCREEN_SIZE),
        }
    }

    pub fn get_center(&self) -> V2 {
        (&self.viewport.from + &self.viewport.to) / 2.0
    }

    pub fn get_viewport(&self) -> Viewport {
        self.viewport.clone()
    }

    pub fn set_center(&mut self, to: V2) {
        let size = &self.viewport.to - &self.viewport.from;
        self.viewport.from = &to - &(size / 2.0);
        self.viewport.to = &to + &(size / 2.0);
    }

    pub fn set_x(&mut self, to: f32) {
        let size = &self.viewport.to.x - &self.viewport.from.x;
        self.viewport.from.x = to - (size / 2.0);
        self.viewport.to.x = to + (size / 2.0);
    }

    pub fn set_y(&mut self, to: f32) {
        let size = &self.viewport.to.y - &self.viewport.from.y;
        self.viewport.from.y = to - (size / 2.0);
        self.viewport.to.y = to + (size / 2.0);
    }

    pub fn get_viewport_size_relative_to_screen(&self) -> f32 {
        (self.viewport.to.x - self.viewport.from.x) / (SCREEN_SIZE as f32)
    }

    pub fn zoom(&mut self, multiplier: f32) {
        let center = self.get_center();

        self.viewport.from = (&self.viewport.from - &center) * multiplier;
        self.viewport.to = (&self.viewport.to - &center) * multiplier;
    }

    pub fn can_see(&self, point: V2) -> bool {
        self.viewport.can_see(point)
    }

    pub fn can_see_actor(&self, actor_id: ActorId, world: &World) -> bool {
        self.viewport.can_see_actor(actor_id, world)
    }
}
