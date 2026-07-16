use super::obstacle::{ObstacleMap, ObstacleType};
use super::tank::Tank;
use crate::engine::{color::Color, color_matrix::ColorMatrix, v2::V2};

const SPEED: f32 = 20.0;
const HALF: f32 = 1.0;

pub struct Bullet {
    pub pos: V2,
    pub dir: V2,
    pub level: u8,
    pub from_p1: bool,
    pub alive: bool,
}

impl Bullet {
    pub fn new(pos: V2, dir: V2, level: u8, from_p1: bool) -> Self {
        Self {
            pos,
            dir,
            level,
            from_p1,
            alive: true,
        }
    }

    pub fn tick(&mut self, obstacle: &mut ObstacleMap, tank1: &mut Tank, tank2: &mut Tank, delta_time: f32) {
        self.pos = self.pos + self.dir * SPEED * delta_time;

        let from = self.pos - V2::one() * HALF;
        let to = self.pos + V2::one() * HALF;
        let hit = obstacle.blockers_at(from, to);

        if *hit.get(&ObstacleType::None).unwrap_or(&false) {
            if !self.from_p1 && tank1.alive && overlaps(self.pos, tank1.pos) {
                tank1.take_damage();
                self.impact(obstacle);
                self.alive = false;
            } else if self.from_p1 && tank2.alive && overlaps(self.pos, tank2.pos) {
                tank2.take_damage();
                self.impact(obstacle);
                self.alive = false;
            }
        } else if *hit.get(&ObstacleType::Grass).unwrap_or(&false) {
            if self.level == 1 {
                self.impact(obstacle);
            }
        } else if *hit.get(&ObstacleType::Water).unwrap_or(&false) {
            if self.level == 2 {
                self.impact(obstacle);
            }
        } else if *hit.get(&ObstacleType::Brick).unwrap_or(&false)
            || *hit.get(&ObstacleType::Steel).unwrap_or(&false)
            || *hit.get(&ObstacleType::Edge).unwrap_or(&false)
        {
            self.impact(obstacle);
            self.alive = false;
        }
    }

    pub fn render(&self) -> ColorMatrix {
        ColorMatrix::new(2, 2, Color::white())
    }

    fn impact(&self, obstacle: &mut ObstacleMap) {
        let from = self.pos - V2::one() * (HALF + 0.5);
        let to = self.pos + V2::one() * (HALF + 0.5);
        obstacle.remove_at(from, to, self.level);
    }
}

fn overlaps(bullet: V2, tank: V2) -> bool {
    (bullet.x - tank.x).abs() < HALF + 2.0 && (bullet.y - tank.y).abs() < HALF + 2.0
}
