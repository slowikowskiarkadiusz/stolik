extern crate alloc;
use crate::{
    engine::{
        color::Color,
        color_matrix::ColorMatrix,
        components::{physics::Physics, transform::Transform, world::World},
        engine::ActorId,
        v2::V2,
    },
    scenes::utils::lerp_f32,
    write_m,
};
use alloc::vec;
use core::f32::consts::PI;
use libm::{cos, sin};
use rand::{Rng, SeedableRng, rngs::SmallRng};

const PARTICLES_COUNT: u8 = 4;
const PARTICLE_FORCE: f32 = 4.0;
const MAX_PARTICLE_ANGLE: f32 = 35.0;
const PARTICLES_DURATION: f32 = 2.0;

pub struct BoostParticles {
    particle_actors: Vec<ActorId>,
    original_colors: Vec<Color>,
    timer: f32,
}

impl BoostParticles {
    pub fn new(range: (V2, V2), ship_throttle_dir: V2, world: &mut World) -> Self {
        let mut rng: SmallRng = SmallRng::seed_from_u64(embassy_time::Instant::now().as_micros());
        Self {
            particle_actors: (0..PARTICLES_COUNT)
                .map(|| spawn_particle(get_random_point(range, &mut rng), ship_throttle_dir * -1.0, &mut rng, world)),
            original_colors: (0..PARTICLES_COUNT)
                .map(|| spawn_particle(get_random_point(range, &mut rng), ship_throttle_dir * -1.0, &mut rng, world)),
            timer: PARTICLES_DURATION,
        }
    }

    pub fn render(&mut self, world: &mut World, out: &mut ColorMatrix, delta_time: f32) {
        self.timer -= delta_time;

        if self.timer <= 0.0 {
            for particle_actor_id in self.particle_actors {
                world.murder(&particle_actor_id);
            }
        } else {
            for particle_actor_id in self.particle_actors {
                if let Some(particle_transform) = world.get_transform(&particle_actor_id) {
                    out.set(particle_transform.center.x as u8, particle_transform.center.y as u8, to);
                }
            }
        }
    }
}

fn get_random_point(range: (V2, V2), rng: &mut SmallRng) -> V2 {
    let t = rng.r#gen::<f32>();
    V2::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)
}

fn spawn_particle(at: V2, direction: V2, rng: &mut SmallRng, world: &mut World) -> ActorId {
    let degrees_angle = lerp_f32(0.0, MAX_PARTICLE_ANGLE, rng.r#gen::<f32>());
    let radians = (degrees_angle * PI / 180.0) as f64;
    let x = direction.x as f64;
    let y = direction.y as f64;
    let direction = V2::new(
        (x * cos(radians) - y * sin(radians)) as f32,
        (x * sin(radians) + y * cos(radians)) as f32,
    );

    let mut physics = Physics::new().with_mass(1.0).with_drag(0.5);
    physics.add_force(direction * PARTICLE_FORCE);

    world.add_new_actor(Some(Transform::new(at, V2::one())), None, Some(physics.into()))
}
