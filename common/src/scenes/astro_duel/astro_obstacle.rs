extern crate alloc;

use alloc::collections::BTreeMap;

use crate::engine::{
    color::Color,
    color_matrix::ColorMatrix,
    components::{
        collider::{Collider, ColliderPart},
        transform::Transform,
        world::World,
    },
    engine::ActorId,
    v2::V2,
};
use rand::{Rng, rngs::SmallRng};

pub const CELL_SIZE: u8 = 4;
pub const BOARD_CELLS: u8 = 16;
const INTERIOR: u8 = BOARD_CELLS - 2;
const INTERIOR_HALF: u8 = INTERIOR / 2;
const OPENING_CELLS: u8 = 6;
const OPENING_START: u8 = (BOARD_CELLS - OPENING_CELLS) / 2;
const OPENING_END: u8 = OPENING_START + OPENING_CELLS;
pub const OPENING_PX_START: f32 = (OPENING_START * CELL_SIZE) as f32;
pub const OPENING_PX_END: f32 = (OPENING_END * CELL_SIZE) as f32;

#[repr(u8)]
#[derive(PartialEq, PartialOrd, Clone, Copy, Debug, Eq, Ord)]
pub enum AstroObstacleType {
    None = 0,
    Brick = 2,
    Steel = 3,
    Edge = 5,
}

pub struct AstroObstacleMap {
    map: BTreeMap<ActorId, AstroObstacleType>,
}

impl AstroObstacleMap {
    pub fn new(rng: &mut SmallRng, world: &mut World) -> Self {
        Self {
            map: generate_interior(rng, world),
        }
    }

    pub fn is_destroyable_obstacle(&self, actor_id: &ActorId) -> bool {
        if let Some(obstacle_type) = self.map.get(actor_id) {
            return obstacle_type != &AstroObstacleType::Steel;
        }
        false
    }

    pub fn render(&self, world: &mut World) -> ColorMatrix {
        let total = BOARD_CELLS as u16 * CELL_SIZE as u16;
        let mut out = ColorMatrix::new(total as u8, total as u8, Color::black());
        for pair in &self.map {
            if let Some(transform) = world.get_transform(&pair.0) {
                draw_cell(&mut out, (transform.center.x / 4.0) as u8, (transform.center.y / 4.0) as u8, pair.1);
            }
        }
        out
    }
}

fn draw_cell(out: &mut ColorMatrix, cx: u8, cy: u8, t: &AstroObstacleType) {
    match t {
        AstroObstacleType::None => {}

        AstroObstacleType::Brick => {
            let light = Color::new(100, 110, 130, 255);
            let dark = Color::new(50, 55, 70, 255);
            let cs = CELL_SIZE as u8;
            let px0 = cx * cs;
            let py0 = cy * cs;
            for dy in 0..cs {
                for dx in 0..cs {
                    let is_odd_row = dy % 2 == 1;
                    let c = if (is_odd_row && dx == cs - 1) || (!is_odd_row && dx == 0) {
                        light
                    } else {
                        dark
                    };
                    out.set(px0 + dx, py0 + dy, c);
                }
            }
        }

        AstroObstacleType::Steel => {
            let light = Color::new(180, 180, 180, 255);
            let dark = Color::new(70, 70, 70, 255);
            let cs = CELL_SIZE as u8;
            let px0 = cx * cs;
            let py0 = cy * cs;
            for dy in 0..cs {
                for dx in 0..cs {
                    let c = if dx == 0 || dx == cs - 1 || dy == 0 || dy == cs - 1 {
                        light
                    } else {
                        dark
                    };
                    out.set(px0 + dx, py0 + dy, c);
                }
            }
        }

        AstroObstacleType::Edge => {}
    }
}

fn is_cell_forbidden(x: u8, y: u8) -> bool {
    x >= (BOARD_CELLS / 2 - 2) && x <= (BOARD_CELLS / 2 + 1) && y <= 2
}

fn generate_interior(rng: &mut SmallRng, world: &mut World) -> BTreeMap<ActorId, AstroObstacleType> {
    let size = INTERIOR;
    let half = INTERIOR_HALF;
    let mut cells: BTreeMap<ActorId, AstroObstacleType> = BTreeMap::new();

    for cy in 0..half {
        for cx in 0..size {
            if !rng.gen_bool(0.22) {
                continue;
            }

            let w = rng.gen_range(1..=3);
            let h = rng.gen_range(1..=2);

            for dy in 0..h {
                for dx in 0..w {
                    let nx = (cx + dx).min(size - 1);
                    let ny = (cy + dy).min(half - 1);
                    if !is_cell_forbidden(nx, ny) {
                        cells.insert(
                            create_collider_actor(
                                V2::new(nx as f32, ny as f32) * CELL_SIZE as f32,
                                V2::one() * CELL_SIZE as f32,
                                world,
                            ),
                            AstroObstacleType::Brick,
                        );

                        //rotate
                        let mx = size - 1 - nx;
                        let my = size - 1 - ny;
                        cells.insert(
                            create_collider_actor(
                                V2::new(mx as f32, my as f32) * CELL_SIZE as f32,
                                V2::one() * CELL_SIZE as f32,
                                world,
                            ),
                            AstroObstacleType::Brick,
                        );
                    }
                }
            }
        }
    }

    for dy in 0..=size {
        cells.insert(
            create_collider_actor(V2::new(0 as f32, dy as f32) * CELL_SIZE as f32, V2::one() * CELL_SIZE as f32, world),
            AstroObstacleType::Steel,
        );

        cells.insert(
            create_collider_actor(
                V2::new(BOARD_CELLS as f32 - 1.0, dy as f32) * CELL_SIZE as f32,
                V2::one() * CELL_SIZE as f32,
                world,
            ),
            AstroObstacleType::Steel,
        );
    }

    for dx in 0..=size + 1 {
        if !is_cell_forbidden(dx, 0) {
            cells.insert(
                create_collider_actor(V2::new(dx as f32, 0.0) * CELL_SIZE as f32, V2::one() * CELL_SIZE as f32, world),
                AstroObstacleType::Steel,
            );
            cells.insert(
                create_collider_actor(
                    V2::new(dx as f32, BOARD_CELLS as f32 - 1.0) * CELL_SIZE as f32,
                    V2::one() * CELL_SIZE as f32,
                    world,
                ),
                AstroObstacleType::Steel,
            );
        }
    }

    cells
}

fn create_collider_actor(center: V2, size: V2, world: &mut World) -> ActorId {
    world.add_new_actor(
        Some(Transform::new(center + V2::one() * (CELL_SIZE as f32 / 2.0), size.clone())),
        Some(Collider::new(vec![ColliderPart::rect(V2::zero(), size.clone(), false)], Some(0))),
        None,
        None, // Some(ColorMatrix::new(size.x as u8, size.y as u8, color)),
    )
}
