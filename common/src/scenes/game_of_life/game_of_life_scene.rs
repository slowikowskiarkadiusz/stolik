extern crate alloc;

use alloc::{boxed::Box, vec::Vec};

use crate::engine::{
    color::Color,
    color_matrix::ColorMatrix,
    components::{camera::Camera, collider::CollisionResult, world::World},
    engine::{ActorId, SCREEN_SIZE, SCREEN_SIZEF32, SCREEN_SIZEUSIZE},
    hash_map::HashMap,
    input::{input::Input, key::Key},
    scene::Scene,
    v2::V2,
};

static TURN_DELAY: f32 = 0.5;
static TURN_DELAY_STEP: f32 = 0.1;
static SELECTION_BLINK_DELAY: f32 = 0.5;

pub struct GameOfLifeScene {
    is_playing: bool,
    is_taken: [bool; SCREEN_SIZEUSIZE * SCREEN_SIZEUSIZE],
    turn_delay: f32,
    turn_timer: f32,
    currently_selected: V2,
    selection_blink_timer: f32,
}

impl Scene for GameOfLifeScene {
    fn init(&mut self, _world: &mut World) {}

    fn tick(&mut self, input: &Box<dyn Input>, _world: &mut World, delta_time: f32) {
        if input.is_key_down(Key::P1Green) {
            self.is_playing = !self.is_playing;
        }

        if !self.is_playing {
            self.selection_blink_timer -= delta_time;
            if self.selection_blink_timer <= 0.0 {
                self.selection_blink_timer = TURN_DELAY;
            }

            let dif = V2::new(
                if input.is_key_down(Key::P1Right) {
                    1.0
                } else if input.is_key_down(Key::P1Left) {
                    -1.0
                } else {
                    0.0
                },
                if input.is_key_down(Key::P1Down) {
                    1.0
                } else if input.is_key_down(Key::P1Up) {
                    -1.0
                } else {
                    0.0
                },
            );

            if dif.mag() > 0.0 {
                self.selection_blink_timer = SELECTION_BLINK_DELAY;
            }

            if input.is_key_down(Key::P1Blue) {
                let csx = self.currently_selected.x as usize;
                let csy = self.currently_selected.y as usize;
                self.set(csx, csy, !self.get(csx, csy));
            }

            self.currently_selected += dif;
        } else {
            self.turn_timer -= delta_time;
            if self.turn_timer <= 0.0 {
                self.turn_timer = self.turn_delay;

                let mut new_table: [bool; SCREEN_SIZEUSIZE * SCREEN_SIZEUSIZE] = [false; SCREEN_SIZEUSIZE * SCREEN_SIZEUSIZE];

                for x in 0..SCREEN_SIZEUSIZE {
                    for y in 0..SCREEN_SIZEUSIZE {
                        let neighbor_count = self.number_of_neighbors(x, y);

                        if self.get(x, y) {
                            new_table[y * SCREEN_SIZEUSIZE + x] = matches!(neighbor_count, 2 | 3);
                        } else {
                            new_table[y * SCREEN_SIZEUSIZE + x] = neighbor_count == 3;
                        }
                    }
                }

                self.is_taken = new_table;
            }

            if input.is_key_down(Key::P1Down) {
                self.turn_delay += TURN_DELAY_STEP;
            }
            if input.is_key_down(Key::P1Up) && self.turn_delay >= TURN_DELAY_STEP {
                self.turn_delay -= TURN_DELAY_STEP;
            }
        }
    }

    fn render(&mut self, _camera: &Camera, _world: &mut World, _delta_time: f32) -> ColorMatrix {
        let mut result = ColorMatrix::new(SCREEN_SIZE, SCREEN_SIZE, Color::none());

        for x in 0usize..SCREEN_SIZEUSIZE {
            for y in 0usize..SCREEN_SIZEUSIZE {
                result.set(x as u8, y as u8, if self.get(x, y) { Color::white() } else { Color::black() });
            }
        }

        if !self.is_playing {
            let csx = self.currently_selected.x as u8;
            let csy = self.currently_selected.y as u8;

            result.set(
                csx,
                csy,
                if self.selection_blink_timer > SELECTION_BLINK_DELAY / 2.0 {
                    *Color::white().a(125)
                } else if self.get(csx as usize, csy as usize) {
                    Color::white()
                } else {
                    Color::black()
                },
            );
        }

        result
    }

    fn on_overlaps(&mut self, _overlaps: &HashMap<ActorId, Vec<ActorId>>, _world: &mut World, _delta_time: f32) {}

    fn on_collisions(&mut self, _collisions: &HashMap<u16, Vec<(u16, CollisionResult)>>, _world: &mut World, _delta_time: f32) {}
}

impl GameOfLifeScene {
    pub fn new() -> Self {
        Self {
            is_playing: false,
            is_taken: [false; SCREEN_SIZEUSIZE * SCREEN_SIZEUSIZE],
            turn_delay: TURN_DELAY,
            turn_timer: TURN_DELAY,
            currently_selected: V2::new(SCREEN_SIZEF32 / 2.0, SCREEN_SIZEF32 / 2.0),
            selection_blink_timer: SELECTION_BLINK_DELAY,
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
