#[cfg(feature = "esp")]
use esp_println::println;

use crate::{
    engine::{
        self, ai::ai_input::AiInput, color::Color, color_matrix::ColorMatrix, components::{camera::Camera, collider::CollisionResult, world::World}, engine::{ActorId, set_input}, hash_map::HashMap, input::{input::Input, key::Key}, scene::Scene, v2::V2,
    }, scenes::{
        tetris::{
            board::create_board_actor,
            tetris_ai_input::{TetrisAiInput, set_tetris_ai_data, set_tetris_piece_center_x},
            world::TetrisWorld,
        },
        utils::print_victory_text,
    },
};
extern crate alloc;
use alloc::{boxed::Box, vec::Vec};
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
    use_ai: bool,
}

impl Scene for TetrisScene {
    fn init(&mut self, world: &mut World) {
        if self.use_ai {
            set_input(1, TetrisAiInput::new(1));
        }
        let seed = SmallRng::seed_from_u64(embassy_time::Instant::now().as_micros()).next_u32();

        self.p1_board_actor_id = create_board_actor(world, &mut self.tetris_world, true, seed);

        if !matches!(self.mode, TetrisSceneMode::Solo) {
            self.p2_board_actor_id = create_board_actor(world, &mut self.tetris_world, false, seed);
        }
    }

    fn tick(&mut self, inputs: [&Box<dyn Input>; 2], _world: &mut World, delta_time: f32) {
        let mut damage_for_p1 = 0;
        let mut damage_for_p2 = 0;

        if let Some(p1_board) = self.tetris_world.get_mut_board(&self.p1_board_actor_id) {
            damage_for_p2 = p1_board.tick(inputs[0], delta_time);
            if p1_board.is_dead {
                self.is_player_dead = 1;
            }
        }

        if !matches!(self.mode, TetrisSceneMode::Solo) {
            if let Some(p2_board) = self.tetris_world.get_mut_board(&self.p2_board_actor_id) {
                damage_for_p1 = p2_board.tick(inputs[1], delta_time);
                if p2_board.is_dead {
                    self.is_player_dead = 2;
                }
            }
        }

        if let Some(p1_board) = self.tetris_world.get_mut_board(&self.p1_board_actor_id) {
            p1_board.take_damage(damage_for_p1);
        }

        if !matches!(self.mode, TetrisSceneMode::Solo) {
            if let Some(p2_board) = self.tetris_world.get_mut_board(&self.p2_board_actor_id) {
                p2_board.take_damage(damage_for_p2);
            }
        }
    }

    fn render(&mut self, camera: &Camera, world: &mut World, _delta_time: f32) -> ColorMatrix {
        world.get_mut_camera().set_viewport((V2::zero(), V2::one() * 32.0));

        let mut result = ColorMatrix::new(
            camera.get_viewport().get_size().x as u8,
            camera.get_viewport().get_size().y as u8,
            Color::none(),
        );

        if let Some(p2_board) = self.tetris_world.get_board(&self.p2_board_actor_id) {
            p2_board.render_into(V2::new(18.0, 5.0), &mut result);
            result.flip();
        }

        if let Some(p1_board) = self.tetris_world.get_board(&self.p1_board_actor_id) {
            p1_board.render_into(V2::new(0.0, 5.0), &mut result);
        }

        self.on_players_death(world, camera, &mut result);

        result
    }

    fn on_overlaps(&mut self, _: &engine::hash_map::HashMap<ActorId, Vec<ActorId>>, _: &mut World, _: f32) {}

    fn on_collisions(&mut self, _collisions: &HashMap<u16, Vec<(u16, CollisionResult)>>, _world: &mut World, _delta_time: f32) {}

    fn is_game_over(&self) -> bool {
        self.is_player_dead > 0
    }
}

impl TetrisScene {
    fn on_drop(&self, is_p1: bool) {}

    pub fn new(mode: TetrisSceneMode, use_ai: bool) -> Self {
        Self {
            p1_board_actor_id: ActorId::MAX,
            p2_board_actor_id: ActorId::MAX,
            tetris_world: TetrisWorld::new(),
            mode: mode,
            is_player_dead: 0,
            use_ai,
        }
    }

    fn on_players_death(&mut self, _world: &mut World, camera: &Camera, result: &mut ColorMatrix) {
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

            print_victory_text(result, if is_p1 { 1 } else { 2 }, camera, true);
        }
    }
}
