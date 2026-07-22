// TODO HashMap

extern crate alloc;
use alloc::{boxed::Box, vec::Vec};

use crate::engine::{
    color_matrix::ColorMatrix,
    components::{camera::Camera, collider::CollisionResult, world::World},
    engine::ActorId,
    hash_map::HashMap,
    input::input::Input,
};

pub trait Scene {
    fn init(&mut self, world: &mut World);
    fn tick(&mut self, input: &Box<dyn Input>, world: &mut World, delta_time: f32);
    fn render(&mut self, camera: &Camera, world: &mut World, delta_time: f32) -> ColorMatrix;
    fn on_overlaps(&mut self, overlaps: &HashMap<ActorId, Vec<ActorId>>, world: &mut World, delta_time: f32);
    fn on_collisions(&mut self, collisions: &HashMap<u16, Vec<(u16, CollisionResult)>>, world: &mut World, delta_time: f32);
    fn get_ai_inputs(&self) -> [Vec<f64>; 2];
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

    fn render(&mut self, _: &Camera, _: &mut World, _: f32) -> ColorMatrix {
        todo!()
    }

    fn on_overlaps(&mut self, _: &HashMap<ActorId, Vec<ActorId>>, _: &mut World, _: f32) {
        todo!()
    }

    fn on_collisions(&mut self, _collisions: &HashMap<u16, Vec<(u16, CollisionResult)>>, _world: &mut World, _delta_time: f32) {
        todo!()
    }

    fn get_ai_inputs(&self) -> [Vec<f64>; 2] {
        todo!()
    }
}
