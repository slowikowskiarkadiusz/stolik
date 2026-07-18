extern crate alloc;
use alloc::vec::Vec;

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
    input::{gesture::{Gesture, State}, input::Input, key::Key},
    v2::V2,
};

use super::astro_obstacle::AstroObstacleMap;
use super::boost_particles::BoostParticles;
use super::bullet::wrap_center;
use super::power_ups::{shield, reflector};
use super::power_ups::mine::PlacedMine;
use super::power_ups::ray_gun::RayGunBeam;
use super::power_ups::reflector::REFLECTOR_DURATION;
use super::power_ups::power_up::{PowerUpType, PowerUpKind, power_up_kind};
use crate::scenes::utils::{P1_COLOR, P2_COLOR};

const SHIP_SIZE: f32 = 4.0;
const THRUST_IMPULSE: f32 = 0.5;
const SHIP_DRAG: f32 = 0.5;
const BULLET_SPEED: f32 = 50.0;
const SHOOT_COOLDOWN: f32 = 0.15;
const MAX_AMMO: u8 = 4;
const AMMO_REGEN_TIME: f32 = 2.5;
const DASH_REGEN_TIME: f32 = 1.0;
const DASH_MULTIPIER: f32 = 30.0;
pub const MAX_LIVES: u8 = 2;
const BLINK_VISIBLE_TIME: f32 = 0.5;
const BLINK_INVISIBLE_TIME: f32 = 0.1;

pub struct BulletSpawn {
    pub pos: V2,
    pub velocity: V2,
    pub owner_is_p1: bool,
}

pub enum ShipAction {
    SpawnBullet(BulletSpawn),
    PlaceMine(PlacedMine),
    FireRayGun(RayGunBeam),
}

pub struct Ship {
    pub actor_id: ActorId,
    pub is_p1: bool,
    pub rotation: i32,
    pub lives: u8,
    /// Passive power-up currently active (Shield or Reflector).
    pub active_power_up: Option<PowerUpType>,
    pub reflector_timer: f32,
    /// Active power-up held in reserve (Mine or RayGun), triggered by hold-fire.
    pub stored_power_up: Option<PowerUpType>,
    pub boost_particles: Vec<BoostParticles>,
    shoot_cooldown: f32,
    ammo: u8,
    ammo_timer: f32,
    dash_timer: f32,
    blink_timer: f32,
    is_visible: bool,
    last_thrust: V2,
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
            lives: MAX_LIVES,
            active_power_up: None,
            reflector_timer: 0.0,
            stored_power_up: None,
            boost_particles: Vec::new(),
            shoot_cooldown: 0.0,
            ammo: MAX_AMMO,
            ammo_timer: AMMO_REGEN_TIME,
            dash_timer: 0.0,
            blink_timer: 0.0,
            is_visible: true,
            last_thrust: if is_p1 { V2::up() } else { V2::down() },
        }
    }

    pub fn tick(
        &mut self,
        input: &dyn Input,
        world: &mut World,
        _obstacle: &AstroObstacleMap,
        delta_time: f32,
    ) -> Vec<ShipAction> {
        let (thrust_dir, new_rotation, dashed) =
            get_thrust_and_rotation(self.dash_timer <= 0.0, input, self.is_p1, &mut self.last_thrust);

        if dashed {
            self.dash_timer = DASH_REGEN_TIME;
            let facing = rotation_to_dir(self.rotation);
            let tail = world.get_transform(&self.actor_id)
                .map(|t| t.center - facing * (SHIP_SIZE / 2.0))
                .unwrap_or(V2::zero());
            self.boost_particles.push(BoostParticles::new(tail, facing * -1.0, world));
        }

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

        if self.lives == 0 {
            self.is_visible = false;
        } else if self.lives < MAX_LIVES {
            self.blink_timer -= delta_time;
            if self.blink_timer <= 0.0 {
                self.is_visible = !self.is_visible;
                self.blink_timer = if self.is_visible { BLINK_VISIBLE_TIME } else { BLINK_INVISIBLE_TIME };
            }
        } else {
            self.is_visible = true;
        }

        self.boost_particles.retain_mut(|p| !p.tick(delta_time, world));

        if self.lives == 0 {
            return Vec::new();
        }

        if self.dash_timer > 0.0 {
            self.dash_timer -= delta_time;
        }

        if self.ammo < MAX_AMMO {
            self.ammo_timer -= delta_time;
            if self.ammo_timer <= 0.0 {
                self.ammo += 1;
                self.ammo_timer = AMMO_REGEN_TIME;
            }
        }

        // Tick reflector timer
        if self.active_power_up == Some(PowerUpType::Reflector) {
            self.reflector_timer -= delta_time;
            if self.reflector_timer <= 0.0 {
                self.active_power_up = None;
            }
        }

        let fire_key = if self.is_p1 { Key::P1Blue } else { Key::P2Blue };
        let fire_down = input.is_key_down(fire_key);
        let fire_activate = input.gestures().is(fire_key, State::Press, Gesture::Prolonged, None);

        let mut actions: Vec<ShipAction> = Vec::new();

        // Stored active power-ups (Mine / RayGun) triggered by double-tap
        if fire_activate {
            if let Some(stored) = self.stored_power_up.take() {
                let pos = world
                    .get_transform(&self.actor_id)
                    .map(|t| t.center)
                    .unwrap_or(V2::zero());
                match stored {
                    PowerUpType::Mine => {
                        actions.push(ShipAction::PlaceMine(PlacedMine::new(pos, self.is_p1)));
                    }
                    PowerUpType::RayGun => {
                        let rot = self.rotation;
                        actions.push(ShipAction::FireRayGun(RayGunBeam::new(pos, rot as f32, self.is_p1)));
                    }
                    _ => {
                        self.stored_power_up = Some(stored);
                    }
                }
            }
        }

        // Normal bullet shooting
        self.shoot_cooldown -= delta_time;
        if self.ammo > 0 && self.shoot_cooldown <= 0.0 && fire_down {
            self.ammo -= 1;
            if self.ammo < MAX_AMMO {
                self.ammo_timer = AMMO_REGEN_TIME;
            }
            self.shoot_cooldown = SHOOT_COOLDOWN;
            let facing = rotation_to_dir(self.rotation);
            let ship_pos = world
                .get_transform(&self.actor_id)
                .map(|t| t.center)
                .unwrap_or(V2::zero());
            let ship_vel = world
                .get_physics(&self.actor_id)
                .map(|p| *p.get_velocity())
                .unwrap_or(V2::zero());
            actions.push(ShipAction::SpawnBullet(BulletSpawn {
                pos: ship_pos + facing * (SHIP_SIZE / 2.0 + 2.0),
                velocity: ship_vel + facing * BULLET_SPEED,
                owner_is_p1: self.is_p1,
            }));
        }

        actions
    }

    /// Returns (died, shield_blocked).
    pub fn take_hit(&mut self) -> (bool, bool) {
        if self.lives == 0 {
            return (false, false);
        }
        if self.active_power_up == Some(PowerUpType::Shield) {
            self.active_power_up = None;
            return (false, true);
        }
        self.lives -= 1;
        (self.lives == 0, false)
    }

    /// Take damage that bypasses shield (ray gun does 2 hits).
    /// Returns true if ship died.
    pub fn take_damage(&mut self, amount: u8) -> bool {
        for _ in 0..amount {
            let (died, _) = self.take_hit();
            if died { return true; }
        }
        false
    }

    /// Returns true if the power-up was accepted.
    pub fn give_power_up(&mut self, pu_type: PowerUpType) -> bool {
        match power_up_kind(pu_type) {
            PowerUpKind::Passive => {
                if pu_type == PowerUpType::Reflector {
                    self.reflector_timer = REFLECTOR_DURATION;
                }
                self.active_power_up = Some(pu_type);
                true
            }
            PowerUpKind::Active => {
                if self.stored_power_up.is_some() {
                    false // stored slot full
                } else {
                    self.stored_power_up = Some(pu_type);
                    true
                }
            }
        }
    }

    pub fn render(&self, world: &World, _camera: &Camera, result: &mut ColorMatrix) {
        for boost_particle in &self.boost_particles {
            boost_particle.render(world, result);
        }

        if !self.is_visible {
            return;
        }
        if let Some(t) = world.get_transform(&self.actor_id) {
            result.write(
                &make_ship_sprite(self.ammo, self.is_p1),
                &t.center,
                Some(self.rotation as f32),
                None,
                None,
                None,
            );

            match self.active_power_up {
                Some(PowerUpType::Shield) => {
                    shield::draw_on_ship(t.center, self.rotation as f32, result);
                }
                Some(PowerUpType::Reflector) => {
                    reflector::draw_on_ship(t.center, self.rotation as f32, result);
                }
                _ => {}
            }
        }
    }
}

