extern crate alloc;
use alloc::{boxed::Box, string::String, vec, vec::Vec};

use crate::{
    engine::{
        actor::{arrow_actor::render_arrow, text::render_text},
        color::Color,
        color_matrix::ColorMatrix,
        components::{camera::Camera, collider::CollisionResult, world::World},
        engine::{ActorId, SCREEN_SIZE, SceneFactory, open_scene},
        hash_map::HashMap,
        input::{input::Input, key::Key},
        scene::Scene,
        v2::V2,
    },
    scenes::{
        astro_duel::astro_scene::AstroDuelScene,
        controls::controls_scene::ControlsScene,
        mario::mario_scene::MarioScene,
        pong::pong_scene::PongScene,
        tanks::tanks_scene::TanksScene,
        tetris::tetris_scene::{TetrisScene, TetrisSceneMode},
    },
};

struct MenuOption {
    next_scene_factory: SceneFactory,
    next_scene_code_name: &'static str,
    next_scene_print_name: &'static str,
}

impl MenuOption {
    pub fn new(next_scene_factory: SceneFactory, next_scene_code_name: &'static str, next_scene_print_name: &'static str) -> Self {
        Self {
            next_scene_factory,
            next_scene_code_name: next_scene_code_name,
            next_scene_print_name: next_scene_print_name,
        }
    }
}

pub struct MenuScene {
    cursor_position: u8,
    options: Vec<MenuOption>,
}

impl Scene for MenuScene {
    fn init(&mut self, _world: &mut World) {
        self.options = vec![
            MenuOption::new(Box::new(|| Box::new(PongScene::new(false))), "pong", "pong"),
            MenuOption::new(Box::new(|| Box::new(PongScene::new(true))), "pong", "pong -- vs ai"),
            MenuOption::new(
                Box::new(|| Box::new(TetrisScene::new(TetrisSceneMode::AgainstHuman))),
                "tetris",
                "tetris",
            ),
            MenuOption::new(
                Box::new(|| Box::new(TetrisScene::new(TetrisSceneMode::Solo))),
                "tetris",
                "tetris -- solo",
            ),
            MenuOption::new(Box::new(|| Box::new(MarioScene::new())), "mario", "mario"),
            MenuOption::new(Box::new(|| Box::new(TanksScene::new())), "tanks", "tanks"),
            MenuOption::new(Box::new(|| Box::new(AstroDuelScene::new())), "astro_duel", "astro duel"),
        ];
    }

    fn tick(&mut self, input: &Box<dyn Input>, _world: &mut World, _delta_time: f32) {
        if self.options.len() == 0 {
            return;
        }

        if input.is_key_down(Key::P1Up) {
            if self.cursor_position == 0 {
                self.cursor_position = self.options.len() as u8 - 1;
            } else {
                self.cursor_position -= 1;
            }
        }
        if input.is_key_down(Key::P1Down) {
            if self.cursor_position == self.options.len() as u8 - 1 {
                self.cursor_position = 0;
            } else {
                self.cursor_position += 1;
            }
        }

        if input.is_key_down(Key::Start) {
            let selected = self.options.remove(self.cursor_position as usize);
            let name = selected.next_scene_code_name;
            // open_scene(selected.next_scene_factory);
            open_scene(Box::new(|| Box::new(ControlsScene::new(name, selected.next_scene_factory))));

            // let factory = core::mem::replace(&mut self.next_scene, Box::new(|| Box::new(MenuScene::new())));
            // open_scene(factory);
        }

        //todo move to render
        // if changed {
        //     if let Some(cursor_blinker) = world.get_mut_blinker(&self.cursor_actor_id) {
        //         cursor_blinker.reset();
        //     }
        // }
    }

    fn render(&mut self, camera: &Camera, world: &mut World, _delta_time: f32) -> ColorMatrix {
        let vsize = camera.get_viewport().get_size();
        let mut result = ColorMatrix::new(vsize.x as u8, vsize.y as u8, Color::none());

        for i in 0..self.options.len() {
            render_text(
                self.options[i].next_scene_print_name,
                V2::new(4.0, i as f32 * 6.0),
                V2::new(SCREEN_SIZE as f32 - 4.0, 5.0),
                None,
                None,
                Color::white(),
                Some(camera),
                &mut result,
            );
        }

        render_arrow(
            world,
            V2::new(1.5, 2.5 + self.cursor_position as f32 * 6.0),
            5,
            Color::white(),
            500,
            camera,
            &mut result,
        );

        result
    }

    fn on_overlaps(&mut self, _overlaps: &HashMap<ActorId, Vec<ActorId>>, _world: &mut World, _delta_time: f32) {}

    fn on_collisions(&mut self, _collisions: &HashMap<u16, Vec<(u16, CollisionResult)>>, _world: &mut World, _delta_time: f32) {}
}

impl MenuScene {
    pub fn new() -> Self {
        Self {
            cursor_position: 0,
            options: vec![],
        }
    }
}
