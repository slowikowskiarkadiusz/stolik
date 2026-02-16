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

pub struct EmptyInput;

impl EmptyInput {
    pub fn new() -> Self {
        Self {}
    }
}

impl Input for EmptyInput {
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
