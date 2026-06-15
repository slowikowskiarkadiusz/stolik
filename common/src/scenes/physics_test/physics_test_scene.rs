extern crate alloc;
use std::mem::transmute;

use alloc::{boxed::Box, string::ToString, vec::Vec};
use rand::{Rng, SeedableRng, rngs::SmallRng};

use crate::{
    engine::{
        color::Color,
        color_matrix::ColorMatrix,
        components::{
            camera::Camera, collider::{Collider, ColliderPart, ColliderType, CollisionResult}, physics::Physics, transform::Transform, world::World
        },
        engine::{ActorId, SCREEN_SIZE, SCREEN_SIZEF32, open_scene},
        hash_map::HashMap,
        input::{input::Input, key::Key},
        scene::Scene,
        v2::V2,
    },
    scenes::utils::print_victory_text,
};
use alloc::vec;

static ORIGINAL_BALL_SPEED: f32 = 10.0;
static SIZE_FACTOR: f32 = SCREEN_SIZEF32 / 32.0;
static BALL_SIZE: f32 = 2.0 * SIZE_FACTOR;

pub struct PhysicsTestScene {
    ball: ActorId,
    foot_sensor: ActorId,
    collides: bool,
    resize_timer: f32,

    walls: Vec<ActorId>,
    floors: Vec<ActorId>,
    pipes: Vec<ActorId>,
    platforms: Vec<ActorId>,
}

impl Scene for PhysicsTestScene {
    fn init(&mut self, world: &mut World) {
        let map_width = 128.0_f32;
        let map_height = 64.0_f32;

        // podłoga
        self.floors.push(create_rectangle_actor(
            world,
            V2::new(map_width / 2.0, map_height - SIZE_FACTOR / 2.0),
            V2::new(map_width, SIZE_FACTOR),
            // Color::white(),
            None,
        ));

        // rury
        let pipe_positions = [16.0, 48.0, 80.0, 112.0];
        let pipe_height = 12.0;
        let pipe_width = 6.0;
        for px in pipe_positions {
            self.pipes.push(create_rectangle_actor(
                world,
                V2::new(px, map_height - pipe_height / 2.0 - SIZE_FACTOR),
                V2::new(pipe_width, pipe_height),
                // Color::green(),
                None,
            ));
        }

        // platformy nad rurami (ten sam Y)
        let platform_y = map_height - pipe_height - SIZE_FACTOR - 8.0;
        for px in pipe_positions {
            self.pipes.push(create_rectangle_actor(
                world,
                V2::new(px - 10.0, platform_y),
                V2::new(14.0, SIZE_FACTOR),
                // Color::new(139, 69, 19, 255),
                None,
            ));
        }

        // ściany
        let wall_thickness = SIZE_FACTOR;
        self.walls.push(create_rectangle_actor(
            world,
            V2::new(map_width / 2.0, wall_thickness / 2.0),
            V2::new(map_width, wall_thickness),
            // Color::white(),
            None,
        ));
        self.walls.push(create_rectangle_actor(
            world,
            V2::new(wall_thickness / 2.0, map_height / 2.0),
            V2::new(wall_thickness, map_height),
            // Color::none(),
            None,
        ));
        self.walls.push(create_rectangle_actor(
            world,
            V2::new(map_width - wall_thickness / 2.0, map_height / 2.0),
            V2::new(wall_thickness, map_height),
            // Color::none(),
            None,
        ));
        self.walls.push(create_rectangle_actor(
            world,
            V2::new(map_width / 2.0, map_height - wall_thickness / 2.0),
            V2::new(map_width, wall_thickness),
            // Color::white(),
            None,
        ));

        // ball
        self.ball = create_rectangle_actor(
            world,
            V2::new(8.0, map_height * 0.5),
            V2::one() * BALL_SIZE,
            // Color::white(),
            Some("ball"),
        );
        world.get_mut_physics(&self.ball).unwrap().with_can_move(true);

        self.foot_sensor = world.add_new_actor(
            Some(Transform::new(
                V2::new(8.0, map_height * 0.5 + BALL_SIZE / 2.0 + 0.5),
                V2::new(BALL_SIZE - 1.0, 1.0),
            )),
            Some(Collider::new(
                vec![ColliderPart {
                    offset: V2::zero(),
                    extend: V2::new(BALL_SIZE - 1.0, 1.0),
                    is_overlap: true,
                }],
                Some(0),
            )),
            None,
            None,
        );
    }

