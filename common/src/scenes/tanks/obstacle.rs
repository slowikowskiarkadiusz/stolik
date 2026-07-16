extern crate alloc;
use std::collections::BTreeMap;

use crate::engine::{color::Color, color_matrix::ColorMatrix, matrix::Matrix, v2::V2};
use rand::{Rng, rngs::SmallRng};

pub const CELL_SIZE: u8 = 4;
pub const BORDER: u8 = 0;

#[repr(u8)]
#[derive(PartialEq, PartialOrd, Clone, Copy, Debug, Eq, Ord)]
pub enum ObstacleType {
    None = 0,
    Grass = 1,
    Brick = 2,
    Steel = 3,
    Water = 4,
    Edge = 5,
}

pub struct ObstacleMap {
    pub cells: Matrix<ObstacleType>,
    pub board_size: u8,
}

impl ObstacleMap {
    pub fn new(board_size: u8, rng: &mut SmallRng) -> Self {
        let cells = generate_map(board_size, rng);
        Self { cells, board_size }
    }

    pub fn blockers_at(&self, from: V2, to: V2) -> BTreeMap<ObstacleType, bool> {
        let mut result: BTreeMap<ObstacleType, bool> = BTreeMap::new();

        for px in (from.x as i32)..(to.x as i32) {
            for py in (from.y as i32)..(to.y as i32) {
                if let Some((cx, cy)) = self.pixel_to_cell(px, py) {
                    let t = *self.cells.get(cx, cy);
                    result.insert(t, true);
                } else {
                    result.insert(ObstacleType::Edge, true);
                }
            }
        }

        result
    }

    pub fn remove_at(&mut self, from: V2, to: V2, level: u8) {
        for px in (from.x as i32)..=(to.x as i32) {
            for py in (from.y as i32)..=(to.y as i32) {
                if let Some((cx, cy)) = self.pixel_to_cell(px, py) {
                    let t = *self.cells.get(cx, cy);
                    let removable = match t {
                        ObstacleType::Brick => true,
                        ObstacleType::Grass => level >= 1,
                        ObstacleType::Steel | ObstacleType::Water => level >= 2,
                        _ => false,
                    };
                    if removable {
                        self.cells.set(cx, cy, ObstacleType::None);
                    }
                }
            }
        }
    }

    pub fn render(&self) -> ColorMatrix {
        let total = BORDER as u16 * 2 + self.board_size as u16 * CELL_SIZE as u16;
        let mut out = ColorMatrix::new(total as u8, total as u8, Color::black());
        for cy in 0..self.board_size {
            for cx in 0..self.board_size {
                let t = *self.cells.get(cx, cy);
                draw_cell(&mut out, cx, cy, t);
            }
        }
        out
    }

    fn pixel_to_cell(&self, px: i32, py: i32) -> Option<(u8, u8)> {
        let ox = px - BORDER as i32;
        let oy = py - BORDER as i32;
        if ox < 0 || oy < 0 {
            return None;
        }
        let cx = ox / CELL_SIZE as i32;
        let cy = oy / CELL_SIZE as i32;
        if cx >= self.board_size as i32 || cy >= self.board_size as i32 {
            return None;
        }
        Some((cx as u8, cy as u8))
    }
}

// ── pixel art per cell type, matches C++ draw_one ────────────────────────────

fn draw_cell(out: &mut ColorMatrix, cx: u8, cy: u8, t: ObstacleType) {
    let px0 = BORDER + cx * CELL_SIZE;
    let py0 = BORDER + cy * CELL_SIZE;
    let s = CELL_SIZE as usize;

    match t {
        ObstacleType::None => {}

        ObstacleType::Grass => {
            let grass = Color::new(168, 230, 29, 255);
            for dy in 0..s {
                for dx in 0..s {
                    if (dy % 2 == 0) == (dx % 2 == 0) {
                        out.set(px0 + dx as u8, py0 + dy as u8, grass);
                    }
                }
            }
        }

        ObstacleType::Brick => {
            let light = Color::new(156, 90, 60, 255);
            let dark = Color::new(138, 63, 30, 255);
            let third = Color::new(138, 87, 65, 255);
            for dy in 0..s {
                let is_odd = dy % 2 == 1;
                for dx in 0..s {
                    let c = if (is_odd && dx == s - 1) || (!is_odd && dx == 0) {
                        third
                    } else if is_odd {
                        light
                    } else {
                        dark
                    };
                    out.set(px0 + dx as u8, py0 + dy as u8, c);
                }
            }
        }

        ObstacleType::Steel => {
            let light = Color::new(180, 180, 180, 255);
            let dark = Color::new(70, 70, 70, 255);
            for dy in 0..s {
                for dx in 0..s {
                    let c = if dx == 0 || dx == s - 1 || dy == 0 || dy == s - 1 {
                        light
                    } else {
                        dark
                    };
                    out.set(px0 + dx as u8, py0 + dy as u8, c);
                }
            }
        }

        ObstacleType::Water => {
            let light = Color::new(153, 217, 234, 255);
            let dark = Color::new(77, 109, 243, 255);
            for dy in 0..s {
                for dx in 0..s {
                    let c = if (dy % 2 == 0) == (dx % 2 == 0) { light } else { dark };
                    out.set(px0 + dx as u8, py0 + dy as u8, c);
                }
            }
        }

        ObstacleType::Edge => {}
    }
}

