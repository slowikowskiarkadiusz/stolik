extern crate alloc;

use alloc::{boxed::Box, string::ToString, vec::Vec};
use rand::{Rng, SeedableRng, rngs::SmallRng};

use crate::{
    engine::{
        actor::text::create_text_actor_at_center,
        color::Color,
        color_matrix::ColorMatrix,
        components::{
            camera::Camera,
            collider::{Collider, ColliderPart, ColliderType, CollisionResult},
            physics::Physics,
            transform::Transform,
            world::World,
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
static CELL_SIZE: f32 = 4.0;

static QBLOCKS: [V2; 6] = [
    V2::new(17.0, 6.0),
    V2::new(22.0, 6.0),
    V2::new(24.0, 6.0),
    V2::new(23.0, 10.0),
    V2::new(65.0, 7.0),
    V2::new(79.0, 6.0),
];
static BRICKS: [V2; 12] = [
    V2::new(21.0, 6.0),
    V2::new(23.0, 6.0),
    V2::new(25.0, 6.0),
    V2::new(78.0, 6.0),
    V2::new(80.0, 6.0),
    V2::new(82.0, 10.0),
    V2::new(83.0, 10.0),
    V2::new(84.0, 10.0),
    V2::new(85.0, 10.0),
    V2::new(86.0, 10.0),
    V2::new(87.0, 10.0),
    V2::new(88.0, 10.0),
];
static PIPES: [(f32, f32, f32); 4] = [(29.0, 2.0, 4.0), (39.0, 2.0, 5.0), (47.0, 2.0, 6.0), (58.0, 2.0, 6.0)];
static FLOOR: [(f32, f32); 5] = [(0.0, 30.0), (30.0, 70.0), (72.0, 87.0), (90.0, 154.0), (156.0, 200.0)];

pub struct MarioScene {
    plumber: ActorId,
    foot_sensor: ActorId,
    collides: bool,
    resize_timer: f32,

    walls: Vec<ActorId>,
    floors: Vec<ActorId>,
    qblocks: Vec<ActorId>,
    pipes: Vec<ActorId>,
    bricks: Vec<ActorId>,
}

fn at(x: f32, y: f32) -> V2 {
    V2::new(x * CELL_SIZE, SCREEN_SIZEF32 - (y * CELL_SIZE))
}

impl Scene for MarioScene {
    fn init(&mut self, world: &mut World) {
        let map_width = 128.0_f32;
        let map_height = 64.0_f32;

        for (from_x, to_x) in &FLOOR {
            self.floors.push(create_rectangle_at_origin(
                world,
                at(from_x.clone(), 0.0),
                at(to_x.clone(), 2.0),
                true,
            ));
        }

        for (x0, y0, y1) in &PIPES {
            self.pipes.push(create_rectangle_at_origin(
                world,
                at(x0.clone(), y0.clone()),
                at((x0 + 2.0), y1.clone()),
                true,
            ));
        }

        for point_at in &BRICKS {
            self.bricks.push(create_rectangle_at_origin(
                world,
                at(point_at.x, point_at.y),
                at((point_at.x + 1.0), (point_at.y + 1.0)),
                true,
            ));
        }

        for point_at in &QBLOCKS {
            self.qblocks.push(create_rectangle_at_origin(
                world,
                at(point_at.x, point_at.y),
                at((point_at.x + 1.0), (point_at.y + 1.0)),
                true,
            ));
        }

        let wall_thickness = SIZE_FACTOR;
        self.walls.push(create_rectangle_actor(
            world,
            V2::new(wall_thickness / 2.0, map_height / 2.0),
            V2::new(wall_thickness, map_height),
            true,
        ));
        // self.walls.push(create_rectangle_actor(
        //     world,
        //     V2::new(map_width / 2.0, map_height - wall_thickness / 2.0 - 6.0),
        //     V2::new(map_width, wall_thickness),
        //     true,
        // ));

        self.plumber = create_plumber(world, V2::new(8.0, map_height * 0.5), V2::one() * BALL_SIZE, true);
        world.get_mut_physics(&self.plumber).unwrap().with_can_move(true);

        self.foot_sensor = world.add_new_actor(
            Some(Transform::new(
                V2::new(8.0, map_height * 0.5 + BALL_SIZE / 2.0 + 0.5),
                V2::new(BALL_SIZE - 1.0, 1.0),
            )),
            Some(Collider::new(
                vec![ColliderPart::rect(V2::zero(), V2::new(BALL_SIZE - 1.0, 1.0), true)],
                Some(0),
            )),
            None,
            None,
        );
    }

    fn tick(&mut self, input: &Box<dyn Input>, world: &mut World, delta_time: f32) {
        self.handle_input(input, world, delta_time);

        let plumber_center = world.get_transform(&self.plumber).unwrap().center;

        if let Some(t) = world.get_mut_transform(&self.foot_sensor) {
            t.center = V2::new(plumber_center.x, plumber_center.y + BALL_SIZE / 2.0 + 0.5);
        }
        let mut camera = world.get_mut_camera();
        camera.set_x(plumber_center.x);
    }

    fn render(&mut self, camera: &Camera, world: &mut World, delta_time: f32) -> ColorMatrix {
        let vsize = camera.get_viewport().get_size();
        let mut result = ColorMatrix::new(vsize.x as u8, vsize.y as u8, Color::none());

        let mut i = 0;

        render_static_objects(&self.walls, Color::white(), world, camera, &mut result, &mut i);
        render_static_objects(&self.bricks, Color::brown(), world, camera, &mut result, &mut i);
        render_static_objects(&self.pipes, Color::green(), world, camera, &mut result, &mut i);
        render_static_objects(&self.qblocks, Color::yellow(), world, camera, &mut result, &mut i);
        render_static_objects(&self.floors, Color::brown(), world, camera, &mut result, &mut i);

        if let Some(plumber_transform) = world.get_transform(&self.plumber) {
            i += 1;
            result.write(
                &ColorMatrix::new(plumber_transform.size.x as u8, plumber_transform.size.y as u8, Color::blue()),
                &plumber_transform.center,
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
                if col.1.iter().any(|c| c != &self.plumber) {
                    self.collides = true;
                }
            }
        }
    }

    fn on_collisions(&mut self, collisions: &HashMap<u16, Vec<(u16, CollisionResult)>>, world: &mut World, delta_time: f32) {}
}

impl MarioScene {
    pub fn new() -> Self {
        Self {
            plumber: 0,
            foot_sensor: 0,
            collides: false,
            resize_timer: 0.0,

            walls: Vec::new(),
            qblocks: Vec::new(),
            floors: Vec::new(),
            pipes: Vec::new(),
            bricks: Vec::new(),
        }
    }

    fn handle_input(&mut self, input: &Box<dyn Input + 'static>, world: &mut World, delta_time: f32) {
        if let Some(plumber_physics) = world.get_mut_physics(&self.plumber) {
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
                    -1400.0
                } else if is_down {
                    1.0
                } else {
                    0.0
                },
            );

            plumber_physics.add_force(move_by);
        }
    }
}

fn create_rectangle_actor(world: &mut World, center: V2, size: V2, collides: bool) -> ActorId {
    let mut physics = Physics::new();
    physics.with_mass(1.0).with_drag(0.8);
    world.add_new_actor(
        Some(Transform::new(center, size.clone())),
        if collides {
            Some(Collider::new(vec![ColliderPart::rect(V2::zero(), size.clone(), false)], Some(0)))
        } else {
            None
        },
        Some(physics),
        None,
    )
}

fn create_plumber(world: &mut World, center: V2, size: V2, collides: bool) -> ActorId {
    let mut physics = Physics::new();
    physics.with_mass(1.0).with_drag(0.8);
    world.add_new_actor(
        Some(Transform::new(center, size.clone())),
        if collides {
            Some(Collider::new(
                vec![
                    ColliderPart::rect(V2::new(0.0, -size.y / 2.0), V2::new(size.x, size.y / 2.0), false),
                    ColliderPart::circle(V2::zero(), size.y / 2.0, false),
                ],
                Some(0),
            ))
        } else {
            None
        },
        Some(physics),
        None,
    )
}

fn create_rectangle_at_origin(world: &mut World, from: V2, to: V2, collides: bool) -> ActorId {
    let center = V2::new((from.x + to.x) / 2.0, (from.y + to.y) / 2.0);
    let size = V2::new((to.x - from.x).abs(), (to.y - from.y).abs());

    let mut physics = Physics::new();
    physics.with_mass(1.0).with_drag(0.8);
    world.add_new_actor(
        Some(Transform::new(center, size.clone())),
        if collides {
            Some(Collider::new(vec![ColliderPart::rect(V2::zero(), size.clone(), false)], Some(0)))
        } else {
            None
        },
        Some(physics),
        None,
    )
}

fn render_static_objects(collection: &Vec<ActorId>, color: Color, world: &World, camera: &Camera, result: &mut ColorMatrix, i: &mut u32) {
    for actor_id in collection {
        if let Some(transform) = world.get_transform(&actor_id)
            && camera.can_see_actor(actor_id.clone(), world)
        {
            *i += 1;

            result.write(
                &ColorMatrix::new(transform.size.x as u8, transform.size.y as u8, color),
                &transform.center,
                None,
                None,
                None,
                Some(camera),
            );
        }
    }
}