    fn tick(&mut self, input: &Box<dyn Input>, world: &mut World, delta_time: f32) {
        self.handle_input(input, world, delta_time);

        let ball_center = world.get_transform(&self.ball).unwrap().center;

        if let Some(t) = world.get_mut_transform(&self.foot_sensor) {
            t.center = V2::new(ball_center.x, ball_center.y + BALL_SIZE / 2.0 + 0.5);
        }
        let mut camera = world.get_mut_camera();
        camera.set_x(ball_center.x);
    }

    fn render(&mut self, camera: &Camera, world: &mut World, delta_time: f32) -> ColorMatrix {
        let mut result = ColorMatrix::new(camera.get_viewport_size().x as u8, camera.get_viewport_size().y as u8, Color::none());

        for actor_id in &self.walls {
            if let Some(transform) = world.get_transform(&actor_id) {
                result.write(
                    &ColorMatrix::new(transform.size.x as u8, transform.size.y as u8, Color::white()),
                    &transform.center,
                    None,
                    None,
                    None,
                    Some(camera),
                );
            }
        }

        for actor_id in &self.platforms {
            if let Some(transform) = world.get_transform(&actor_id) {
                result.write(
                    &ColorMatrix::new(transform.size.x as u8, transform.size.y as u8, Color::brown()),
                    &transform.center,
                    None,
                    None,
                    None,
                    Some(camera),
                );
            }
        }

        for actor_id in &self.pipes {
            if let Some(transform) = world.get_transform(&actor_id) {
                result.write(
                    &ColorMatrix::new(transform.size.x as u8, transform.size.y as u8, Color::green()),
                    &transform.center,
                    None,
                    None,
                    None,
                    Some(camera),
                );
            }
        }

        for actor_id in &self.pipes {
            if let Some(transform) = world.get_transform(&actor_id) {
                result.write(
                    &ColorMatrix::new(transform.size.x as u8, transform.size.y as u8, Color::green()),
                    &transform.center,
                    None,
                    None,
                    None,
                    Some(camera),
                );
            }
        }

        for actor_id in &self.floors {
            if let Some(transform) = world.get_transform(&actor_id) {
                result.write(
                    &ColorMatrix::new(transform.size.x as u8, transform.size.y as u8, Color::brown()),
                    &transform.center,
                    None,
                    None,
                    None,
                    Some(camera),
                );
            }
        }

        if let Some(ball_transform) = world.get_transform(&self.ball) {
            result.write(
                &ColorMatrix::new(ball_transform.size.x as u8, ball_transform.size.y as u8, Color::green()),
                &ball_transform.center,
                None,
                None,
                None,
                Some(camera),
            );
        }

        result
    }

    fn on_overlaps(&mut self, overlaps: &HashMap<ActorId, Vec<ActorId>>, world: &mut World, _delta_time: f32) {
        self.collides = false;
        for col in overlaps {
            if col.0 == &self.foot_sensor {
                if col.1.iter().any(|c| c != &self.ball) {
                    self.collides = true;
                }
            }
        }
    }

    fn on_collisions(&mut self, collisions: &HashMap<u16, Vec<(u16, CollisionResult)>>, world: &mut World, delta_time: f32) {
        // self.collides = false;
        // for col in collisions {
        //     if col.0 == &self.ball && col.1[0].0 == self.wall_bottom {
        //         self.collides = true;
        //     }
        // }
    }
}

impl PhysicsTestScene {
    pub fn new() -> Self {
        Self {
            ball: 0,
            foot_sensor: 0,
            collides: false,
            resize_timer: 0.0,

            walls: Vec::new(),
            floors: Vec::new(),
            pipes: Vec::new(),
            platforms: Vec::new(),
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
                if is_up && self.collides {
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

fn create_rectangle_actor(world: &mut World, center: V2, size: V2, _name: Option<&str>) -> ActorId {
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
    )
}
