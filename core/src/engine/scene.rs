use crate::engine::{actor::actor::TActor, engine::EngineView};

pub trait Scene {
    fn get_actors(&self) -> &[Box<dyn TActor>];
    fn init(&mut self);
    fn update(&self, engine: &EngineView);
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

    fn update(&self, _: &EngineView) {
        todo!()
    }
}
