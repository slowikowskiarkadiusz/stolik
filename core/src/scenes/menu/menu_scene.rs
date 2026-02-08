use std::collections::HashMap;

use crate::{
    engine::{
        actor::{arrow_actor::create_arrow_actor, text::create_text_actor},
        color::Color,
        components::world::World,
        engine::{ActorId, SCREEN_SIZE, SceneFactory, open_scene},
        input::{input::Input, key::Key},
        scene::Scene,
        v2::V2,
    },
    scenes::pong::pong_scene::PongScene,
};

struct MenuOption {
    next_scene_factory: SceneFactory,
    next_scene_code_name: String,
    next_scene_print_name: String,
    text_actor_id: ActorId,
}

impl MenuOption {
    pub fn new(next_scene_factory: SceneFactory, next_scene_code_name: &str, next_scene_print_name: &str, text_actor_id: ActorId) -> Self {
        Self {
            next_scene_factory,
            next_scene_code_name: String::from(next_scene_code_name),
            next_scene_print_name: String::from(next_scene_print_name),
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
        self.options = vec![MenuOption::new(Box::new(|| Box::new(PongScene::new())), "pong_scene", "pong", 0)];

        for i in 0..self.options.len() {
            self.options[i].text_actor_id = create_text_actor(
                world,
                self.options[i].next_scene_print_name.clone(),
                V2::new(4.0, i as f32 * 6.0),
                V2::new(SCREEN_SIZE as f32 - 4.0, 5.0),
                Color::white(),
                None,
                Some("scene text"),
            );
        }

        self.cursor_actor_id = create_arrow_actor(world, V2::new(1.5, 2.5), 5, Color::white(), 0.5, Some("arrow"));
    }

    fn tick(&mut self, input: &Box<dyn Input>, world: &mut World, delta_time: f32) {
        let mut changed = false;
        if input.is_key_down(Key::P1Up) {
            self.cursor_position = self.cursor_position.saturating_sub(1);
            changed = true;
        }
        if input.is_key_down(Key::P1Down) {
            self.cursor_position = self.cursor_position.saturating_add(1);
            changed = true;
        }

        if input.is_key_down(Key::P1Green) {
            let selected = self.options.remove(self.cursor_position as usize);
            open_scene(selected.next_scene_factory);
        }

        self.cursor_position = self.cursor_position % self.options.len() as u8;

        if changed {
            // let i =0;
            // for option in &self.options{

            // }

            if let Some(cursor_blinker) = world.get_mut_blinker(&self.cursor_actor_id) {
                cursor_blinker.reset();
            }
        }
    }

    fn on_overlaps(&mut self, overlaps: &HashMap<ActorId, Vec<ActorId>>, world: &mut World, delta_time: f32) {
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
