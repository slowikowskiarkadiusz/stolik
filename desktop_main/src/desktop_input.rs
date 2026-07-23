use common::engine::input::{
    gesture::Gestures,
    input::Input,
    key::{KEYS_LENGTH, Key, KeyState},
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

// thread_local! {
//     static KEY_MAP: RefCell<HashMap<minifb::Key, Key>> = RefCell::new(HashMap::new());
// }

pub struct DesktopInput {
    player: u8,
    gestures: Gestures,
    input_state: Arc<Mutex<HashMap<minifb::Key, (bool, bool)>>>,
    keys_down: [bool; KEYS_LENGTH as usize],
    keys_up: [bool; KEYS_LENGTH as usize],
    keys_press: [bool; KEYS_LENGTH as usize],
    key_map: HashMap<minifb::Key, Key>,
}

impl DesktopInput {
    pub fn new(player: u8, input_state: Arc<Mutex<HashMap<minifb::Key, (bool, bool)>>>) -> Self {
        let mut key_map: HashMap<minifb::Key, Key> = HashMap::new();
        if player == 0 {
            key_map.insert(minifb::Key::Space, Key::Start);
            key_map.insert(minifb::Key::S, Key::Down);
            key_map.insert(minifb::Key::W, Key::Up);
            key_map.insert(minifb::Key::A, Key::Left);
            key_map.insert(minifb::Key::D, Key::Right);
            key_map.insert(minifb::Key::F, Key::Blue);
            key_map.insert(minifb::Key::G, Key::Green);
        }

        if player == 1 {
            key_map.insert(minifb::Key::Space, Key::Start);
            key_map.insert(minifb::Key::Down, Key::Up);
            key_map.insert(minifb::Key::Up, Key::Down);
            key_map.insert(minifb::Key::Left, Key::Left);
            key_map.insert(minifb::Key::Right, Key::Right);
            key_map.insert(minifb::Key::O, Key::Blue);
            key_map.insert(minifb::Key::P, Key::Green);
        }

        Self {
            player: player,
            gestures: Gestures::new(),
            input_state: input_state.clone(),
            keys_down: [false; KEYS_LENGTH as usize],
            keys_up: [false; KEYS_LENGTH as usize],
            keys_press: [false; KEYS_LENGTH as usize],
            key_map,
        }
    }

    pub fn on_key_pressed(&mut self, key: &minifb::Key) {
        if let Some(mapped) = self.key_map.get(&key) {
            let m = mapped.clone();
            self.keys_down[m as usize] = true;
            self.keys_press[m as usize] = true;
        }
    }

    pub fn on_key_released(&mut self, key: &minifb::Key) {
        if let Some(mapped) = self.key_map.get(&key) {
            let m = mapped.clone();
            self.keys_up[m as usize] = true;
            self.keys_press[m as usize] = false;
        }
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
            Key::Down | Key::Up | Key::Left | Key::Right | Key::Blue | Key::Green | Key::Start => vec![key],
            Key::AnyDirection => vec![Key::Up, Key::Down, Key::Left, Key::Right],
            Key::Any => vec![Key::Up, Key::Down, Key::Left, Key::Right, Key::Blue, Key::Green],
        }
    }
}

impl Input for DesktopInput {
    fn gestures(&self) -> &Gestures {
        &self.gestures
    }

    fn update(&mut self, delta_time: f32) {
        let snapshot = {
            let mut guard = self.input_state.lock().unwrap();

            let snapshot = guard.clone();

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

        self.gestures.tick(self.get_snapshot(), delta_time);
    }

    fn late_update(&mut self, _: f32) {
        let mut guard = self.input_state.lock().unwrap();
        for v in guard.values_mut() {
            *v = (false, false);
        }
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
