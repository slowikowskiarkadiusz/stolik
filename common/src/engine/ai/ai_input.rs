extern crate alloc;
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::Mutex;
use crate::engine::input::{
    gesture::Gestures,
    input::Input,
    key::{KEYS_LENGTH, Key, KeyState},
};

pub struct AiInput {
    gestures: Gestures,
    held: Arc<Mutex<[bool; KEYS_LENGTH as usize]>>,
    keys_down: [bool; KEYS_LENGTH as usize],
    keys_press: [bool; KEYS_LENGTH as usize],
    keys_up: [bool; KEYS_LENGTH as usize],
}

impl AiInput {
    pub fn new(held: Arc<Mutex<[bool; KEYS_LENGTH as usize]>>) -> Self {
        Self {
            gestures: Gestures::new(),
            held,
            keys_down: [false; KEYS_LENGTH as usize],
            keys_press: [false; KEYS_LENGTH as usize],
            keys_up: [false; KEYS_LENGTH as usize],
        }
    }

    fn is_key(&self, key: Option<Key>, key_state: KeyState) -> bool {
        fn func(key: Option<Key>, arr: &[bool]) -> bool {
            if let Some(k) = key {
                return AiInput::map_key(k).iter().any(|kk: &Key| arr[*kk as usize]);
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

impl Input for AiInput {
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
}
