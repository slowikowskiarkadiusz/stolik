use crate::{
    engine::{
        color::Color,
        color_matrix::ColorMatrix,
        components::{camera::Camera, collider::CollisionResult, world::World},
        engine::ActorId,
        hash_map::HashMap,
        scene::Scene,
    },
    scenes::{
        tetris::{board::create_board_actor, world::TetrisWorld},
        utils::print_victory_text,
    },
};
extern crate alloc;
use alloc::{boxed::Box, vec::Vec};
// use esp_println::println;
use rand::{RngCore, SeedableRng, rngs::SmallRng};

pub enum TetrisSceneMode {
    Solo,
    AgainstAi,
    AgainstHuman,
}

pub struct TetrisScene {
    p1_board_actor_id: ActorId,
    p2_board_actor_id: ActorId,
    tetris_world: TetrisWorld,
    mode: TetrisSceneMode,
    is_player_dead: u8,
}

impl Scene for TetrisScene {
    fn init(&mut self, world: &mut World) {
        // println!("[Tetris] init start");
        let seed = SmallRng::seed_from_u64(embassy_time::Instant::now().as_micros()).next_u32();

        // println!("[Tetris] creating p1 board");
        self.p1_board_actor_id = create_board_actor(world, &mut self.tetris_world, true, seed);
        // println!("[Tetris] p1 board created: {}", self.p1_board_actor_id);

        // println!("[Tetris] creating p2 board");
        self.p2_board_actor_id = create_board_actor(world, &mut self.tetris_world, false, seed);
        // println!("[Tetris] p2 board created: {}", self.p2_board_actor_id);
        // println!("[Tetris] init done");
    }

    fn tick(
        &mut self,
        input: &Box<dyn crate::engine::input::input::Input>,
        world: &mut crate::engine::components::world::World,
        delta_time: f32,
    ) {
        // println!("[Tetris] tick start");

        let mut damage_for_p1 = 0;
        let mut damage_for_p2 = 0;

        // println!("[Tetris] getting p1 board");
        if let Some(p1_board) = self.tetris_world.get_mut_board(&self.p1_board_actor_id) {
            // println!("[Tetris] ticking p1");
            damage_for_p2 = p1_board.tick(input, delta_time);
            if p1_board.is_dead {
                self.is_player_dead = 1;
            }
        }

        if !matches!(self.mode, TetrisSceneMode::Solo) {
            // println!("[Tetris] getting p2 board");
            if let Some(p2_board) = self.tetris_world.get_mut_board(&self.p2_board_actor_id) {
                // println!("[Tetris] ticking p2");
                damage_for_p1 = p2_board.tick(input, delta_time);
                if p2_board.is_dead {
                    self.is_player_dead = 2;
                }
            }
        }

        // println!("[Tetris] applying damage");
        if let Some(p1_board) = self.tetris_world.get_mut_board(&self.p1_board_actor_id) {
            p1_board.take_damage(damage_for_p2);
        }

        if !matches!(self.mode, TetrisSceneMode::Solo) {
            if let Some(p2_board) = self.tetris_world.get_mut_board(&self.p2_board_actor_id) {
                p2_board.take_damage(damage_for_p1);
            }
        }
    }

    fn render(&mut self, camera: &Camera, world: &mut World, delta_time: f32) -> ColorMatrix {
        let mut result = ColorMatrix::new(camera.get_viewport_size().x as u8, camera.get_viewport_size().y as u8, Color::none());

        if camera.can_see_actor(self.p1_board_actor_id, world) {
            if let Some(p1_board) = self.tetris_world.get_board(&self.p1_board_actor_id) {
                p1_board.render_into(&mut result);
            }
        }

        // if !matches!(self.mode, TetrisSceneMode::Solo) {
        if camera.can_see_actor(self.p2_board_actor_id, world) {
            if let Some(p2_board) = self.tetris_world.get_board(&self.p2_board_actor_id) {
                p2_board.render_into(&mut result);
            }
        }

        self.on_players_death(world, camera, &mut result);

        result
    }

    fn on_overlaps(
        &mut self,
        _: &crate::engine::hash_map::HashMap<ActorId, Vec<ActorId>>,
        _: &mut crate::engine::components::world::World,
        _: f32,
    ) {
    }

    fn on_collisions(&mut self, _collisions: &HashMap<u16, Vec<(u16, CollisionResult)>>, _world: &mut World, _delta_time: f32) {}
}

impl TetrisScene {
    pub fn new(mode: TetrisSceneMode) -> Self {
        Self {
            p1_board_actor_id: ActorId::MAX,
            p2_board_actor_id: ActorId::MAX,
            tetris_world: TetrisWorld::new(),
            mode: mode,
            is_player_dead: 0,
        }
    }

    fn on_players_death(&mut self, world: &mut World, camera: &Camera, result: &mut ColorMatrix) {
        if self.is_player_dead > 0 {
            let is_p1 = self.is_player_dead == 1;

            if let Some(p1_board) = self.tetris_world.get_mut_board(&self.p1_board_actor_id) {
                p1_board.stop();
                p1_board.dim(51);
            }

            if let Some(p2_board) = self.tetris_world.get_mut_board(&self.p2_board_actor_id) {
                p2_board.stop();
                p2_board.dim(51);
            }

            print_victory_text(world, if is_p1 { 1 } else { 2 }, camera, result);
        }
    }
}
