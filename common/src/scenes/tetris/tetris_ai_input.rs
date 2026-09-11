extern crate alloc;
use core::sync::atomic::{AtomicBool, AtomicU8, AtomicU16, AtomicU64, Ordering};
use std::println;

use crate::{
    engine::{
        ai::{ai_config::AiConfig, ai_input::AiInput, neat_genome::NeatGenome},
        input::{
            gesture::Gestures,
            input::Input,
            key::{KEYS_LENGTH, Key},
        },
    },
    scenes::tetris::board::{BOARD_WIDTH, TetrisAiData},
};
use alloc::vec::Vec;
use spin::Mutex;

static TETRIS_PIECE_ROTATION_0: AtomicU16 = AtomicU16::new(0);
static TETRIS_PIECE_ROTATION_1: AtomicU16 = AtomicU16::new(0);
static TETRIS_PIECE_CENTER_X_0: AtomicU8 = AtomicU8::new(0);
static TETRIS_PIECE_CENTER_X_1: AtomicU8 = AtomicU8::new(0);
static TETRIS_AI_DATA_0: Mutex<Option<[Option<TetrisAiData>; BOARD_WIDTH as usize * 4]>> = Mutex::new(None);
static TETRIS_AI_DATA_1: Mutex<Option<[Option<TetrisAiData>; BOARD_WIDTH as usize * 4]>> = Mutex::new(None);
static TETRIS_FITNESS_0: AtomicU64 = AtomicU64::new(0);
static TETRIS_FITNESS_1: AtomicU64 = AtomicU64::new(0);

static TETRIS_NEEDS_AI_DATA_0: AtomicBool = AtomicBool::new(false);
static TETRIS_NEEDS_AI_DATA_1: AtomicBool = AtomicBool::new(false);

pub fn take_tetris_needs_ai_data(player: u8) -> bool {
    match player {
        0 => TETRIS_NEEDS_AI_DATA_0.swap(false, Ordering::Relaxed),
        1 => TETRIS_NEEDS_AI_DATA_1.swap(false, Ordering::Relaxed),
        _ => false,
    }
}

fn request_tetris_ai_data(player: usize) {
    match player {
        0 => TETRIS_NEEDS_AI_DATA_0.store(true, Ordering::Relaxed),
        1 => TETRIS_NEEDS_AI_DATA_1.store(true, Ordering::Relaxed),
        _ => {}
    }
}

pub fn set_tetris_fitness(player: u8, points: f64) {
    let bits = points.to_bits();
    match player {
        0 => TETRIS_FITNESS_0.store(bits, Ordering::Relaxed),
        1 => TETRIS_FITNESS_1.store(bits, Ordering::Relaxed),
        _ => {}
    }
}

pub fn set_tetris_ai_data(player: u8, newdata: [Option<TetrisAiData>; BOARD_WIDTH as usize * 4]) {
    match player {
        0 => *TETRIS_AI_DATA_0.lock() = Some(newdata),
        1 => *TETRIS_AI_DATA_1.lock() = Some(newdata),
        _ => panic!("[set_tetris_ai_data] the player parameter has to be either 0 or 1"),
    }
}

pub fn get_tetris_ai_data(player: u8) -> &'static Mutex<Option<[Option<TetrisAiData>; BOARD_WIDTH as usize * 4]>> {
    match player {
        0 => &TETRIS_AI_DATA_0,
        1 => &TETRIS_AI_DATA_1,
        _ => panic!("[get_tetris_ai_data] the player parameter has to be either 0 or 1"),
    }
}

pub fn set_tetris_piece_center_x(player: u8, newx: u8) {
    match player {
        0 => TETRIS_PIECE_CENTER_X_0.store(newx, Ordering::SeqCst),
        1 => TETRIS_PIECE_CENTER_X_1.store(newx, Ordering::SeqCst),
        _ => panic!("[set_tetris_piece_center_x] the player parameter has to be either 0 or 1"),
    }
}

pub fn get_tetris_piece_center_x(player: usize) -> &'static AtomicU8 {
    match player {
        0 => &TETRIS_PIECE_CENTER_X_0,
        1 => &TETRIS_PIECE_CENTER_X_1,
        _ => panic!("[get_tetris_piece_center_x] the player parameter has to be either 0 or 1"),
    }
}

pub fn set_tetris_piece_rotation(player: u8, newrotation: u16) {
    match player {
        0 => TETRIS_PIECE_ROTATION_0.store(newrotation, Ordering::SeqCst),
        1 => TETRIS_PIECE_ROTATION_1.store(newrotation, Ordering::SeqCst),
        _ => panic!("[set_tetris_piece_rotation] the player parameter has to be either 0 or 1"),
    }
}

pub fn get_tetris_piece_rotation(player: usize) -> &'static AtomicU16 {
    match player {
        0 => &TETRIS_PIECE_ROTATION_0,
        1 => &TETRIS_PIECE_ROTATION_1,
        _ => panic!("[get_tetris_piece_rotation] the player parameter has to be either 0 or 1"),
    }
}

pub struct TetrisAiInput {
    gestures: Gestures,
    keys_press: [bool; KEYS_LENGTH as usize],
    keys_down: [bool; KEYS_LENGTH as usize],
    keys_up: [bool; KEYS_LENGTH as usize],
    genome: NeatGenome,
    player: usize,
    goto_x: Option<u8>,
    goto_rotation: Option<u16>,
    last_center_x: u8,
}

