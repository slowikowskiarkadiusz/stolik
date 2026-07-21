use crate::engine::{
    color::Color,
    color_matrix::ColorMatrix,
    components::{camera::Camera, collider::CollisionResult, world::World},
    engine::{ActorId, SCREEN_SIZE, SCREEN_SIZEUSIZE},
    hash_map::HashMap,
    input::input::Input,
    scene::Scene,
};

static DELAY: f32 = 0.5;

pub struct GameOfLifeScene {
    is_taken: [bool; SCREEN_SIZEUSIZE * SCREEN_SIZEUSIZE],
    timer: f32,
}

impl Scene for GameOfLifeScene {
    fn init(&mut self, world: &mut World) {
        self.set(SCREEN_SIZEUSIZE / 2, SCREEN_SIZEUSIZE / 2, true);
        self.set(SCREEN_SIZEUSIZE / 2, SCREEN_SIZEUSIZE / 2 - 5, true);
        self.set(SCREEN_SIZEUSIZE / 2-1, SCREEN_SIZEUSIZE / 2 - 5, true);
        self.set(SCREEN_SIZEUSIZE / 2+1, SCREEN_SIZEUSIZE / 2 - 5, true);
    }

    fn tick(&mut self, input: &Box<dyn Input>, world: &mut World, delta_time: f32) {
        self.timer -= delta_time;
        if self.timer > 0.0 {
            return;
        }
        self.timer = DELAY;

        let mut new_table: [bool; SCREEN_SIZEUSIZE * SCREEN_SIZEUSIZE] = [false; SCREEN_SIZEUSIZE * SCREEN_SIZEUSIZE];

        for x in 0..SCREEN_SIZEUSIZE {
            for y in 0..SCREEN_SIZEUSIZE {
                let neighbor_count = self.number_of_neighbors(x, y);

                if self.is_taken[y * SCREEN_SIZEUSIZE + x] {
                    new_table[y * SCREEN_SIZEUSIZE + x] = match neighbor_count {
                        0 | 1 => false,
                        2 | 3 => true,
                        _ => false,
                    };
                } else {
                    new_table[y * SCREEN_SIZEUSIZE + x] = neighbor_count == 2 || neighbor_count == 3
                }
            }
        }

        self.is_taken = new_table;
    }

    fn render(&mut self, camera: &Camera, world: &mut World, delta_time: f32) -> ColorMatrix {
        let mut result = ColorMatrix::new(SCREEN_SIZE, SCREEN_SIZE, Color::none());

        for x in 0usize..SCREEN_SIZEUSIZE {
            for y in 0usize..SCREEN_SIZEUSIZE {
                result.set(x as u8, y as u8, if self.get(x, y) { Color::white() } else { Color::black() });
            }
        }

        result
    }

    fn on_overlaps(&mut self, overlaps: &HashMap<ActorId, Vec<ActorId>>, world: &mut World, delta_time: f32) {}

    fn on_collisions(&mut self, collisions: &HashMap<u16, Vec<(u16, CollisionResult)>>, world: &mut World, delta_time: f32) {}
}

impl GameOfLifeScene {
    pub fn new() -> Self {
        Self {
            is_taken: [false; SCREEN_SIZEUSIZE * SCREEN_SIZEUSIZE],
            timer: DELAY,
        }
    }

    fn get(&self, x: usize, y: usize) -> bool {
        self.is_taken[y * SCREEN_SIZEUSIZE + x]
    }

    fn number_of_neighbors(&self, x: usize, y: usize) -> u8 {
        let mut count = 0;
        let x_start = sat_sub(x, 1);
        let x_end = sat_add(x, 1, SCREEN_SIZEUSIZE - 1);
        let y_start = sat_sub(y, 1);
        let y_end = sat_add(y, 1, SCREEN_SIZEUSIZE - 1);

        for ix in x_start..=x_end {
            for iy in y_start..=y_end {
                if (ix != x || iy != y) && self.get(ix, iy) {
                    count += 1;
                }
            }
        }
        count
    }

    fn set(&mut self, x: usize, y: usize, new_val: bool) {
        self.is_taken[y * SCREEN_SIZEUSIZE + x] = new_val
    }
}

fn sat_sub(a: usize, b: usize) -> usize {
    if a > b { a - b } else { 0 }
}

fn sat_add(a: usize, b: usize, max: usize) -> usize {
    if a > max - b { max } else { a + b }
}
