use crate::{
    engine::{
        components::world::World,
        engine::{ActorId, SCREEN_SIZE},
        hash_map::HashMap,
        scene::Scene,
        v2::V2,
    },
    scenes::{
        tetris::{
            board::{Board, create_board_actor},
            world::TetrisWorld,
        },
        utils::print_victory_text,
    },
};
extern crate alloc;
use alloc::{boxed::Box, vec::Vec};
use rand::{RngCore, SeedableRng, rngs::SmallRng};

pub struct TetrisScene {
    p1_board_actor_id: ActorId,
    p2_board_actor_id: ActorId,
    tetris_world: TetrisWorld,
}

impl Scene for TetrisScene {
    fn init(&mut self, world: &mut World) {
        let seed = SmallRng::seed_from_u64(embassy_time::Instant::now().as_micros()).next_u32();
        self.p1_board_actor_id = create_board_actor(world, &mut self.tetris_world, true, seed);
        self.p2_board_actor_id = create_board_actor(world, &mut self.tetris_world, false, seed);
    }

    fn tick(
        &mut self,
        input: &Box<dyn crate::engine::input::input::Input>,
        world: &mut crate::engine::components::world::World,
        delta_time: f32,
    ) {
        let mut damage_for_p1 = 0;
        let mut damage_for_p2 = 0;
        let mut is_p1_dead = false;
        let mut is_p2_dead = false;
        if let Some(p1_board) = self.tetris_world.get_mut_board(&self.p1_board_actor_id) {
            damage_for_p2 = p1_board.tick(input, delta_time);
            is_p1_dead = p1_board.is_dead;
        }

        // if let Some(p2_board) = self.tetris_world.get_mut_board(&self.p2_board_actor_id) {
        //     damage_for_p1 = p2_board.tick(input, delta_time);
        //     is_p2_dead = p2_board.is_dead;
        // }

        if is_p1_dead || is_p2_dead {
            self.on_players_death(world, is_p1_dead);
        }

        if let Some(p1_board) = self.tetris_world.get_mut_board(&self.p1_board_actor_id) {
            p1_board.take_damage(damage_for_p2);
        }

        // if let Some(p2_board) = self.tetris_world.get_mut_board(&self.p2_board_actor_id) {
        //     p2_board.take_damage(damage_for_p1);
        // }

        self.render(world);
    }

    fn on_overlaps(
        &mut self,
        _: &crate::engine::hash_map::HashMap<ActorId, Vec<ActorId>>,
        _: &mut crate::engine::components::world::World,
        _: f32,
    ) {
    }
}

impl TetrisScene {
    pub fn new() -> Self {
        Self {
            p1_board_actor_id: ActorId::MAX,
            p2_board_actor_id: ActorId::MAX,
            tetris_world: TetrisWorld::new(),
        }
    }

    fn on_players_death(&mut self, world: &mut World, is_p1: bool) {
        if let Some(p1_board) = self.tetris_world.get_mut_board(&self.p1_board_actor_id) {
            p1_board.stop();
            p1_board.dim(51);
        }

        if let Some(p2_board) = self.tetris_world.get_mut_board(&self.p2_board_actor_id) {
            p2_board.stop();
            p2_board.dim(51);
        }

        print_victory_text(world, if is_p1 { 1 } else { 2 });
    }

    fn render(&mut self, world: &mut World) {
        if let Some(p1_render) = world.get_mut_render(&self.p1_board_actor_id) {
            if let Some(p1_board) = self.tetris_world.get_mut_board(&self.p1_board_actor_id) {
                p1_render.write_at_origin(&p1_board.render(), &V2::zero());
            }
        }

        if let Some(p2_render) = world.get_mut_render(&self.p1_board_actor_id) {
            if let Some(p2_board) = self.tetris_world.get_mut_board(&self.p2_board_actor_id) {
                // p2_render.write_at_origin(&p2_board.render(), &V2::zero());
            }
        }
    }
}
