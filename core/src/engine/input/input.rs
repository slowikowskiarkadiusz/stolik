use crate::engine::input::key::Key;

pub trait Input {
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
