extern crate alloc;
use alloc::{boxed::Box, string::ToString, vec::Vec};
use rand::{Rng, SeedableRng, rngs::SmallRng};

use crate::{
    engine::{
        color::Color,
        color_matrix::ColorMatrix,
        components::{
            collider::{Collider, ColliderPart, ColliderType},
            physics::Physics,
            transform::Transform,
            world::World,
        },
        engine::{ActorId, SCREEN_SIZE, open_scene},
        hash_map::HashMap,
        input::{input::Input, key::Key},
        scene::Scene,
        v2::V2,
    },
    scenes::utils::print_victory_text,
};

static ORIGINAL_BALL_SPEED: f32 = 10.0;

pub struct PhysicsTestScene {
    ball: ActorId,
    ball_2: ActorId,
}

impl Scene for PhysicsTestScene {
    fn init(&mut self, world: &mut World) {
        let screen_size = SCREEN_SIZE as f32;
        let size_factor = screen_size / 32.0;
        self.ball = create_rectangle_actor(
            world,
            V2::one() * screen_size / 2.0,
            V2::one() * 2.0 * size_factor,
            Color::white(),
            Some("ball"),
        );
        self.ball_2 = create_rectangle_actor(
            world,
            &(V2::one() * screen_size / 2.0) - &V2::new(0.0, -5.0),
            V2::one() * 2.0 * size_factor,
            Color::red(),
            Some("ball"),
        );
    }

    fn tick(&mut self, input: &Box<dyn Input>, world: &mut World, delta_time: f32) {
        self.handle_input(input, world, delta_time);
    }

    fn on_overlaps(&mut self, overlaps: &HashMap<ActorId, Vec<ActorId>>, world: &mut World, _delta_time: f32) {}
}

impl PhysicsTestScene {
    pub fn new() -> Self {
        Self { ball: 0, ball_2: 0 }
    }

    fn handle_input(&mut self, input: &Box<dyn Input + 'static>, world: &mut World, delta_time: f32) {
        if let Some(ball_physics) = world.get_mut_physics(&self.ball) {
            let is_left = input.is_key_press(Key::P1Left);
            let is_right = input.is_key_press(Key::P1Right);
            let is_down = input.is_key_press(Key::P1Down);
            let is_up = input.is_key_press(Key::P1Up);
            let move_by = V2::new(
                if is_left {
                    -1.0
                } else if is_right {
                    1.0
                } else {
                    0.0
                },
                if is_up {
                    -1.0
                } else if is_down {
                    1.0
                } else {
                    0.0
                },
            );

            ball_physics.add_force(move_by * 10.0);
        }
    }
}

fn create_rectangle_actor(world: &mut World, center: V2, size: V2, color: Color, _name: Option<&str>) -> ActorId {
    let mut physics = Physics::new();
    physics.with_mass(1.0).with_drag(0.5);
    world.add_new_actor(
        Some(Transform::new(center, size.clone())),
        Some(Collider::new(
            vec![ColliderPart {
                offset: V2::zero(),
                extend: size.clone(),
                is_overlap: false,
            }],
            Some(0),
        )),
        Some(physics),
        None,
        Some(ColorMatrix::new(size.x as u8, size.y as u8, color)),
    )
}