fn spawn_pos(is_p1: bool) -> V2 {
    if is_p1 { V2::new(16.0, 48.0) } else { V2::new(48.0, 16.0) }
}

fn create_actor(world: &mut World, pos: V2, rotation: f32) -> ActorId {
    let mut physics = Physics::new();
    physics
        .with_can_move(true)
        .with_mass(1.0)
        .with_drag(SHIP_DRAG)
        .with_can_rotate(false);

    let id = world.add_new_actor(
        Some(Transform::new(pos, V2::new(SHIP_SIZE, SHIP_SIZE))),
        Some(Collider::new(
            alloc::vec![ColliderPart::circle(V2::zero(), SHIP_SIZE / 2.0, false)],
            Some(0),
            false,
        )),
        Some(physics)

    );
    if let Some(t) = world.get_mut_transform(&id) {
        t.rotation = rotation;
    }
    id
}

fn make_ship_sprite(ammo: u8, is_p1: bool) -> ColorMatrix {
    let base = if is_p1 { P1_COLOR } else { P2_COLOR };
    let dim = Color::new(base.r, base.g, base.b, 120);

    let mut m = ColorMatrix::new(4, 4, Color::none());
    for row in 0u8..4 {
        let c = if (MAX_AMMO - row) <= ammo { base } else { dim };
        if row <= 1 {
            m.set(1, row, c);
            m.set(2, row, c);
        } else {
            for x in 0..4u8 {
                m.set(x, row, c);
            }
        }
    }
    m
}

fn get_thrust_and_rotation(
    can_dash: bool,
    input: &dyn Input,
    is_p1: bool,
    last_thrust: &mut V2,
) -> (V2, Option<i32>, bool) {
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

    let dashed = can_dash
        && (is_p1 && input.is_key_press(Key::P1Green)
            || (!is_p1 && input.is_key_press(Key::P2Green)));

    let mut thrust = V2::zero();
    if up    { thrust.y -= 1.0; }
    if down  { thrust.y += 1.0; }
    if left  { thrust.x -= 1.0; }
    if right { thrust.x += 1.0; }

    let mag = thrust.mag();
    if mag > 0.0 { thrust = thrust / mag; }

    if !(thrust.x == 0.0 && thrust.y == 0.0) {
        last_thrust.x = thrust.x;
        last_thrust.y = thrust.y;
    }

    if dashed {
        thrust += last_thrust.clone() * DASH_MULTIPIER;
    }

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

    (thrust, new_rotation, dashed)
}

fn rotation_to_dir(rotation: i32) -> V2 {
    V2::up().rotate(rotation as f32)
}
