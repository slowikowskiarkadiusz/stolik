extern crate alloc;
pub use crate::engine::ai::ai_input::AiInput;
use crate::engine::{
    ai::{ai_config::AiConfig, neat_genome::NeatGenome},
    input::{
        gesture::Gestures,
        input::Input,
        key::{KEYS_LENGTH, Key, KeyState},
    },
};
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::Mutex;

pub struct DefaultAiInput {
    gestures: Gestures,
    pub held: Arc<Mutex<[bool; KEYS_LENGTH as usize]>>,
    keys_down: [bool; KEYS_LENGTH as usize],
    keys_press: [bool; KEYS_LENGTH as usize],
    keys_up: [bool; KEYS_LENGTH as usize],
    genome: Option<NeatGenome>,
}

impl DefaultAiInput {
    pub fn new() -> Self {
        Self {
            gestures: Gestures::new(),
            held: Arc::new(Mutex::new([false; KEYS_LENGTH as usize])),
            keys_down: [false; KEYS_LENGTH as usize],
            keys_press: [false; KEYS_LENGTH as usize],
            keys_up: [false; KEYS_LENGTH as usize],
            genome: None,
        }
    }

    pub fn new_with_ai(game_name: &'static str) -> Self {
        Self {
            gestures: Gestures::new(),
            held: Arc::new(Mutex::new([false; KEYS_LENGTH as usize])),
            keys_down: [false; KEYS_LENGTH as usize],
            keys_press: [false; KEYS_LENGTH as usize],
            keys_up: [false; KEYS_LENGTH as usize],
            genome: AiConfig::get(game_name).load_genome(),
        }
    }

    fn is_key(&self, key: Option<Key>, key_state: KeyState) -> bool {
        fn func(key: Option<Key>, arr: &[bool]) -> bool {
            if let Some(k) = key {
                return DefaultAiInput::map_key(k).iter().any(|kk: &Key| arr[*kk as usize]);
            } else {
                return arr.iter().any(|x| !!x);
            }
        }

        match key_state {
            KeyState::Down => func(key, &self.keys_down),
            KeyState::Up => func(key, &self.keys_up),
            KeyState::Press => func(key, &self.keys_press),
        }
    }

    fn map_key(key: Key) -> Vec<Key> {
        match key {
            Key::Down | Key::Up | Key::Left | Key::Right | Key::Blue | Key::Green | Key::Start => alloc::vec![key],
            Key::AnyDirection => alloc::vec![Key::Up, Key::Down, Key::Left, Key::Right],
            Key::Any => alloc::vec![Key::Up, Key::Down, Key::Left, Key::Right, Key::Blue, Key::Green],
        }
    }
}

impl AiInput for DefaultAiInput {
    fn on_ai_data(&mut self, inputs: &[f64], outputs_to_keys: fn(&[f64]) -> [bool; KEYS_LENGTH as usize]) {
        if let Some(genome) = &mut self.genome {
            let outputs = genome.activate(inputs.to_vec());
            *self.held.lock() = outputs_to_keys(&outputs);
        }
    }
}

impl Input for DefaultAiInput {
    fn gestures(&self) -> &Gestures {
        &self.gestures
    }

    fn update(&mut self, delta_time: f32) {
        let keys = *self.held.lock();
        for i in 0..KEYS_LENGTH as usize {
            let was = self.keys_press[i];
            let is = keys[i];
            self.keys_down[i] = is && !was;
            self.keys_up[i] = !is && was;
            self.keys_press[i] = is;
        }
        self.gestures.tick(self.get_snapshot(), delta_time);
    }

    fn late_update(&mut self, _: f32) {
        self.keys_down = [false; KEYS_LENGTH as usize];
        self.keys_up = [false; KEYS_LENGTH as usize];

        self.gestures.late_tick();
    }

    fn is_key_down(&self, key: Key) -> bool {
        self.is_key(Some(key), KeyState::Down)
    }

    fn is_any_key_down(&self) -> bool {
        self.is_key(None, KeyState::Down)
    }

    fn is_key_up(&self, key: Key) -> bool {
        self.is_key(Some(key), KeyState::Up)
    }

    fn is_any_key_up(&self) -> bool {
        self.is_key(None, KeyState::Down)
    }

    fn is_key_press(&self, key: Key) -> bool {
        self.is_key(Some(key), KeyState::Press)
    }

    fn is_any_key_press(&self) -> bool {
        self.is_key(None, KeyState::Down)
    }

    fn clear(&mut self) {
        self.keys_down = [false; KEYS_LENGTH as usize];
        self.keys_up = [false; KEYS_LENGTH as usize];
        self.keys_press = [false; KEYS_LENGTH as usize];
    }

    fn as_ai_input_mut(&mut self) -> Option<&mut dyn AiInput> {
        let ai: &mut dyn AiInput = self;
        Some(ai)
    }
}