fn generate_map(board_size: u8, rng: &mut SmallRng) -> Matrix<ObstacleType> {
    let bs = board_size as usize;
    let mut cells = Matrix::new(board_size, board_size, ObstacleType::None);

    for cx in 0..board_size {
        for cy in 0..(board_size / 2) {
            let obstacle = randomize_obstacle_type(rng);
            match obstacle {
                ObstacleType::None | ObstacleType::Edge => {}
                ObstacleType::Grass => {
                    generate_obstacle(
                        board_size,
                        cx,
                        cy,
                        ObstacleType::Grass,
                        1,
                        3,
                        1,
                        4,
                        &[ObstacleType::Brick],
                        &mut cells,
                        rng,
                    );
                }
                ObstacleType::Brick => {
                    generate_obstacle(board_size, cx, cy, ObstacleType::Brick, 1, 3, 1, 2, &[], &mut cells, rng);
                }
                ObstacleType::Steel => {
                    generate_obstacle(
                        board_size,
                        cx,
                        cy,
                        ObstacleType::Steel,
                        0,
                        3,
                        0,
                        3,
                        &[ObstacleType::Brick, ObstacleType::Grass],
                        &mut cells,
                        rng,
                    );
                }
                ObstacleType::Water => {
                    generate_obstacle(
                        board_size,
                        cx,
                        cy,
                        ObstacleType::Water,
                        1,
                        3,
                        1,
                        3,
                        &[ObstacleType::Brick, ObstacleType::Grass, ObstacleType::Steel],
                        &mut cells,
                        rng,
                    );
                }
            }
        }
    }

    let mid = board_size / 2;
    let excluded: &[(i32, i32)] = &[
        (0, 0),
        (1, 0),
        (1, 1),
        (0, 1),
        (mid as i32, 0),
        (mid as i32 - 1, 0),
        (mid as i32 + 1, 0),
        (mid as i32 + 2, 0),
        (mid as i32 - 2, 0),
        (mid as i32, 1),
        (mid as i32 - 1, 1),
        (mid as i32 + 1, 1),
        (mid as i32 + 2, 1),
        (mid as i32 - 2, 1),
    ];
    for &(ex, ey) in excluded {
        if ex >= 0 && ey >= 0 && (ex as usize) < bs && (ey as usize) < bs {
            cells.set(ex as u8, ey as u8, ObstacleType::None);
        }
        let mx = (board_size as i32 - 1 - ex) as usize;
        let my = (board_size as i32 - 1 - ey) as usize;
        if mx < bs && my < bs {
            cells.set(mx as u8, my as u8, ObstacleType::None);
        }
    }

    cells
}

fn generate_obstacle(
    board_size: u8,
    at_x: u8,
    at_y: u8,
    t: ObstacleType,
    min_extra_rows: u32,
    max_extra_rows: u32,
    min_continue_for: u32,
    max_continue_for: u32,
    override_types: &[ObstacleType],
    cells: &mut Matrix<ObstacleType>,
    rng: &mut SmallRng,
) {
    let bs = board_size as i32;
    let half = bs / 2;
    let do_columns = rng.gen_bool(0.5);

    let additional_rows = {
        let r = rng.gen_range(0..4u32) + min_extra_rows;
        r.min(max_extra_rows)
    };
    let bound_for = if do_columns {
        ((half - at_y as i32).max(1)) as u32
    } else {
        ((bs - 1 - at_x as i32).max(1)) as u32
    };
    let continue_for = {
        let r = rng.gen_range(0..bound_for.max(1)) + min_continue_for;
        r.min(max_continue_for)
    };

    for ii in 0..additional_rows {
        for i in 0..continue_for {
            let x = (if do_columns { ii } else { i }) as i32 + at_x as i32;
            let y = (if do_columns { i } else { ii }) as i32 + at_y as i32;

            if x >= bs || y >= bs {
                continue;
            }
            let x = x as u8;
            let y = y as u8;

            if *cells.get(x, y) != ObstacleType::None && override_types.contains(cells.get(x, y)) {
                cells.set(x, y, ObstacleType::None);
                let mx = (board_size as i32 - 1 - x as i32) as u8;
                let my = (board_size as i32 - 1 - y as i32) as u8;
                if *cells.get(mx, my) != ObstacleType::None {
                    cells.set(mx, my, ObstacleType::None);
                }
            }

            if *cells.get(x, y) == ObstacleType::None {
                cells.set(x, y, t);
                if (y as i32) < half {
                    let mx = (board_size as i32 - 1 - x as i32) as u8;
                    let my = (board_size as i32 - 1 - y as i32) as u8;
                    if (mx as i32) < bs && (my as i32) < bs {
                        if *cells.get(mx, my) != ObstacleType::None {
                            cells.set(mx, my, ObstacleType::None);
                        }
                        cells.set(mx, my, t);
                    }
                }
            }
        }
    }
}

fn randomize_obstacle_type(rng: &mut SmallRng) -> ObstacleType {
    let weights = [5.0, 0.3, 1.2, 0.3, 0.2];
    let total: f32 = weights.iter().sum();
    let mut r = rng.r#gen::<f32>() * total;
    for (i, &w) in weights.iter().enumerate() {
        r -= w;
        if r <= 0.0 {
            return match i {
                0 => ObstacleType::None,
                1 => ObstacleType::Grass,
                2 => ObstacleType::Brick,
                3 => ObstacleType::Steel,
                _ => ObstacleType::Water,
            };
        }
    }
    ObstacleType::Brick
}
