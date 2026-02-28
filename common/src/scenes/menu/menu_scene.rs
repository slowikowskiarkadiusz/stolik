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
    scenes::{controls::controls_scene::ControlsScene, pong::pong_scene::PongScene, tetris::tetris_scene::TetrisScene},
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
        self.options = crate::my_vec![
            MenuOption::new(Box::new(|| Box::new(PongScene::new())), "pong", "pong", 0),
            MenuOption::new(Box::new(|| Box::new(TetrisScene::new())), "tetris", "tetris", 0),
            // MenuOption::new(Box::new(|| Box::new(PongScene::new())), "fong", "fong", 0),
        ];

        let stack_var = 0u8;
        let stack_ptr = &stack_var as *const u8 as usize;
        let stack_start = 0x3fcf0000usize; // przybliżony koniec SRAM dla CPU1

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
        // esp_println::println!("menu scene tick");
        if self.options.len() == 0 {
            return;
        }

        let mut changed = false;
        if input.is_key_down(Key::P1Up) {
            // esp_println::println!("P11");
            if self.cursor_position == 0 {
                self.cursor_position = self.options.len() as u8 - 1;
            } else {
                self.cursor_position -= 1;
            }
            changed = true;
            // esp_println::println!("P12");
        }
        if input.is_key_down(Key::P1Down) {
            // esp_println::println!("P21");
            if self.cursor_position == self.options.len() as u8 - 1 {
                self.cursor_position = 0;
            } else {
                self.cursor_position += 1;
            }
            changed = true;
            // esp_println::println!("P22");
        }

        // esp_println::println!("0");
        if let Some(cursor_transform) = world.get_mut_transform(&self.cursor_actor_id) {
            cursor_transform.center = V2::new(1.5, 2.5 + self.cursor_position as f32 * 6.0);
        }
        // esp_println::println!("1");

        // if input.is_key_down(Key::P1Down) {
        // esp_println::println!("P1");
        // }

        // if input.is_key_down(Key::P2Down) {
        // esp_println::println!("P2");
        // }

        if input.is_key_down(Key::Start) {
            // esp_println::println!("Key::Start");
            let selected = self.options.remove(self.cursor_position as usize);
            let name = selected.next_scene_code_name;
            open_scene(Box::new(|| Box::new(ControlsScene::new(name, selected.next_scene_factory))));
        }
        // esp_println::println!("2");

        if changed {
            // esp_println::println!("3");
            if let Some(cursor_blinker) = world.get_mut_blinker(&self.cursor_actor_id) {
                // esp_println::println!("4");
                cursor_blinker.reset();
                // esp_println::println!("5");
            }
            // esp_println::println!("6");
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
            options: Vec::new(),
        }
    }
}
