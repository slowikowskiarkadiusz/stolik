extern crate alloc;
use crate::engine::{
    ai::{ai_config::AiConfig, ai_input::AiInput, neat_genome::NeatGenome},
    input::{
        gesture::Gestures,
        input::Input,
        key::{KEYS_LENGTH, Key},
    },
};
use alloc::vec::Vec;
use spin::Mutex;

pub struct PongAiData {
    pub inputs: [Vec<f64>; 2],
}

static PONG_AI_DATA: Mutex<Option<PongAiData>> = Mutex::new(None);

pub fn set_pong_ai_data(data: PongAiData) {
    *PONG_AI_DATA.lock() = Some(data);
}

pub struct PongAiInput {
    gestures: Gestures,
    keys_press: [bool; KEYS_LENGTH as usize],
    keys_down: [bool; KEYS_LENGTH as usize],
    keys_up: [bool; KEYS_LENGTH as usize],
    genome: NeatGenome,
    player: usize,
}

impl AiInput for PongAiInput {
    fn new(player: usize) -> Box<dyn AiInput + Send>
    where
        Self: Sized,
    {
        Box::new(Self {
            gestures: Gestures::new(),
            keys_press: [false; KEYS_LENGTH as usize],
            keys_down: [false; KEYS_LENGTH as usize],
            keys_up: [false; KEYS_LENGTH as usize],
            genome: AiConfig::get("pong").load_genome().unwrap(),
            player,
        })
    }

    fn set_genome(&mut self, genome: NeatGenome) {
        self.genome = genome;
    }

    fn get_genome(&self) -> &NeatGenome {
        &self.genome
    }
}

fn pong_outputs_to_keys(outputs: &[f64]) -> [bool; KEYS_LENGTH as usize] {
    let mut keys = [false; KEYS_LENGTH as usize];
    if let Some(&v) = outputs.first() {
        keys[Key::Left as usize] = v > 0.5;
        keys[Key::Right as usize] = v < 0.5;
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

impl Input for PongAiInput {
    fn gestures(&self) -> &Gestures {
        &self.gestures
    }

    fn update(&mut self, delta_time: f32) {
        let inputs = PONG_AI_DATA.lock().as_ref().map(|d| d.inputs[self.player].clone());
        let new_press = if let (Some(inputs), genome) = (inputs, &mut self.genome) {
            if !inputs.is_empty() {
                pong_outputs_to_keys(&genome.activate(inputs))
            } else {
                [false; KEYS_LENGTH as usize]
            }
        } else {
            [false; KEYS_LENGTH as usize]
        };

        for i in 0..KEYS_LENGTH as usize {
            self.keys_down[i] = new_press[i] && !self.keys_press[i];
            self.keys_up[i] = !new_press[i] && self.keys_press[i];
            self.keys_press[i] = new_press[i];
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
