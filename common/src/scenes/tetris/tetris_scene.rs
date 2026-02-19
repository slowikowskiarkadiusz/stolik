use crate::engine::{
    engine::{ActorId, SCREEN_SIZE},
    scene::Scene,
    v2::V2,
};
extern crate alloc;
use alloc::{boxed::Box, vec::Vec};
use embassy_time::Instant;

pub struct TetrisScene {
    p1_board_actor_id: ActorId,
    p2_board_actor_id: ActorId,
}

impl Scene for TetrisScene {
    fn init(&mut self, world: &mut crate::engine::components::world::World) {
        let seed = Instant::now().as_millis() % 10000;
        let center = V2::one() * (SCREEN_SIZE / 2) as f32;
        let size_factor = (SCREEN_SIZE / 32) as f32;
    }

    fn tick(
        &mut self,
        input: &Box<dyn crate::engine::input::input::Input>,
        world: &mut crate::engine::components::world::World,
        delta_time: f32,
    ) {
        todo!()
    }

    fn on_overlaps(
        &mut self,
        overlaps: &crate::engine::hash_map::HashMap<ActorId, Vec<ActorId>>,
        world: &mut crate::engine::components::world::World,
        delta_time: f32,
    ) {
        todo!()
    }
}
