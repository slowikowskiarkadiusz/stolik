use std::println;

use crate::{
    engine::{
        self,
        ai::neat_genome::DataForAi,
        color::Color,
        color_matrix::ColorMatrix,
        components::{
            camera::Camera,
            collider::CollisionResult,
            world::{self, World},
        },
        engine::ActorId,
        hash_map::HashMap,
        input::{input::Input, key::KEYS_LENGTH},
        scene::Scene,
        v2::V2,
    },
    scenes::{
        tetris::{board::create_board_actor, world::TetrisWorld},
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
}

impl Scene for TetrisScene {
    fn init(&mut self, world: &mut World) {
        // println!("Opening Tetris!");
        // println!("[Tetris] init start");
        let seed = SmallRng::seed_from_u64(embassy_time::Instant::now().as_micros()).next_u32();

        // println!("[Tetris] creating p1 board");
        self.p1_board_actor_id = create_board_actor(world, &mut self.tetris_world, true, seed);
        // println!("[Tetris] p1 board created: {}", self.p1_board_actor_id);

        if !matches!(self.mode, TetrisSceneMode::Solo) {
            // println!("[Tetris] creating p2 board");
            self.p2_board_actor_id = create_board_actor(world, &mut self.tetris_world, false, seed);
            // println!("[Tetris] p2 board created: {}", self.p2_board_actor_id);
        }
        // println!("[Tetris] init done");
    }

    fn tick(&mut self, inputs: [&Box<dyn Input>; 2], _world: &mut World, delta_time: f32) {
        let mut damage_for_p1 = 0;
        let mut damage_for_p2 = 0;

        // println!("[Tetris] getting p1 board");
        if let Some(p1_board) = self.tetris_world.get_mut_board(&self.p1_board_actor_id) {
            // println!("[Tetris] ticking p1");
            damage_for_p2 = p1_board.tick(inputs[0], delta_time);
            if p1_board.is_dead {
                self.is_player_dead = 1;
            }
        }

        if !matches!(self.mode, TetrisSceneMode::Solo) {
            // println!("[Tetris] getting p2 board");
            if let Some(p2_board) = self.tetris_world.get_mut_board(&self.p2_board_actor_id) {
                // println!("[Tetris] ticking p2");
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
        // println!("[Tetris] render start");

        let mut result = ColorMatrix::new(
            camera.get_viewport().get_size().x as u8,
            camera.get_viewport().get_size().y as u8,
            Color::none(),
        );
        // println!("[Tetris] render 0");

        // if !matches!(self.mode, TetrisSceneMode::Solo) {
        if let Some(p2_board) = self.tetris_world.get_board(&self.p2_board_actor_id) {
            p2_board.render_into(V2::new(18.0, 5.0), &mut result);
            result.flip();
        }
        // }

        if let Some(p1_board) = self.tetris_world.get_board(&self.p1_board_actor_id) {
            p1_board.render_into(V2::new(0.0, 5.0), &mut result);
        }
        // println!("[Tetris] render 1");

        // println!("[Tetris] render 2");

        self.on_players_death(world, camera, &mut result);
        // println!("[Tetris] render 3");

        result
    }

    fn on_overlaps(&mut self, _: &engine::hash_map::HashMap<ActorId, Vec<ActorId>>, _: &mut World, _: f32) {}

    fn on_collisions(&mut self, _collisions: &HashMap<u16, Vec<(u16, CollisionResult)>>, _world: &mut World, _delta_time: f32) {}

    fn get_data_for_ai(&self) -> DataForAi {
        let p1_board = self.tetris_world.get_board(&self.p1_board_actor_id);
        let p2_board = self.tetris_world.get_board(&self.p2_board_actor_id);
        DataForAi {
            inputs: [
                p1_board.map(|b| b.get_data_for_ai()).unwrap_or_default(),
                p2_board.map(|b| b.get_data_for_ai()).unwrap_or_default(),
            ],
            points: [
                p1_board.map(|b| b.points).unwrap_or(0.0),
                p2_board.map(|b| b.points).unwrap_or(0.0),
            ],
            is_gameover: self.is_player_dead > 0,
            outputs_to_keys: tetris_outputs_to_keys,
        }
    }

    fn is_game_over(&self) -> bool {
        self.is_player_dead > 0
    }
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

fn tetris_outputs_to_keys(outputs: &[f64]) -> [bool; KEYS_LENGTH as usize] {
    let mut keys = [false; KEYS_LENGTH as usize];
    if outputs.len() >= 5 {
        keys[2] = outputs[0] < 0.4; // Left
        keys[3] = outputs[0] > 0.6; // Right
        keys[1] = outputs[1] > 0.5; // Up (hard drop)
        keys[0] = outputs[2] > 0.5; // Down (fast drop)
        keys[5] = outputs[3] > 0.5; // Blue (rotate right)
        keys[6] = outputs[4] > 0.5; // Green (rotate left)
    }
    keys
}
