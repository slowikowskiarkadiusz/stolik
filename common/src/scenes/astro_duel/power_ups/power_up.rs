extern crate alloc;
use alloc::vec;

use rand::{Rng, rngs::SmallRng};

use crate::scenes::astro_duel::astro_obstacle::{CELL_SIZE, CELL_SIZEF32};
use crate::{
    engine::{
        color::Color,
        color_matrix::ColorMatrix,
        components::{
            collider::{Collider, ColliderPart},
            transform::Transform,
            world::World,
        },
        engine::ActorId,
        v2::V2,
    },
    scenes::utils::lerp_u8,
};

const TOTAL_FADE_TIME: f32 = 2.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerUpType {
    Shield,
    Reflector,
    Mine,
    RayGun,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerUpKind {
    Passive, // Shield, Reflector — activate automatically on pickup
    Active,  // Mine, RayGun — activated manually by holding fire
}

pub const fn power_up_kind(t: PowerUpType) -> PowerUpKind {
    match t {
        PowerUpType::Shield | PowerUpType::Reflector => PowerUpKind::Passive,
        PowerUpType::Mine   | PowerUpType::RayGun    => PowerUpKind::Active,
    }
}

pub struct PowerUp {
    pub actor_id: ActorId,
    power_up_type: PowerUpType,
    fade_timer: f32,
}

impl PowerUp {
    pub fn new(world_position: V2, rng: &mut SmallRng, world: &mut World) -> Self {
        let size = V2::one() * CELL_SIZEF32;
        let power_up_type: PowerUpType = match rng.gen_range(0..=3) {
            0 => PowerUpType::Shield,
            1 => PowerUpType::Reflector,
            2 => PowerUpType::Mine,
            3 => PowerUpType::RayGun,
            _ => PowerUpType::Shield,
        };
        Self {
            actor_id: world.add_new_actor(
                Some(Transform::new(world_position, size)),
                Some(Collider::new(vec![ColliderPart::rect(V2::zero(), size, true)], Some(0), true)),
                None,
                None,
            ),
            power_up_type,
            fade_timer: TOTAL_FADE_TIME,
        }
    }

    pub fn power_up_type(&self) -> PowerUpType {
        match self.power_up_type {
            PowerUpType::Shield    => PowerUpType::Shield,
            PowerUpType::Reflector => PowerUpType::Reflector,
            PowerUpType::Mine      => PowerUpType::Mine,
            PowerUpType::RayGun    => PowerUpType::RayGun,
        }
    }

    pub fn tick(&mut self, delta_time: f32) {
        self.fade_timer -= delta_time;
        if self.fade_timer <= 0.0 {
            self.fade_timer = TOTAL_FADE_TIME;
        }
    }

    pub fn render(&self, world: &World, result: &mut ColorMatrix) {
        if let Some(transform) = world.get_transform(&self.actor_id) {
            let x = (transform.center.x - CELL_SIZEF32 / 2.0) as u8;
            let y = (transform.center.y - CELL_SIZEF32 / 2.0) as u8;
            let fade = self.get_fade_progress();
            match self.power_up_type {
                PowerUpType::Shield => draw_shield(x, y, fade, result),
                PowerUpType::Reflector => draw_reflector(x, y, fade, result),
                PowerUpType::Mine => draw_mine(x, y, fade, result),
                PowerUpType::RayGun => draw_ray_gun(x, y, fade, result),
            }
        }
    }

    fn get_fade_progress(&self) -> f32 {
        let p = self.fade_timer - TOTAL_FADE_TIME / 2.0;
        if p < 0.0 { p * -1.0 } else { p }
    }
}

fn draw_ray_gun(start_x: u8, start_y: u8, fade_progress: f32, result: &mut ColorMatrix) {
    let color = Color::new(0, 55, 255, lerp_u8(255, 150, fade_progress));
    let excluded_cells = [
        (0, 0),
        (1, 0),
        (0, 1),
        (CELL_SIZE - 1, CELL_SIZE - 1),
        (CELL_SIZE - 2, CELL_SIZE - 1),
        (CELL_SIZE - 1, CELL_SIZE - 2),
    ];
    for y in 0..CELL_SIZE {
        for x in 0..CELL_SIZE {
            if !excluded_cells.iter().any(|f| f.0 == x && f.1 == y) {
                result.set(start_x + x, start_y + y, color);
            }
        }
    }
}

fn draw_reflector(start_x: u8, start_y: u8, fade_progress: f32, result: &mut ColorMatrix) {
    let color = Color::new(0, 55, 255, lerp_u8(255, 150, fade_progress));
    for y in 0..CELL_SIZE {
        if y < CELL_SIZE - 1 {
            for x in 0..CELL_SIZE {
                if (y == 0 && (x != 0 || x != CELL_SIZE - 1)) || (y > 0 && (x == 0 || x == CELL_SIZE - 1)) {
                    result.set(start_x + x, start_y + y, color);
                }
            }
        }
    }
}

fn draw_mine(start_x: u8, start_y: u8, fade_progress: f32, result: &mut ColorMatrix) {
    let color = Color::new(255, 255, 255, lerp_u8(255, 150, fade_progress));

    for x in 0..CELL_SIZE - 1 {
        result.set(start_x + x, start_y + 1, color);
    }

    for x in 1..CELL_SIZE {
        result.set(start_x + x, start_y + 2, color);
    }

    for y in 1..CELL_SIZE {
        result.set(start_x + 1, start_y + y, color);
    }

    for y in 0..CELL_SIZE - 1 {
        result.set(start_x + 2, start_y + y, color);
    }
}

fn draw_shield(start_x: u8, start_y: u8, fade_progress: f32, result: &mut ColorMatrix) {
    let color = Color::new(200, 200, 200, lerp_u8(255, 150, fade_progress));
    for y in 0..CELL_SIZE {
        for x in 0..CELL_SIZE {
            if !((x == 0 && y == CELL_SIZE - 1) || (x == CELL_SIZE - 1 && y == CELL_SIZE - 1)) {
                result.set(start_x + x, start_y + y, color);
            }
        }
    }
}
