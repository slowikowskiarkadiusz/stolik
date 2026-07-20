extern crate alloc;
use alloc::vec::Vec;

use crate::engine::{color::Color, color_matrix::ColorMatrix, v2::V2};
use crate::scenes::astro_duel::{astro_obstacle::CELL_SIZE, ship::Ship, scoring::apply_damage};
use libm::sqrtf;

const FUSE_TIME: f32 = 3.0;
const TRIGGER_COUNTDOWN: f32 = 1.0;
const EXPLOSION_SHOW_TIME: f32 = 1.0;
pub const BLAST_RADIUS: f32 = 15.0;

pub struct PlacedMine {
    pub pos: V2,
    pub owner_is_p1: bool,
    fuse_timer: f32,
    pub is_active: bool,      // true after fuse — starts detecting ships
    pub triggered: bool,      // true when ship entered radius; center turns red
    trigger_timer: f32,       // countdown to explosion after trigger
    pub just_detonated: bool, // true for exactly one frame — scene applies damage here
    pub detonated: bool,      // true after explosion (shows blast circle)
    explosion_timer: f32,
}

impl PlacedMine {
    pub fn new(pos: V2, owner_is_p1: bool) -> Self {
        Self {
            pos,
            owner_is_p1,
            fuse_timer: FUSE_TIME,
            is_active: false,
            triggered: false,
            trigger_timer: 0.0,
            just_detonated: false,
            detonated: false,
            explosion_timer: EXPLOSION_SHOW_TIME,
        }
    }

    pub fn trigger(&mut self) {
        if self.is_active && !self.triggered {
            self.triggered = true;
            self.trigger_timer = TRIGGER_COUNTDOWN;
        }
    }

    pub fn tick(&mut self, delta_time: f32) -> bool {
        self.just_detonated = false;

        if self.detonated {
            self.explosion_timer -= delta_time;
            return self.explosion_timer <= 0.0;
        }
        if self.triggered {
            self.trigger_timer -= delta_time;
            if self.trigger_timer <= 0.0 {
                self.detonated = true;
                self.just_detonated = true;
            }
            return false;
        }
        if !self.is_active {
            self.fuse_timer -= delta_time;
            if self.fuse_timer <= 0.0 {
                self.is_active = true;
            }
        }
        false
    }

    pub fn in_blast_radius(&self, pos: V2) -> bool {
        (pos - self.pos).mag() <= BLAST_RADIUS
    }

    pub fn apply(
        mines: &mut Vec<PlacedMine>,
        p1_center: Option<V2>,
        p2_center: Option<V2>,
        ship_p1: &mut Option<Ship>,
        ship_p2: &mut Option<Ship>,
        score: &mut [u8; 2],
        winner: &mut Option<u8>,
        game_over_timer: &mut f32,
        death_timer: &mut Option<(f32, usize)>,
    ) {
        for mine in mines.iter_mut() {
            if mine.just_detonated {
                if p1_center.map(|c| mine.in_blast_radius(c)).unwrap_or(false) {
                    apply_damage(ship_p1, 3, score, winner, game_over_timer, death_timer, 1);
                }
                if p2_center.map(|c| mine.in_blast_radius(c)).unwrap_or(false) {
                    apply_damage(ship_p2, 3, score, winner, game_over_timer, death_timer, 0);
                }
                continue;
            }
            if mine.detonated { continue; }
            if mine.is_active && !mine.triggered {
                let p1_near = p1_center.map(|c| mine.in_blast_radius(c)).unwrap_or(false);
                let p2_near = p2_center.map(|c| mine.in_blast_radius(c)).unwrap_or(false);
                if p1_near || p2_near {
                    mine.trigger();
                }
            }
        }
    }

    pub fn render(&self, result: &mut ColorMatrix) {
        if self.detonated {
            draw_explosion(self.pos, result);
        } else {
            draw_placed(self.pos, self.triggered, result);
        }
    }
}

fn draw_placed(pos: V2, red_center: bool, result: &mut ColorMatrix) {
    let sx = (pos.x as i16) - (CELL_SIZE as i16 / 2);
    let sy = (pos.y as i16) - (CELL_SIZE as i16 / 2);
    let white = Color::new(255, 255, 255, 255);
    let center_color = if red_center { Color::new(255, 0, 0, 255) } else { white };

    let is_center = |x: u8, y: u8| x >= 1 && x <= 2 && y >= 1 && y <= 2;

    for x in 0..CELL_SIZE - 1 {
        set_safe(result, sx + x as i16, sy + 1, if is_center(x, 1) { center_color } else { white });
    }
    for x in 1..CELL_SIZE {
        set_safe(result, sx + x as i16, sy + 2, if is_center(x, 2) { center_color } else { white });
    }
    for y in 1..CELL_SIZE {
        set_safe(result, sx + 1, sy + y as i16, if is_center(1, y) { center_color } else { white });
    }
    for y in 0..CELL_SIZE - 1 {
        set_safe(result, sx + 2, sy + y as i16, if is_center(2, y) { center_color } else { white });
    }
}

fn draw_explosion(pos: V2, result: &mut ColorMatrix) {
    let cx = pos.x as i16;
    let cy = pos.y as i16;
    let r = BLAST_RADIUS;
    let ri = r as i16;

    for dy in -ri..=ri {
        for dx in -ri..=ri {
            let dist = sqrtf((dx * dx + dy * dy) as f32);
            if dist > r { continue; }
            // alpha: 0 at center, 255 at edge
            let t = dist / r;
            let alpha = (255.0 * t) as u8;
            set_safe(result, cx + dx, cy + dy, Color::new(255, 255, 255, alpha));
        }
    }
}

fn set_safe(result: &mut ColorMatrix, x: i16, y: i16, color: Color) {
    if x >= 0 && y >= 0 && x < 64 && y < 64 {
        result.set(x as u8, y as u8, color);
    }
}
