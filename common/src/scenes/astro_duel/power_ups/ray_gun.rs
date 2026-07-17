use libm::{cosf, roundf, sinf};
use crate::engine::{color::Color, color_matrix::ColorMatrix, v2::V2};

pub const RAY_GUN_LINE_WIDTH: i16 = 2;
const VISUAL_DURATION: f32 = 2.0;

pub struct RayGunBlast {
    pub start: V2,
    pub end: V2,
    pub owner_is_p1: bool,
    visual_timer: f32,
    pub collider_active: bool, // true only on the first frame
}

impl RayGunBlast {
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
        self.collider_active = false;
        self.visual_timer -= delta_time;
        self.visual_timer <= 0.0
    }

    /// Returns true if point is within RAY_GUN_LINE_WIDTH/2 of the ray segment.
    pub fn hits(&self, pos: V2) -> bool {
        if !self.collider_active {
            return false;
        }
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
