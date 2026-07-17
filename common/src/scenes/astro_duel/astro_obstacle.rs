extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

use crate::engine::{
    color::Color,
    color_matrix::ColorMatrix,
    components::{
        collider::{Collider, ColliderPart},
        transform::Transform,
        world::{MAX_ACTORS, World},
    },
    engine::ActorId,
    v2::V2,
};
use rand::{Rng, rngs::SmallRng};

pub const CELL_SIZE: u8 = 4;
pub const CELL_SIZEF32: f32 = CELL_SIZE as f32;
pub const BOARD_CELLS: u8 = 16;
pub const BOARD_CELLSF32: f32 = BOARD_CELLS as f32;
const INTERIOR: u8 = BOARD_CELLS - 2;
const INTERIOR_HALF: u8 = INTERIOR / 2;
const OPENING_CELLS: u8 = 6;
const OPENING_START: u8 = (BOARD_CELLS - OPENING_CELLS) / 2 + 1;
const OPENING_END: u8 = OPENING_START + OPENING_CELLS - 3;

pub struct AstroObstacleMap {
    border_actors: Vec<ActorId>,
    brick_actors: Vec<ActorId>,
}

impl AstroObstacleMap {
    pub fn new(rng: &mut SmallRng, world: &mut World) -> Self {
        let border_actors = create_border_actors(world);
        let brick_actors = generate_interior(rng, world);
        Self {
            border_actors,
            brick_actors,
        }
    }

    pub fn remove(&mut self, id: &ActorId) {
        self.brick_actors.retain(|a| a != id);
    }

    pub fn is_destroyable_obstacle(&self, actor_id: &ActorId) -> bool {
        self.brick_actors.contains(actor_id)
    }

    pub fn is_border_obstacle(&self, actor_id: &ActorId) -> bool {
        self.border_actors.contains(actor_id)
    }

    pub fn render(&self, world: &mut World) -> ColorMatrix {
        let total = (BOARD_CELLS as u16) * (CELL_SIZE as u16);
        let mut out = ColorMatrix::new(total as u8, total as u8, Color::black());

        render_border(&mut out);

        for actor_id in &self.brick_actors {
            if let Some(t) = world.get_transform(actor_id) {
                let cx = (t.center.x / CELL_SIZEF32) as u8;
                let cy = (t.center.y / CELL_SIZEF32) as u8;
                draw_brick(&mut out, cx, cy);
            }
        }

        out
    }
}

fn create_border_actors(world: &mut World) -> Vec<ActorId> {
    let cell = CELL_SIZEF32;
    let board = BOARD_CELLSF32 * cell;

    // Top solid segment: cells gy=1..OPENING_START-1
    let top_h = (OPENING_START - 1) as f32 * cell;
    let top_cy = cell + top_h / 2.0;

    // Bottom solid segment: cells gy=OPENING_END..BOARD_CELLS-2
    let bot_h = (BOARD_CELLS - 1 - OPENING_END) as f32 * cell;
    let bot_cy = OPENING_END as f32 * cell + bot_h / 2.0;

    let segments: [(V2, V2); 6] = [
        // top wall (full width)
        (V2::new(board / 2.0, cell / 2.0),        V2::new(board, cell)),
        // bottom wall (full width)
        (V2::new(board / 2.0, board - cell / 2.0), V2::new(board, cell)),
        // left wall — top segment
        (V2::new(cell / 2.0,        top_cy), V2::new(cell, top_h)),
        // right wall — top segment
        (V2::new(board - cell / 2.0, top_cy), V2::new(cell, top_h)),
        // left wall — bottom segment
        (V2::new(cell / 2.0,        bot_cy), V2::new(cell, bot_h)),
        // right wall — bottom segment
        (V2::new(board - cell / 2.0, bot_cy), V2::new(cell, bot_h)),
    ];

    let mut result: Vec<ActorId> = Vec::new();

    for (center, size) in &segments {
        result.push(world.add_new_actor(
            Some(Transform::new(*center, *size)),
            Some(Collider::new(vec![ColliderPart::rect(V2::zero(), *size, false)], Some(0), true)),
            None,
            None,
        ));
    }

    result
}

