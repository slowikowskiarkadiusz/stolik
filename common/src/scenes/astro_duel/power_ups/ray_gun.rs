extern crate alloc;
use alloc::vec::Vec;

use libm::{cosf, roundf, sinf};
use crate::engine::{color::Color, color_matrix::ColorMatrix, v2::V2};
use crate::scenes::astro_duel::{ship::Ship, scoring::apply_damage};

pub const RAY_GUN_LINE_WIDTH: i16 = 2;
const VISUAL_DURATION: f32 = 2.0;

pub struct RayGunBeam {
    pub start: V2,
    pub end: V2,
    pub owner_is_p1: bool,
    visual_timer: f32,
    pub collider_active: bool, // true only on the first frame
}

impl RayGunBeam {
    pub fn new(ship_center: V2, rotation_deg: f32, owner_is_p1: bool) -> Self {
        let rad = rotation_deg.to_radians();
        let dir_x = sinf(rad);
        let dir_y = -cosf(rad);

        // Extend to edge of 64x64 screen
        let t_max = {
            let mut t = f32::MAX;
            if dir_x > 0.0 { t = t.min((63.0 - ship_center.x) / dir_x); }
            if dir_x < 0.0 { t = t.min((0.0  - ship_center.x) / dir_x); }
            if dir_y > 0.0 { t = t.min((63.0 - ship_center.y) / dir_y); }
            if dir_y < 0.0 { t = t.min((0.0  - ship_center.y) / dir_y); }
            t
        };

        let end = V2::new(ship_center.x + dir_x * t_max, ship_center.y + dir_y * t_max);

        Self {
            start: ship_center,
            end,
            owner_is_p1,
            visual_timer: VISUAL_DURATION,
            collider_active: true,
        }
    }

    /// Returns true when blast should be removed.
    pub fn tick(&mut self, delta_time: f32) -> bool {
        self.visual_timer -= delta_time;
        self.visual_timer <= 0.0
    }

    /// Returns true if point is within RAY_GUN_LINE_WIDTH/2 of the ray segment.
    pub fn hits(&self, pos: V2) -> bool {
        let dx = self.end.x - self.start.x;
        let dy = self.end.y - self.start.y;
        let len_sq = dx * dx + dy * dy;
        if len_sq == 0.0 { return false; }
        let t = ((pos.x - self.start.x) * dx + (pos.y - self.start.y) * dy) / len_sq;
        let t = t.max(0.0).min(1.0);
        let proj_x = self.start.x + t * dx;
        let proj_y = self.start.y + t * dy;
        let dist_sq = (pos.x - proj_x) * (pos.x - proj_x) + (pos.y - proj_y) * (pos.y - proj_y);
        dist_sq <= (RAY_GUN_LINE_WIDTH as f32 / 2.0) * (RAY_GUN_LINE_WIDTH as f32 / 2.0)
    }

    /// Returns true if the segment start→end crosses any edge of the polygon defined by vertices.
    pub fn hits_polygon(&self, vertices: &[V2]) -> bool {
        let n = vertices.len();
        if n < 2 { return false; }
        for i in 0..n {
            let a = vertices[i];
            let b = vertices[(i + 1) % n];
            if seg_cross(self.start, self.end, a, b) {
                return true;
            }
        }
        false
    }

    pub fn apply(
        blasts: &mut Vec<RayGunBeam>,
        p1_center: Option<V2>,
        p2_center: Option<V2>,
        ship_p1: &mut Option<Ship>,
        ship_p2: &mut Option<Ship>,
        score: &mut [u8; 2],
        winner: &mut Option<u8>,
        game_over_timer: &mut f32,
        death_timer: &mut Option<(f32, usize)>,
    ) {
        for blast in blasts.iter_mut() {
            if !blast.collider_active { continue; }
            if !blast.owner_is_p1 && p1_center.map(|c| blast.hits(c)).unwrap_or(false) {
                apply_damage(ship_p1, 2, score, winner, game_over_timer, death_timer, 1);
            }
            if blast.owner_is_p1 && p2_center.map(|c| blast.hits(c)).unwrap_or(false) {
                apply_damage(ship_p2, 2, score, winner, game_over_timer, death_timer, 0);
            }
            blast.collider_active = false;
        }
    }

    pub fn render(&self, result: &mut ColorMatrix) {
        let blue = Color::new(0, 80, 255, 200);
        // Bresenham line, drawn with width RAY_GUN_LINE_WIDTH
        let x0 = roundf(self.start.x) as i16;
        let y0 = roundf(self.start.y) as i16;
        let x1 = roundf(self.end.x) as i16;
        let y1 = roundf(self.end.y) as i16;
        bresenham_thick(result, x0, y0, x1, y1, RAY_GUN_LINE_WIDTH, blue);
    }
}

/// Signed area of triangle (o, a, b) — positive if CCW.
fn cross2d(o: V2, a: V2, b: V2) -> f32 {
    (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x)
}

/// True if segment p1→p2 properly crosses segment p3→p4.
fn seg_cross(p1: V2, p2: V2, p3: V2, p4: V2) -> bool {
    let d1 = cross2d(p3, p4, p1);
    let d2 = cross2d(p3, p4, p2);
    let d3 = cross2d(p1, p2, p3);
    let d4 = cross2d(p1, p2, p4);
    (d1 * d2 < 0.0) && (d3 * d4 < 0.0)
}

fn bresenham_thick(result: &mut ColorMatrix, x0: i16, y0: i16, x1: i16, y1: i16, width: i16, color: Color) {
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx: i16 = if x0 < x1 { 1 } else { -1 };
    let sy: i16 = if y0 < y1 { 1 } else { -1 };
    let mut err = dx - dy;
    let (mut x, mut y) = (x0, y0);
    loop {
        for ow in 0..width {
            let (px, py) = if dx >= dy { (x, y + ow) } else { (x + ow, y) };
            if px >= 0 && py >= 0 && px < 64 && py < 64 {
                result.set(px as u8, py as u8, color);
            }
        }
        if x == x1 && y == y1 { break; }
        let e2 = 2 * err;
        if e2 > -dy { err -= dy; x += sx; }
        if e2 <  dx { err += dx; y += sy; }
    }
}
