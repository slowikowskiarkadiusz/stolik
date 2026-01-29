use core::engine::input::{
    input::Input,
    key::{KEYS_LENGTH, Key, KeyState},
};
use std::{
    cell::RefCell,
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::InputState;

thread_local! {
    static KEY_MAP: RefCell<HashMap<minifb::Key, Key>> = RefCell::new(HashMap::new());
}

pub struct DesktopInput {
    input_state: Arc<Mutex<InputState>>,
    keys_down: [bool; KEYS_LENGTH as usize],
    keys_up: [bool; KEYS_LENGTH as usize],
    keys_press: [bool; KEYS_LENGTH as usize],
}

impl DesktopInput {
    pub fn new(input_state: Arc<Mutex<InputState>>) -> Self {
        KEY_MAP.with(|f| {
            f.borrow_mut().insert(minifb::Key::Space, Key::Start);
            f.borrow_mut().insert(minifb::Key::S, Key::P1Down);
            f.borrow_mut().insert(minifb::Key::W, Key::P1Up);
            f.borrow_mut().insert(minifb::Key::A, Key::P1Left);
            f.borrow_mut().insert(minifb::Key::D, Key::P1Right);
            f.borrow_mut().insert(minifb::Key::F, Key::P1Blue);
            f.borrow_mut().insert(minifb::Key::G, Key::P1Green);
            f.borrow_mut().insert(minifb::Key::Down, Key::P2Down);
            f.borrow_mut().insert(minifb::Key::Up, Key::P2Up);
            f.borrow_mut().insert(minifb::Key::Left, Key::P2Left);
            f.borrow_mut().insert(minifb::Key::Right, Key::P2Right);
            f.borrow_mut().insert(minifb::Key::O, Key::P2Blue);
            f.borrow_mut().insert(minifb::Key::P, Key::P2Green);
        });

        Self {
            input_state: input_state.clone(),
            keys_down: [false; KEYS_LENGTH as usize],
            keys_up: [false; KEYS_LENGTH as usize],
            keys_press: [false; KEYS_LENGTH as usize],
        }
    }

    pub fn on_key_pressed(&mut self, key: &minifb::Key) {
        KEY_MAP.with(|f| {
            if let Some(mapped) = f.borrow().get(&key) {
                let m = mapped.clone();
                self.keys_down[m as usize] = true;
                self.keys_press[m as usize] = true;
            }
        });
    }

    pub fn on_key_released(&mut self, key: &minifb::Key) {
        KEY_MAP.with(|f| {
            if let Some(mapped) = f.borrow().get(&key) {
                let m = mapped.clone();
                self.keys_up[m as usize] = true;
                self.keys_press[m as usize] = false;
            }
        });
    }

    fn is_key(&self, key: Option<Key>, key_state: KeyState) -> bool {
        fn func(key: Option<Key>, arr: &[bool]) -> bool {
            if let Some(k) = key {
                return DesktopInput::map_key(k).iter().any(|kk| arr[kk.clone() as usize]);
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
            Key::P1Down
            | Key::P1Up
            | Key::P1Left
            | Key::P1Right
            | Key::P1Blue
            | Key::P1Green
            | Key::P2Down
            | Key::P2Up
            | Key::P2Left
            | Key::P2Right
            | Key::P2Blue
            | Key::P2Green
            | Key::Start => vec![key],
            Key::P1AnyDirection => vec![Key::P1Up, Key::P1Down, Key::P1Left, Key::P1Right],
            Key::P1Any => vec![Key::P1Up, Key::P1Down, Key::P1Left, Key::P1Right, Key::P1Blue, Key::P1Green],
            Key::P2AnyDirection => vec![Key::P2Up, Key::P2Down, Key::P2Left, Key::P2Right],
            Key::P2Any => vec![Key::P2Up, Key::P2Down, Key::P2Left, Key::P2Right, Key::P2Blue, Key::P2Green],
        }
    }
}

impl Input for DesktopInput {
    fn update(&mut self, _: f32) {
        let snapshot = {
            let mut guard = self.input_state.lock().unwrap();

            let snapshot = guard.clone();
            for v in guard.values_mut() {
                *v = (false, false);
            }

            snapshot
        };

        for (k, v) in snapshot.iter() {
            if v.0 {
                self.on_key_pressed(k);
            }
            if v.1 {
                self.on_key_released(k);
            }
        }
    }

    fn late_update(&mut self, _: f32) {
        self.keys_down = [false; KEYS_LENGTH as usize];
        self.keys_up = [false; KEYS_LENGTH as usize];
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
