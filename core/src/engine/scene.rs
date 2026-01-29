use crate::engine::{components::world::World, input::input::Input};

pub trait Scene {
    fn init(&mut self, world: &mut World);
    fn tick(&mut self, input: &Box<dyn Input>, world: &mut World, delta_time: f32);
}

pub struct EmptyScene;

impl EmptyScene {
    pub fn new() -> Self {
        Self {}
    }
}

impl Scene for EmptyScene {
    fn init(&mut self, world: &mut World) {
        todo!()
    }

    fn tick(&mut self, _input: &Box<dyn Input>, _world: &mut World, _delta_time: f32) {
        todo!()
    }
}
