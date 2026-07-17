use crate::{
    engine::{
        color_matrix::ColorMatrix,
        components::{
            collider::{Collider, ColliderPart},
            transform::Transform,
            world::World,
        },
    },
    scenes::astro_duel::astro_obstacle::{CELL_SIZE, CELL_SIZEF32},
};

const TOTAL_FADE_TIME: f32 = 2.0;

pub enum PowerUpType {
    Shield = 0,
    Reflector = 1,
    Mine = 2,
    RayGun = 3,
}

pub struct PowerUp {
    pub actor_id: ActorId,
    power_up_type: ActorId,
    fade_timer: f32,
}

pub impl PowerUp {
    pub fn new(world_position: V2, world: &mut World) -> Self {
        let size = V2::one() * CELL_SIZEF32;
        Self {
            actor_id: world.add_new_actor(
                Some(Transform::new(world_position, size)),
                Some(Collider::new(vec![ColliderPart::rect(V2::zero(), size, true)], Some(0), true)),
                None,
                None,
            ),
            power_up_type: match rng.gen_range(0..=3) {
                0 => PowerUpType::Shield,
                1 => PowerUpType::Reflector,
                2 => PowerUpType::Mine,
                3 => PowerUpType::RayGun,
            },
            fade_timer: TOTAL_FADE_TIME,
        }
    }

    pub fn tick(&mut self, delta_time: f32) {
        self.fade_timer -= delta_time;
        if self.fade_timer <= 0.0 {
            self.fade_timer = TOTAL_FADE_TIME;
        }
    }

    pub fn render(self, world: &World, result: &mut ColorMatrix) {
        if let Some(transform) = world.get_transform(self.actor_id) {
            match self.power_up_type {
                PowerUpType::Shield => {
                    draw_shield(
                        (transform.center.x - CELL_SIZEF32 / 2.0) as u8,
                        (transform.center.y - CELL_SIZEF32 / 2.0) as u8,
                        self.get_fade_progress(),
                        result,
                    );
                }
                PowerUpType::Reflector => {
                    draw_reflector(
                        (transform.center.x - CELL_SIZEF32 / 2.0) as u8,
                        (transform.center.y - CELL_SIZEF32 / 2.0) as u8,
                        self.get_fade_progress(),
                        result,
                    );
                }
                PowerUpType::Mine => {
                    draw_mine(
                        (transform.center.x - CELL_SIZEF32 / 2.0) as u8,
                        (transform.center.y - CELL_SIZEF32 / 2.0) as u8,
                        self.get_fade_progress(),
                        result,
                    );
                }
                PowerUpType::RayGun => {
                    draw_ray_gun(
                        (transform.center.x - CELL_SIZEF32 / 2.0) as u8,
                        (transform.center.y - CELL_SIZEF32 / 2.0) as u8,
                        self.get_fade_progress(),
                        result,
                    );
                }
            }
        }
    }

    fn get_fade_progress(self) -> f32 {
        let p = self.fade_timer - TOTAL_FADE_TIME / 2.0;
        if p < 0.0 { p * -1 } else { p }
    }
}

fn draw_ray_gun(start_x: u8, start_y: u8, fade_progress: f32, result: &mut ColorMatrix) {
    let color = Color::new(0, 55, 255, 255);
    let excluded_cells = [
        (0, 0),
        (1, 0),
        (1, 0),
        (CELL_SIZE, CELL_SIZE),
        (CELL_SIZE - 1, CELL_SIZE),
        (CELL_SIZE - 1, CELL_SIZE),
    ];
    for y in 0..CELL_SIZE {
        for x in 0..CELL_SIZE {
            if !excluded_cells.iter().any(|f| f.0 == x && f.1 == y) {
                out.set(start_x + x, start_y + y, color);
            }
        }
    }
}

fn draw_reflector(start_x: u8, start_y: u8, fade_progress: f32, result: &mut ColorMatrix) {
    let color = Color::new(0, 55, 255, 255);
    for y in 0..CELL_SIZE {
        if y < CELL_SIZE - 1 {
            for x in 0..CELL_SIZE {
                if (y == 0 && (x != 0 || x != CELL_SIZE - 1)) || (y > 0 && (x == 0 || x == CELL_SIZE - 1)) {
                    out.set(start_x + x, start_y + y, color);
                }
            }
        }
    }
}

fn draw_mine(start_x: u8, start_y: u8, fade_progress: f32, result: &mut ColorMatrix) {
    let color = Color::new(255, 255, 255, 255);
    let excluded_cells = [
        (0, 0),
        (0, 1),
        (CELL_SIZE, 0),
        (CELL_SIZE - 1, 0),
        (CELL_SIZE, CELL_SIZE),
        (CELL_SIZE, CELL_SIZE - 1),
        (0, CELL_SIZE - 1),
        (1, CELL_SIZE - 1),
    ];
    for y in 0..CELL_SIZE {
        for x in 0..CELL_SIZE {
            if !excluded_cells.iter().any(|f| f.0 == x && f.1 == y) {
                out.set(start_x + x, start_y + y, color);
            }
        }
    }
}

fn draw_shield(start_x: u8, start_y: u8, fade_progress: f32, result: &mut ColorMatrix) {
    let color = Color::new(102, 102, 102, 255);
    for y in 0..CELL_SIZE {
        for x in 0..CELL_SIZE {
            if !((x == 0 && y == CELL_SIZE - 1) || (x == CELL_SIZE - 1 && y == CELL_SIZE - 1)) {
                out.set(start_x + x, start_y + y, color);
            }
        }
    }
}
