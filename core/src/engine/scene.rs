use crate::engine::{actor::actor::TActor, engine::Engine};

pub trait Scene {
    fn get_actors(&self) -> &[Box<dyn TActor>];
    fn init(&mut self);
    fn update(&self, delta_time: f32, engine: &Engine);
}

pub struct EmptyScene;

impl EmptyScene {
    pub fn new() -> Self {
        Self {}
    }
}

impl Scene for EmptyScene {
    fn get_actors(&self) -> &[Box<dyn TActor>] {
        todo!()
    }

    fn init(&mut self) {
        todo!()
    }

    fn update(&self, _: f32, _: &Engine) {
        todo!()
    }
}
