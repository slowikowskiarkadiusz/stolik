extern crate alloc;
use alloc::vec::Vec;

use crate::engine::{
    color::Color,
    color_matrix::ColorMatrix,
    components::{physics::Physics, transform::Transform, world::World},
    engine::ActorId,
    v2::V2,
};
use rand::{Rng, SeedableRng, rngs::SmallRng};

const PARTICLES_COUNT: u8 = 5;
const PARTICLES_SPEED: f32 = 12.0;
const MAX_PARTICLE_ANGLE: f32 = 40.0;
const PARTICLE_LIFETIME: f32 = 1.0;
const PARTICLE_COLOR: Color = Color::new(255, 140, 30, 220);

pub struct BoostParticles {
    particle_actors: Vec<ActorId>,
    lifetime: f32,
}

impl BoostParticles {
    pub fn new(pos: V2, dir: V2, world: &mut World) -> Self {
        let mut rng = SmallRng::seed_from_u64(embassy_time::Instant::now().as_micros());
        let particle_actors = (0..PARTICLES_COUNT)
            .map(|_| spawn_particle(pos, dir, world, &mut rng))
            .collect();
        Self { particle_actors, lifetime: PARTICLE_LIFETIME }
    }

    /// returns true when expired
    pub fn tick(&mut self, delta_time: f32, world: &mut World) -> bool {
        self.lifetime -= delta_time;
        if self.lifetime <= 0.0 {
            for id in &self.particle_actors {
                world.murder(id);
            }
            return true;
        }
        false
    }

    pub fn render(&self, world: &World, result: &mut ColorMatrix) {
        let alpha = ((self.lifetime / PARTICLE_LIFETIME) * 255.0) as u8;
        let color = Color::new(PARTICLE_COLOR.r, PARTICLE_COLOR.g, PARTICLE_COLOR.b, alpha);
        for id in &self.particle_actors {
            if let Some(t) = world.get_transform(id) {
                let x = t.center.x as u8;
                let y = t.center.y as u8;
                result.set(x, y, color);
            }
        }
    }
}

fn spawn_particle(pos: V2, dir: V2, world: &mut World, rng: &mut SmallRng) -> ActorId {
    let spread_pos = V2::new(
        pos.x + rng.gen_range(-1.0..1.0),
        pos.y + rng.gen_range(-1.0..1.0),
    );

    let mut physics = Physics::new();
    physics.with_can_move(true).with_mass(0.1).with_drag(5.0);
    let id = world.add_new_actor(
        Some(Transform::new(spread_pos, V2::one())),
        None,
        Some(physics),
    );

    let angle_offset = rng.gen_range(-MAX_PARTICLE_ANGLE..MAX_PARTICLE_ANGLE);
    let vel = dir.rotate(angle_offset) * PARTICLES_SPEED;
    if let Some(p) = world.get_mut_physics(&id) {
        p.set_velocity(vel);
    }
    id
}
