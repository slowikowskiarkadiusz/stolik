use crate::engine::{
    hash_map::HashMap,
    input::{
        gesture::Gestures,
        key::{KEYS_LENGTH, Key, u8_to_key},
    },
};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct KeyState {
    pub is_down: bool,
    pub is_press: bool,
    pub is_up: bool,
}
pub type InputSnapshot = HashMap<Key, KeyState>;

pub trait Input {
    fn get_snapshot(&self) -> InputSnapshot {
        let mut result = InputSnapshot::new();

        for k in 0..KEYS_LENGTH {
            let key = u8_to_key(k);
            result.insert(
                key,
                KeyState {
                    is_down: self.is_key_down(key),
                    is_press: self.is_key_press(key),
                    is_up: self.is_key_up(key),
                },
            );
        }

        result
    }

    fn gestures(&self) -> &Gestures;
    fn update(&mut self, delta_time: f32);
    fn late_update(&mut self, delta_time: f32);
    fn is_key_down(&self, key: Key) -> bool;
    fn is_any_key_down(&self) -> bool;
    fn is_key_up(&self, key: Key) -> bool;
    fn is_any_key_up(&self) -> bool;
    fn is_key_press(&self, key: Key) -> bool;
    fn is_any_key_press(&self) -> bool;
    fn clear(&mut self);
}

pub struct EmptyInput;

impl EmptyInput {
    pub fn new() -> Self {
        Self {}
    }
}

impl Input for EmptyInput {
    fn gestures(&self) -> &Gestures {
        todo!()
    }

    fn update(&mut self, _: f32) {
        todo!()
    }

    fn late_update(&mut self, _: f32) {
        todo!()
    }

    fn is_key_down(&self, _: Key) -> bool {
        todo!()
    }

    fn is_any_key_down(&self) -> bool {
        todo!()
    }

    fn is_key_up(&self, _: Key) -> bool {
        todo!()
    }

    fn is_any_key_up(&self) -> bool {
        todo!()
    }

    fn is_key_press(&self, _: Key) -> bool {
        todo!()
    }

    fn is_any_key_press(&self) -> bool {
        todo!()
    }

    fn clear(&mut self) {
        todo!()
    }
}
