use std::collections::HashMap;

use crate::engine::{components::world::World, engine::ActorId, input::input::Input};

pub trait Scene {
    fn init(&mut self, world: &mut World);
    fn tick(&mut self, input: &Box<dyn Input>, world: &mut World, delta_time: f32);
    fn on_overlaps(&mut self, overlaps: &HashMap<ActorId, Vec<ActorId>>, world: &mut World, delta_time: f32);
}

pub struct EmptyScene;

impl EmptyScene {
    pub fn new() -> Self {
        Self {}
    }
}

impl Scene for EmptyScene {
    fn init(&mut self, _: &mut World) {
        todo!()
    }

    fn tick(&mut self, _: &Box<dyn Input>, _: &mut World, _: f32) {
        todo!()
    }

    fn on_overlaps(&mut self, _: &HashMap<ActorId, Vec<ActorId>>, _: &mut World, _: f32) {
        todo!()
    }
}
