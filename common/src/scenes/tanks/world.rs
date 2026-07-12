use rand::{SeedableRng, rngs::SmallRng};
use crate::engine::v2::V2;
use super::obstacle::{ObstacleMap, BORDER, CELL_SIZE};
use super::tank::Tank;
use super::bullet::Bullet;

pub const BOARD_SIZE: u8 = 11;

pub struct TanksWorld {
    pub obstacle: ObstacleMap,
    pub tank1: Tank,
    pub tank2: Tank,
    pub bullet_p1: Option<Bullet>,
    pub bullet_p2: Option<Bullet>,
    pub winner: Option<u8>,
    pub game_over_timer: f32,
}

impl TanksWorld {
    pub fn new() -> Self {
        let mut rng = SmallRng::seed_from_u64(embassy_time::Instant::now().as_micros());
        let obstacle = ObstacleMap::new(BOARD_SIZE, &mut rng);

        let p2_pos = cell_to_px(0, 0);
        let p1_pos = cell_to_px(BOARD_SIZE - 1, BOARD_SIZE - 1);

        Self {
            obstacle,
            tank1: Tank::new(true, p1_pos),
            tank2: Tank::new(false, p2_pos),
            bullet_p1: None,
            bullet_p2: None,
            winner: None,
            game_over_timer: 0.0,
        }
    }
}

pub fn cell_to_px(cx: u8, cy: u8) -> V2 {
    V2::new(
        BORDER as f32 + cx as f32 * CELL_SIZE as f32 + CELL_SIZE as f32 / 2.0,
        BORDER as f32 + cy as f32 * CELL_SIZE as f32 + CELL_SIZE as f32 / 2.0,
    )
}
