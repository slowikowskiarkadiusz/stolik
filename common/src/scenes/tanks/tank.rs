use super::obstacle::{ObstacleMap, ObstacleType};
use crate::engine::{
    color::Color,
    color_matrix::ColorMatrix,
    input::{input::Input, key::Key},
    v2::V2,
};

const SHOOT_COOLDOWN: f32 = 0.25;
const MOVE_COOLDOWN: f32 = 0.05;
const BLINK_OFFSETS: [f32; 4] = [0.0, 0.1, 0.25, 0.0];

pub struct BulletSpawn {
    pub pos: V2,
    pub dir: V2,
    pub level: u8,
}

pub struct Tank {
    pub pos: V2,
    pub is_p1: bool,
    pub health: u8,
    pub level: u8,
    pub rotation: i32,
    pub alive: bool,
    shoot_timer: f32,
    move_timer: f32,
    is_visible: bool,
    blink_timer: f32,
}

impl Tank {
    pub fn new(is_p1: bool, pos: V2) -> Self {
        Self {
            pos,
            is_p1,
            health: 3,
            level: 1,
            rotation: if is_p1 { 180 } else { 0 },
            alive: true,
            shoot_timer: 0.0,
            move_timer: 0.0,
            is_visible: true,
            blink_timer: 0.0,
        }
    }

    pub fn tick(&mut self, input: &dyn Input, obstacle: &ObstacleMap, has_bullet: bool, delta_time: f32) -> Option<BulletSpawn> {
        self.tick_move(input, obstacle, delta_time);
        let bullet = self.tick_shoot(input, has_bullet, delta_time);
        self.tick_blink(delta_time);
        bullet
    }

    pub fn take_damage(&mut self) {
        self.health = self.health.saturating_sub(1);
        if self.health == 0 {
            self.alive = false;
        }
    }

    pub fn level_up(&mut self) {
        self.level = (self.level + 1).min(2);
    }

    pub fn level_down(&mut self) {
        self.level = self.level.saturating_sub(1);
    }

    pub fn render(&self) -> ColorMatrix {
        let brightness = if self.is_visible { 255u8 } else { 166u8 };
        let c = Color::new(brightness, brightness, brightness, 255);
        let mut m = ColorMatrix::new(4, 4, Color::none());
        let rot = ((self.rotation % 360) + 360) % 360;

        // level 0: stub gun (1 unit), 2-unit body
        // level 1: 2-unit gun, 2-unit body
        // level 2: 1-unit gun, 3-unit body
        match rot {
            0 => {
                // gun up
                if self.level > 0 {
                    m.set(1, 0, c);
                    m.set(2, 0, c);
                }
                if self.level < 2 {
                    m.set(1, 1, c);
                    m.set(2, 1, c);
                }
                let start = if self.level >= 2 { 1u8 } else { 2u8 };
                for x in 0..4u8 {
                    for y in start..4u8 {
                        m.set(x, y, c);
                    }
                }
            }
            90 => {
                // gun right
                let body_end = if self.level >= 2 { 3u8 } else { 2u8 };
                for y in 0..4u8 {
                    for x in 0..body_end {
                        m.set(x, y, c);
                    }
                }
                if self.level < 2 {
                    m.set(2, 1, c);
                    m.set(2, 2, c);
                }
                if self.level > 0 {
                    m.set(3, 1, c);
                    m.set(3, 2, c);
                }
            }
            180 => {
                // gun down
                let body_end = if self.level >= 2 { 3u8 } else { 2u8 };
                for x in 0..4u8 {
                    for y in 0..body_end {
                        m.set(x, y, c);
                    }
                }
                if self.level < 2 {
                    m.set(1, 2, c);
                    m.set(2, 2, c);
                }
                if self.level > 0 {
                    m.set(1, 3, c);
                    m.set(2, 3, c);
                }
            }
            _ => {
                // gun left (270)
                let body_start = if self.level >= 2 { 1u8 } else { 2u8 };
                for y in 0..4u8 {
                    for x in body_start..4u8 {
                        m.set(x, y, c);
                    }
                }
                if self.level < 2 {
                    m.set(1, 1, c);
                    m.set(1, 2, c);
                }
                if self.level > 0 {
                    m.set(0, 1, c);
                    m.set(0, 2, c);
                }
            }
        }
        m
    }

    fn tick_move(&mut self, input: &dyn Input, obstacle: &ObstacleMap, delta_time: f32) {
        self.move_timer -= delta_time;
        if self.move_timer > 0.0 {
            return;
        }

        let (rotation, by) = if self.is_p1 {
            if input.is_key_press(Key::P1Down) {
                (180, V2::new(0.0, 1.0))
            } else if input.is_key_press(Key::P1Up) {
                (0, V2::new(0.0, -1.0))
            } else if input.is_key_press(Key::P1Right) {
                (90, V2::new(1.0, 0.0))
            } else if input.is_key_press(Key::P1Left) {
                (270, V2::new(-1.0, 0.0))
            } else {
                return;
            }
        } else {
            if input.is_key_press(Key::P2Down) {
                (0, V2::new(0.0, -1.0))
            } else if input.is_key_press(Key::P2Up) {
                (180, V2::new(0.0, 1.0))
            } else if input.is_key_press(Key::P2Left) {
                (270, V2::new(-1.0, 0.0))
            } else if input.is_key_press(Key::P2Right) {
                (90, V2::new(1.0, 0.0))
            } else {
                return;
            }
        };

        self.rotation = rotation;
        self.move_timer = MOVE_COOLDOWN;

        let new_pos = self.pos + by;
        let from = new_pos - V2::one() * 1.5;
        let to = new_pos + V2::one() * 2.5;
        let hit = obstacle.does_collide(from, to);
        if matches!(hit, ObstacleType::None | ObstacleType::Grass) {
            self.pos = new_pos;
        }
    }

    fn tick_shoot(&mut self, input: &dyn Input, has_bullet: bool, delta_time: f32) -> Option<BulletSpawn> {
        self.shoot_timer -= delta_time;
        let shoot_key = if self.is_p1 { Key::P1Blue } else { Key::P2Blue };
        if self.shoot_timer <= 0.0 && !has_bullet && input.is_key_down(shoot_key) {
            self.shoot_timer = SHOOT_COOLDOWN;
            let dir = rotation_to_dir(self.rotation);
            Some(BulletSpawn {
                pos: self.pos + dir * 2.0,
                dir,
                level: self.level,
            })
        } else {
            None
        }
    }

    fn tick_blink(&mut self, delta_time: f32) {
        if self.health < 3 {
            self.blink_timer -= delta_time;
            if self.blink_timer <= 0.0 {
                self.is_visible = !self.is_visible;
                self.blink_timer = BLINK_OFFSETS[self.health as usize];
            }
        }
    }
}

pub fn rotation_to_dir(rotation: i32) -> V2 {
    match ((rotation % 360) + 360) % 360 {
        90 => V2::new(1.0, 0.0),
        180 => V2::new(0.0, 1.0),
        270 => V2::new(-1.0, 0.0),
        _ => V2::new(0.0, -1.0),
    }
}
