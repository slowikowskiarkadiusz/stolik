extern crate alloc;
use alloc::{boxed::Box, string::String, vec, vec::Vec};

use crate::{
    engine::{
        actor::{arrow_actor::create_arrow_actor, text::create_text_actor},
        color::Color,
        components::world::World,
        engine::{ActorId, SCREEN_SIZE, SceneFactory, open_scene},
        hash_map::HashMap,
        input::{input::Input, key::Key},
        scene::Scene,
        v2::V2,
    },
    scenes::{pong::pong_scene::PongScene, tetris::tetris_scene::{TetrisScene, TetrisSceneMode}},
};

struct MenuOption {
    next_scene_factory: SceneFactory,
    next_scene_code_name: &'static str,
    next_scene_print_name: &'static str,
    text_actor_id: ActorId,
}

impl MenuOption {
    pub fn new(
        next_scene_factory: SceneFactory,
        next_scene_code_name: &'static str,
        next_scene_print_name: &'static str,
        text_actor_id: ActorId,
    ) -> Self {
        Self {
            next_scene_factory,
            next_scene_code_name: next_scene_code_name,
            next_scene_print_name: next_scene_print_name,
            text_actor_id: text_actor_id,
        }
    }
}

pub struct MenuScene {
    cursor_position: u8,
    cursor_actor_id: ActorId,
    options: Vec<MenuOption>,
}

impl Scene for MenuScene {
    fn init(&mut self, world: &mut World) {
        self.options = vec![
            MenuOption::new(Box::new(|| Box::new(PongScene::new(false))), "pong", "pong", 0),
            MenuOption::new(Box::new(|| Box::new(PongScene::new(true))), "pong", "pong -- vs ai", 0),
            MenuOption::new(Box::new(|| Box::new(TetrisScene::new(TetrisSceneMode::AgainstHuman))), "tetris", "tetris", 0),
            MenuOption::new(Box::new(|| Box::new(TetrisScene::new(TetrisSceneMode::Solo))), "tetris", "tetris -- solo", 0),
            // MenuOption::new(Box::new(|| Box::new(TetrisScene::new(TetrisSceneMode::AgainstAi))), "tetris", "tetris -- vs ai", 0),
            // MenuOption::new(Box::new(|| Box::new(PongScene::new())), "fong", "fong", 0),
        ];

        for i in 0..self.options.len() {
            self.options[i].text_actor_id = create_text_actor(
                world,
                String::from(self.options[i].next_scene_print_name),
                V2::new(4.0, i as f32 * 6.0),
                V2::new(SCREEN_SIZE as f32 - 4.0, 5.0),
                Color::white(),
                None,
                Some("scene text"),
            );
        }

        self.cursor_actor_id = create_arrow_actor(world, V2::new(1.5, 2.5), 5, Color::white(), 500, Some("arrow"));
    }

    fn tick(&mut self, input: &Box<dyn Input>, world: &mut World, _delta_time: f32) {
        if self.options.len() == 0 {
            return;
        }

        let mut changed = false;
        if input.is_key_down(Key::P1Up) {
            if self.cursor_position == 0 {
                self.cursor_position = self.options.len() as u8 - 1;
            } else {
                self.cursor_position -= 1;
            }
            changed = true;
        }
        if input.is_key_down(Key::P1Down) {
            if self.cursor_position == self.options.len() as u8 - 1 {
                self.cursor_position = 0;
            } else {
                self.cursor_position += 1;
            }
            changed = true;
        }

        if let Some(cursor_transform) = world.get_mut_transform(&self.cursor_actor_id) {
            cursor_transform.center = V2::new(1.5, 2.5 + self.cursor_position as f32 * 6.0);
        }

        if input.is_key_down(Key::Start) || input.is_key_down(Key::P1Blue) || input.is_key_down(Key::P1Green) {
            let selected = self.options.remove(self.cursor_position as usize);
            let _name = selected.next_scene_code_name;
            open_scene(selected.next_scene_factory);
            // let factory = core::mem::replace(&mut self.next_scene, Box::new(|| Box::new(MenuScene::new())));
            // open_scene(factory);
        }

        if changed {
            if let Some(cursor_blinker) = world.get_mut_blinker(&self.cursor_actor_id) {
                cursor_blinker.reset();
            }
        }
    }

    fn on_overlaps(&mut self, _overlaps: &HashMap<ActorId, Vec<ActorId>>, _world: &mut World, _delta_time: f32) {
        // todo!()
    }
}

impl MenuScene {
    pub fn new() -> Self {
        Self {
            cursor_position: 0,
            cursor_actor_id: 0,
            options: vec![],
        }
    }
}