fn generate_interior(rng: &mut SmallRng, world: &mut World) -> Vec<ActorId> {
    let size = INTERIOR;
    let half = INTERIOR_HALF;

    let mut occupied = [[false; BOARD_CELLS as usize]; BOARD_CELLS as usize];
    let mut bricks: Vec<ActorId> = Vec::new();

    for cy in 0..half {
        for cx in 0..size {
            if !rng.gen_bool(0.15) {
                continue;
            }

            let w = rng.gen_range(1u8..=2);
            let h = rng.gen_range(1u8..=2);

            for dy in 0..h {
                for dx in 0..w {
                    if world.actor_count + 14 >= MAX_ACTORS {
                        return bricks;
                    }

                    let nx = (cx + dx).min(size - 1);
                    let ny = (cy + dy).min(half - 1);

                    let gx = nx + 1;
                    let gy = ny + 1;
                    let mx = size - nx;
                    let my = size - ny;

                        if !occupied[gx as usize][gy as usize] {
                            occupied[gx as usize][gy as usize] = true;
                            bricks.push(create_brick_actor(gx, gy, world));
                        }
                        if !occupied[mx as usize][my as usize] {
                            occupied[mx as usize][my as usize] = true;
                            bricks.push(create_brick_actor(mx, my, world));
                        }
                }
            }
        }
    }

    bricks
}

fn create_brick_actor(gx: u8, gy: u8, world: &mut World) -> ActorId {
    let center = V2::new(
        gx as f32 * CELL_SIZEF32 + CELL_SIZEF32 / 2.0,
        gy as f32 * CELL_SIZEF32 + CELL_SIZEF32 / 2.0,
    );
    let size = V2::one() * CELL_SIZEF32;
    world.add_new_actor(
        Some(Transform::new(center, size)),
        Some(Collider::new(vec![ColliderPart::rect(V2::zero(), size, false)], Some(0), true)),
        None,
        None,
    )
}

fn render_border(out: &mut ColorMatrix) {
    for gx in 0..BOARD_CELLS {
        draw_steel(out, gx, 0);
        draw_steel(out, gx, BOARD_CELLS - 1);
    }
    for gy in 1..BOARD_CELLS - 1 {
        if gy < OPENING_START || gy >= OPENING_END {
            draw_steel(out, 0, gy);
            draw_steel(out, BOARD_CELLS - 1, gy);
        }
    }
}

fn draw_steel(out: &mut ColorMatrix, cx: u8, cy: u8) {
    let light = Color::new(255, 255, 255, 255);
    let dark = Color::new(160, 160, 170, 255);
    let px0 = cx * CELL_SIZE;
    let py0 = cy * CELL_SIZE;
    for dy in 0..CELL_SIZE {
        for dx in 0..CELL_SIZE {
            let c = if dx == 0 || dx == CELL_SIZE - 1 || dy == 0 || dy == CELL_SIZE - 1 {
                light
            } else {
                dark
            };
            out.set(px0 + dx, py0 + dy, c);
        }
    }
}

fn draw_brick(out: &mut ColorMatrix, cx: u8, cy: u8) {
    let light = Color::new(51, 255, 125, 255);
    let dark = Color::new(51, 255, 125, 180);
    let px0 = cx * CELL_SIZE;
    let py0 = cy * CELL_SIZE;
    for dy in 0..CELL_SIZE {
        for dx in 0..CELL_SIZE {
            let is_odd_row = dy % 2 == 1;
            let c = if (is_odd_row && dx == CELL_SIZE - 1) || (!is_odd_row && dx == 0) {
                light
            } else {
                dark
            };
            out.set(px0 + dx, py0 + dy, c);
        }
    }
}
