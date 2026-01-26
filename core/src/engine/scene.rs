use crate::engine::actor::actor::{TActor};

pub trait Scene {
    fn get_actors(&self) -> &[Box<dyn TActor>];
    fn init(&mut self);
    fn update(&self, delta_time: f32);
}
