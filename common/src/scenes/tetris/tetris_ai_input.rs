extern crate alloc;
use core::sync::atomic::{AtomicU8, AtomicU16, Ordering};

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
            let mut max_score = 0.0;
            let mut max_score_index = 0;
            let iter: Vec<_> = tetris_ai_data.unwrap()[self.player].iter().collect();
            for d in 0..iter.len() {
                let data = iter[d];
                // if let Some(data) = iter[d] {
                let score = self.genome.activate(data.into_nn_inputs())[0];

                if score > max_score {
                    max_score = score;
                    max_score_index = d;
                }
                // }
            }

            self.goto_x = Some(iter[max_score_index].piece_x);
            self.goto_rotation = Some(iter[max_score_index].current_rotation.iter().position(|f| *f == 1).unwrap() as u16 * 90);

            tetris_ai_data = None;
        } else if let Some(goto_x) = self.goto_x
            && let Some(goto_rotation) = self.goto_rotation
        {
            let center_x = get_tetris_piece_center_x(self.player).load(Ordering::SeqCst);
            if get_tetris_piece_rotation(self.player).load(Ordering::SeqCst) != goto_rotation {
                self.keys_down[Key::Blue as usize] = center_x < goto_x;
            } else {
                self.keys_down[Key::Left as usize] = center_x < goto_x;
                self.keys_down[Key::Right as usize] = center_x > goto_x;
                self.keys_down[Key::Up as usize] = center_x == goto_x;
            }
        }

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
