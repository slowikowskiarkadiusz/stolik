use crate::engine::{color::Color, color_matrix::ColorMatrix, v2::V2};

pub const HEART_X: u8 = 30;
pub const HEART_W: u8 = 4;
pub const HEART_MAX_HEALTH: u8 = 16;
const SCREEN_H: u8 = 64;

pub struct Heart {
    pub health: u8,
    pub alive: bool,
    pub is_bottom: bool,
    blink_timer: f32,
    is_visible: bool,
}

impl Heart {
    pub fn new(is_bottom: bool) -> Self {
        let (on, _) = blink_params(HEART_MAX_HEALTH);
        Self {
            health: HEART_MAX_HEALTH,
            alive: true,
            is_bottom,
            blink_timer: on,
            is_visible: true,
        }
    }

    pub fn take_hit(&mut self, damage: u8) {
        if !self.alive { return; }
        self.health = self.health.saturating_sub(damage);
        if self.health == 0 {
            self.alive = false;
            return;
        }
        self.is_visible = true;
        self.blink_timer = blink_params(self.health).0;
    }

    pub fn tick(&mut self, delta_time: f32) {
        if !self.alive { return; }
        self.blink_timer -= delta_time;
        if self.blink_timer <= 0.0 {
            self.is_visible = !self.is_visible;
            let (on, off) = blink_params(self.health);
            self.blink_timer = if self.is_visible { on } else { off };
        }
    }

    fn rows(&self) -> u8 {
        (self.health + HEART_W - 1) / HEART_W
    }

    fn y_range(&self) -> (u8, u8) {
        let r = self.rows();
        if self.is_bottom {
            (SCREEN_H - r, SCREEN_H - 1)
        } else {
            (0, r - 1)
        }
    }

    pub fn blocker_box(&self) -> Option<(V2, V2)> {
        if !self.alive { return None; }
        let (y0, y1) = self.y_range();
        Some((
            V2::new(HEART_X as f32, y0 as f32),
            V2::new((HEART_X + HEART_W) as f32, y1 as f32 + 1.0),
        ))
    }

    pub fn overlaps_point(&self, p: V2) -> bool {
        if !self.alive { return false; }
        let (y0, y1) = self.y_range();
        p.x >= HEART_X as f32 - 1.0
            && p.x < (HEART_X + HEART_W) as f32 + 1.0
            && p.y >= y0 as f32 - 1.0
            && p.y <= y1 as f32 + 1.0
    }

    pub fn draw(&self, out: &mut ColorMatrix) {
        if !self.alive || !self.is_visible { return; }
        let c = Color::new(255, 255, 255, 255);
        for i in 0..self.health {
            let row = i / HEART_W;
            let col = i % HEART_W;
            let x = HEART_X + col;
            let y = if self.is_bottom { SCREEN_H - 1 - row } else { row };
            out.set(x, y, c);
        }
    }
}

fn blink_params(health: u8) -> (f32, f32) {
    let band = (health.saturating_sub(1)) / 4;
    let on = match band {
        0 => 0.1,
        1 => 0.2,
        2 => 0.4,
        _ => 0.6,
    };
    (on, 0.2)
}
