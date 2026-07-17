extern crate alloc;

use crate::engine::{
    color::Color,
    color_matrix::ColorMatrix,
    components::{
        camera::Camera,
        collider::{Collider, ColliderPart},
        physics::Physics,
        transform::Transform,
        world::World,
    },
    engine::{ActorId, SCREEN_SIZEF32},
    v2::V2,
};

use super::astro_obstacle::AstroObstacleMap;

const BULLET_LIFETIME: f32 = 2.5;
pub const BULLET_SIZE: f32 = 2.0;

pub struct Bullet {
    pub actor_id: ActorId,
    pub owner_is_p1: bool,
    pub hit: bool,
    lifetime: f32,
}

impl Bullet {
    pub fn new(world: &mut World, pos: V2, velocity: V2, owner_is_p1: bool) -> Self {
        let mut physics = Physics::new();
        physics.with_can_move(true).with_mass(0.1).with_drag(0.0);

        let id = world.add_new_actor(
            Some(Transform::new(pos, V2::new(BULLET_SIZE, BULLET_SIZE))),
            Some(Collider::new(
                alloc::vec![ColliderPart::rect(V2::zero(), V2::new(BULLET_SIZE, BULLET_SIZE), true)],
                Some(0),
                false
            )),
            Some(physics),
            None,
        );
        world.get_mut_physics(&id).unwrap().set_velocity(velocity);

        Self {
            actor_id: id,
            owner_is_p1,
            hit: false,
            lifetime: BULLET_LIFETIME,
        }
    }

    /// Returns true if bullet should be destroyed.
    pub fn tick(&mut self, world: &mut World, _obstacle: &AstroObstacleMap, delta_time: f32) -> bool {
        self.lifetime -= delta_time;
        wrap_center(world, self.actor_id);
        self.hit || self.lifetime <= 0.0
    }

    pub fn pos(&self, world: &World) -> Option<V2> {
        world.get_transform(&self.actor_id).map(|t| t.center)
    }

    pub fn destroy(&self, world: &mut World) {
        world.murder(&self.actor_id);
    }

    pub fn render(&self, world: &World, _camera: &Camera, result: &mut ColorMatrix) {
        if let Some(t) = world.get_transform(&self.actor_id) {
            let color = if self.owner_is_p1 {
                Color::new(255, 255, 80, 255)
            } else {
                Color::new(80, 200, 255, 255)
            };
            result.write(
                &ColorMatrix::new(BULLET_SIZE as u8, BULLET_SIZE as u8, color),
                &t.center,
                None,
                None,
                None,
                None,
            );
        }
    }
}

pub fn wrap_center(world: &mut World, actor_id: ActorId) {
    if let Some(t) = world.get_mut_transform(&actor_id) {
        let s = SCREEN_SIZEF32;
        t.center.x = ((t.center.x % s) + s) % s;
        t.center.y = ((t.center.y % s) + s) % s;
    }
}
