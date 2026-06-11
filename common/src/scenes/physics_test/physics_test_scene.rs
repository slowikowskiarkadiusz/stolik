extern crate alloc;
use alloc::{boxed::Box, string::ToString, vec::Vec};
use rand::{Rng, SeedableRng, rngs::SmallRng};

use crate::{
    engine::{
        color::Color,
        color_matrix::ColorMatrix,
        components::{
            collider::{Collider, ColliderPart, ColliderType, CollisionResult},
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
use alloc::vec;

static ORIGINAL_BALL_SPEED: f32 = 10.0;

pub struct PhysicsTestScene {
    ball: ActorId,
    ball_2: Option<ActorId>,
    wall_top: ActorId,
    wall_bottom: ActorId,
    wall_left: ActorId,
    wall_right: ActorId,
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
        create_rectangle_actor(
            world,
            &(V2::one() * screen_size / 2.0) - &V2::new(0.0, -5.0),
            V2::one() * 2.0 * size_factor,
            Color::red(),
            Some("ball"),
        );

        create_rectangle_actor(
            world,
            &(V2::one() * screen_size / 2.0) - &V2::new(0.0, -15.0),
            V2::one() * 2.0 * size_factor,
            Color::blue(),
            Some("ball"),
        );

        let wall_thickness = 1.0 * size_factor;

        self.wall_top = create_rectangle_actor(
            world,
            V2::new(screen_size / 2.0, wall_thickness / 2.0),
            V2::new(screen_size, wall_thickness),
            Color::white(),
            None,
        );
        self.wall_bottom = create_rectangle_actor(
            world,
            V2::new(screen_size / 2.0, screen_size - wall_thickness / 2.0),
            V2::new(screen_size, wall_thickness),
            Color::white(),
            None,
        );
        self.wall_left = create_rectangle_actor(
            world,
            V2::new(wall_thickness / 2.0, screen_size / 2.0),
            V2::new(wall_thickness, screen_size),
            Color::white(),
            None,
        );
        self.wall_right = create_rectangle_actor(
            world,
            V2::new(screen_size - wall_thickness / 2.0, screen_size / 2.0),
            V2::new(wall_thickness, screen_size),
            Color::white(),
            None,
        );

        world.get_mut_physics(&self.wall_top).unwrap().with_is_fixed(true);
        world.get_mut_physics(&self.wall_bottom).unwrap().with_is_fixed(true);
        world.get_mut_physics(&self.wall_left).unwrap().with_is_fixed(true);
        world.get_mut_physics(&self.wall_right).unwrap().with_is_fixed(true);
    }

    fn tick(&mut self, input: &Box<dyn Input>, world: &mut World, delta_time: f32) {
        self.handle_input(input, world, delta_time);
    }

    fn on_overlaps(&mut self, overlaps: &HashMap<ActorId, Vec<ActorId>>, world: &mut World, _delta_time: f32) {}

    fn on_collisions(&mut self, collisions: &HashMap<u16, Vec<(u16, CollisionResult)>>, world: &mut World, delta_time: f32) {
        let mut collides = false;
        for col in collisions {
            if col.0 == &self.ball && col.1[0].0 == self.wall_bottom {
                collides = true;
            }
        }

        // println!("{}", if collides { "collides" } else { "doesnt" });
    }
}

impl PhysicsTestScene {
    pub fn new() -> Self {
        Self {
            ball: 0,
            ball_2: None,
            wall_top: 0,
            wall_bottom: 0,
            wall_left: 0,
            wall_right: 0,
        }
    }

    fn handle_input(&mut self, input: &Box<dyn Input + 'static>, world: &mut World, delta_time: f32) {
        if let Some(ball_physics) = world.get_mut_physics(&self.ball) {
            let is_left = input.is_key_press(Key::P1Left);
            let is_right = input.is_key_press(Key::P1Right);
            let is_down = input.is_key_press(Key::P1Down);
            let is_up = input.is_key_down(Key::P1Up);
            let move_by = V2::new(
                if is_left {
                    -20.0
                } else if is_right {
                    20.0
                } else {
                    0.0
                },
                if is_up {
                    -1000.0
                } else if is_down {
                    1.0
                } else {
                    0.0
                },
            );

            ball_physics.add_force(move_by);
        }
    }
}

fn create_rectangle_actor(world: &mut World, center: V2, size: V2, color: Color, _name: Option<&str>) -> ActorId {
    let mut physics = Physics::new();
    physics.with_mass(1.0).with_drag(0.8);
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
