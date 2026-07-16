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
    engine::ActorId,
    input::{input::Input, key::Key},
    v2::V2,
};

use super::astro_obstacle::AstroObstacleMap;
use super::bullet::wrap_center;

const SHIP_SIZE: f32 = 4.0;
const THRUST_IMPULSE: f32 = 0.5;
const SHIP_DRAG: f32 = 0.5;
const BULLET_SPEED: f32 = 50.0;
const SHOOT_COOLDOWN: f32 = 0.3;
const RESPAWN_TIME: f32 = 2.0;

pub struct BulletSpawn {
    pub pos: V2,
    pub velocity: V2,
    pub owner_is_p1: bool,
}

pub struct Ship {
    pub actor_id: ActorId,
    pub is_p1: bool,
    pub rotation: i32,
    pub alive: bool,
    pub lives: u8,
    respawn_timer: f32,
    shoot_cooldown: f32,
}

impl Ship {
    pub fn new(world: &mut World, is_p1: bool) -> Self {
        let pos = spawn_pos(is_p1);
        let rotation = if is_p1 { 0 } else { 180 };
        let id = create_actor(world, pos, rotation as f32);
        Self {
            actor_id: id,
            is_p1,
            rotation,
            alive: true,
            lives: 3,
            respawn_timer: 0.0,
            shoot_cooldown: 0.0,
        }
    }

    pub fn tick(&mut self, input: &dyn Input, world: &mut World, obstacle: &AstroObstacleMap, delta_time: f32) -> Option<BulletSpawn> {
        if !self.alive {
            if self.lives > 0 {
                self.respawn_timer -= delta_time;
                if self.respawn_timer <= 0.0 {
                    self.respawn(world);
                }
            }
            return None;
        }

        let (thrust_dir, new_rotation) = get_thrust_and_rotation(input, self.is_p1);

        if let Some(rot) = new_rotation {
            self.rotation = rot;
            if let Some(t) = world.get_mut_transform(&self.actor_id) {
                t.rotation = rot as f32;
            }
        }

        if thrust_dir.mag() > 0.0 {
            if let Some(p) = world.get_mut_physics(&self.actor_id) {
                p.add_impulse(thrust_dir * THRUST_IMPULSE);
            }
        }

        wrap_center(world, self.actor_id);

        self.shoot_cooldown -= delta_time;
        let shoot_key = if self.is_p1 { Key::P1Blue } else { Key::P2Blue };
        let bullet = if self.shoot_cooldown <= 0.0 && input.is_key_down(shoot_key) {
            self.shoot_cooldown = SHOOT_COOLDOWN;
            let facing = rotation_to_dir(self.rotation);
            let ship_pos = world.get_transform(&self.actor_id).map(|t| t.center).unwrap_or(V2::zero());
            let ship_vel = world.get_physics(&self.actor_id).map(|p| *p.get_velocity()).unwrap_or(V2::zero());
            Some(BulletSpawn {
                pos: ship_pos + facing * (SHIP_SIZE / 2.0 + 2.0),
                velocity: ship_vel + facing * BULLET_SPEED,
                owner_is_p1: self.is_p1,
            })
        } else {
            None
        };

        bullet
    }

    pub fn take_hit(&mut self, world: &mut World) {
        if !self.alive {
            return;
        }
        self.lives = self.lives.saturating_sub(1);
        self.alive = false;
        world.remove_actor(&self.actor_id);
        if self.lives > 0 {
            self.respawn_timer = RESPAWN_TIME;
        }
    }

    fn respawn(&mut self, world: &mut World) {
        let pos = spawn_pos(self.is_p1);
        self.rotation = if self.is_p1 { 0 } else { 180 };
        self.actor_id = create_actor(world, pos, self.rotation as f32);
        self.alive = true;
    }

    pub fn pos(&self, world: &World) -> Option<V2> {
        world.get_transform(&self.actor_id).map(|t| t.center)
    }

    pub fn render(&self, world: &World, _camera: &Camera, result: &mut ColorMatrix) {
        if !self.alive {
            return;
        }
        if let Some(t) = world.get_transform(&self.actor_id) {
            let color = if self.is_p1 {
                Color::new(255, 200, 80, 255)
            } else {
                Color::new(80, 180, 255, 255)
            };
            result.write(
                &make_ship_sprite(color),
                &t.center,
                Some(self.rotation as f32),
                None, None,
                None,
            );
        }
    }
}

fn spawn_pos(is_p1: bool) -> V2 {
    if is_p1 { V2::new(16.0, 48.0) } else { V2::new(48.0, 16.0) }
}

fn create_actor(world: &mut World, pos: V2, rotation: f32) -> ActorId {
    let mut physics = Physics::new();
    physics.with_can_move(true).with_mass(1.0).with_drag(SHIP_DRAG).with_can_rotate(false);

    let id = world.add_new_actor(
        Some(Transform::new(pos, V2::new(SHIP_SIZE, SHIP_SIZE))),
        Some(Collider::new(
            alloc::vec![ColliderPart::circle(V2::zero(), SHIP_SIZE / 2.0, false)],
            Some(0),
        )),
        Some(physics),
        None,
    );
    if let Some(t) = world.get_mut_transform(&id) {
        t.rotation = rotation;
    }
    id
}

fn make_ship_sprite(color: Color) -> ColorMatrix {
    let mut m = ColorMatrix::new(4, 4, Color::none());
    // gun (canonical: facing up)
    m.set(1, 0, color);
    m.set(2, 0, color);
    // gun base
    m.set(1, 1, color);
    m.set(2, 1, color);
    // body
    for x in 0..4u8 {
        m.set(x, 2, color);
        m.set(x, 3, color);
    }
    m
}

fn get_thrust_and_rotation(input: &dyn Input, is_p1: bool) -> (V2, Option<i32>) {
    // P2 has swapped up/down (sits on the opposite side of the table)
    let (up, down, left, right) = if is_p1 {
        (
            input.is_key_press(Key::P1Up),
            input.is_key_press(Key::P1Down),
            input.is_key_press(Key::P1Left),
            input.is_key_press(Key::P1Right),
        )
    } else {
        (
            input.is_key_press(Key::P2Down),
            input.is_key_press(Key::P2Up),
            input.is_key_press(Key::P2Left),
            input.is_key_press(Key::P2Right),
        )
    };

    let mut thrust = V2::zero();
    if up { thrust.y -= 1.0; }
    if down { thrust.y += 1.0; }
    if left { thrust.x -= 1.0; }
    if right { thrust.x += 1.0; }

    let mag = thrust.mag();
    if mag > 0.0 {
        thrust = thrust / mag;
    }

    // All 8 directions snap rotation in 45° steps
    let new_rotation = match (up, down, left, right) {
        (true,  false, false, false) => Some(0),
        (true,  false, false, true)  => Some(45),
        (false, false, false, true)  => Some(90),
        (false, true,  false, true)  => Some(135),
        (false, true,  false, false) => Some(180),
        (false, true,  true,  false) => Some(225),
        (false, false, true,  false) => Some(270),
        (true,  false, true,  false) => Some(315),
        _ => None,
    };

    (thrust, new_rotation)
}

fn rotation_to_dir(rotation: i32) -> V2 {
    V2::up().rotate(rotation as f32)
}