impl AiInput for TetrisAiInput {
    fn new(player: usize) -> Box<dyn AiInput + Send>
    where
        Self: Sized,
    {
        Box::new(Self {
            gestures: Gestures::new(),
            keys_press: [false; KEYS_LENGTH as usize],
            keys_down: [false; KEYS_LENGTH as usize],
            keys_up: [false; KEYS_LENGTH as usize],
            genome: AiConfig::get("tetris").load_genome().unwrap(),
            player,
            goto_x: None,
            goto_rotation: None,
            last_center_x: 0,
        })
    }

    fn set_genome(&mut self, genome: NeatGenome) {
        self.genome = genome;
    }

    fn get_genome(&self) -> &NeatGenome {
        &self.genome
    }
}

impl Input for TetrisAiInput {
    fn gestures(&self) -> &Gestures {
        &self.gestures
    }

    fn update(&mut self, delta_time: f32) {
        let mut binding = get_tetris_ai_data(self.player as u8).lock();
        let mut tetris_ai_data = binding.as_mut();
        if tetris_ai_data.is_some() {
            let mut max_score = f64::NEG_INFINITY;
            let mut best_idx: Option<usize> = None;
            let iter: Vec<_> = tetris_ai_data.unwrap().iter().enumerate().filter_map(|(i, x)| x.as_ref().map(|d| (i, d))).collect();
            for (idx, data) in &iter {
                let score = self.genome.activate(data.into_nn_inputs())[0];
                if score > max_score {
                    max_score = score;
                    best_idx = Some(*idx);
                }
            }

            if let Some(idx) = best_idx {
                if let Some((_, data)) = iter.iter().find(|(i, _)| *i == idx) {
                    self.goto_x = Some(data.piece_x);
                    self.goto_rotation = Some((idx % 4) as u16 * 90);
                }
            }

            *binding = None;
        } else if let Some(goto_x) = self.goto_x
            && let Some(goto_rotation) = self.goto_rotation
        {
            let center_x = get_tetris_piece_center_x(self.player).load(Ordering::SeqCst);
            let current_rotation = get_tetris_piece_rotation(self.player).load(Ordering::SeqCst);
            let rotation_ok = current_rotation == goto_rotation;
            let x_ok = center_x == goto_x;
            let stuck = !x_ok && center_x == self.last_center_x;
            if stuck {
                self.goto_x = None;
                self.goto_rotation = None;
                request_tetris_ai_data(self.player);
            } else {
                if !rotation_ok {
                    self.keys_down[Key::Blue as usize] = true;
                }
                self.keys_down[Key::Right as usize] = center_x < goto_x;
                self.keys_down[Key::Left as usize] = center_x > goto_x;
                self.keys_down[Key::Up as usize] = x_ok && rotation_ok;
            }
            self.last_center_x = center_x;
        }

        let fitness_bits = match self.player {
            0 => TETRIS_FITNESS_0.load(Ordering::Relaxed),
            1 => TETRIS_FITNESS_1.load(Ordering::Relaxed),
            _ => 0,
        };
        self.genome.fitness = f64::from_bits(fitness_bits);

        self.gestures.tick(self.get_snapshot(), delta_time);
    }

    fn late_update(&mut self, _: f32) {
        self.keys_down = [false; KEYS_LENGTH as usize];
        self.keys_up = [false; KEYS_LENGTH as usize];
        self.gestures.late_tick();
    }

    fn is_key_down(&self, key: Key) -> bool {
        key_indices(key).iter().any(|&i| self.keys_down[i])
    }

    fn is_any_key_down(&self) -> bool {
        self.keys_down.iter().any(|&x| x)
    }

    fn is_key_up(&self, key: Key) -> bool {
        key_indices(key).iter().any(|&i| self.keys_up[i])
    }

    fn is_any_key_up(&self) -> bool {
        self.keys_up.iter().any(|&x| x)
    }

    fn is_key_press(&self, key: Key) -> bool {
        key_indices(key).iter().any(|&i| self.keys_press[i])
    }

    fn is_any_key_press(&self) -> bool {
        self.keys_press.iter().any(|&x| x)
    }

    fn clear(&mut self) {
        self.keys_down = [false; KEYS_LENGTH as usize];
        self.keys_up = [false; KEYS_LENGTH as usize];
        self.keys_press = [false; KEYS_LENGTH as usize];
    }

    fn as_ai_input(&self) -> Option<&dyn AiInput> {
        Some(self)
    }
}

fn tetris_outputs_to_keys(outputs: &[f64]) -> [bool; KEYS_LENGTH as usize] {
    let mut keys = [false; KEYS_LENGTH as usize];
    if outputs.len() >= 5 {
        keys[Key::Left as usize] = outputs[0] < 0.4;
        keys[Key::Right as usize] = outputs[0] > 0.6;
        keys[Key::Up as usize] = outputs[1] > 0.5;
        keys[Key::Down as usize] = outputs[2] > 0.5;
        keys[Key::Blue as usize] = outputs[3] > 0.5;
        keys[Key::Green as usize] = outputs[4] > 0.5;
    }
    keys
}

fn key_indices(key: Key) -> Vec<usize> {
    match key {
        Key::AnyDirection => alloc::vec![Key::Up as usize, Key::Down as usize, Key::Left as usize, Key::Right as usize],
        Key::Any => alloc::vec![
            Key::Up as usize,
            Key::Down as usize,
            Key::Left as usize,
            Key::Right as usize,
            Key::Blue as usize,
            Key::Green as usize
        ],
        k => alloc::vec![k as usize],
    }
}
