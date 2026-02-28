extern crate alloc;
use alloc::{boxed::Box, string::String, sync::Arc, vec, vec::Vec};
use embassy_sync::lazy_lock::LazyLock;
use spin::Mutex;

use crate::{
    engine::{
        actor::{arrow_actor::create_arrow_actor, rectangle_actor::create_rectangle_actor, text::create_text_actor}, asyncable::{AsyncableType, add_asyncable}, color::Color, components::world::World, engine::{ActorId, SCREEN_SIZE, SceneFactory, open_scene}, hash_map::HashMap, input::{input::Input, key::Key}, scene::Scene, v2::V2
    },
    scenes::{controls::button_icon_actor::create_button_icon_actor, menu::menu_scene::MenuScene},
};

static BUTTON_SIZE: u8 = 5;

#[derive(Clone)]
struct ControlsData {
    keys: Vec<Key>,
    text: String,
    operation: Option<String>,
}

impl ControlsData {
    pub fn new(keys: Vec<Key>, text: &str, operation: Option<&str>) -> Self {
        Self {
            keys,
            text: String::from(text),
            operation: if let Some(operation_string) = operation {
                Some(String::from(operation_string))
            } else {
                None
            },
        }
    }
}

pub struct ControlsScene {
    can_proceed: Arc<Mutex<bool>>,
    divider_actor_id: ActorId,
    pages: Vec<Vec<ControlsData>>,
    current_page_actors: Vec<ActorId>,
    current_page_index: u8,
    next_scene: SceneFactory,
    lines_per_page: u8,
    print_page_timer_seconds: f32,
}

impl Scene for ControlsScene {
    fn init(&mut self, world: &mut crate::engine::components::world::World) {
        self.divider_actor_id = create_rectangle_actor(
            world,
            V2::one() * (SCREEN_SIZE / 2) as f32,
            V2::new(SCREEN_SIZE as f32, 2.0),
            Color::white().a(127).clone(),
            None,
            Some("divider"),
        );

        let can_proceed_arc = self.can_proceed.clone();

        add_asyncable(
            Box::new(move |world, _| {
                *can_proceed_arc.lock() = true;
                ControlsScene::create_arrow(world, true);
                ControlsScene::create_arrow(world, false);
            }),
            2.0,
            AsyncableType::Timeout,
        );

        self.print_page(world);
    }

    fn tick(&mut self, input: &Box<dyn Input>, world: &mut World, delta_time: f32) {
        self.print_page_timer_seconds += delta_time;

        if self.print_page_timer_seconds > 1.6 {
            self.print_page_timer_seconds = 0.0;
            self.current_page_index = (self.current_page_index + 1) % self.pages.len() as u8;
            self.print_page(world);
        }

        if self.can_proceed.lock().clone() && input.is_any_key_down() {
            let factory = core::mem::replace(&mut self.next_scene, Box::new(|| Box::new(MenuScene::new())));
            open_scene(factory);
        }
    }

    fn on_overlaps(&mut self, _: &HashMap<ActorId, Vec<ActorId>>, _: &mut World, _: f32) {}
}

impl ControlsScene {
    pub fn new(next_scene_name: &str, next_scene: SceneFactory) -> Self {
        let lines_per_page = (SCREEN_SIZE / 2 - 5) / (BUTTON_SIZE + 1);
        Self {
            can_proceed: Arc::new(Mutex::new(false)),
            divider_actor_id: 0,
            pages: ControlsScene::paginate(POSSIBLE_CONTROL_SETS.get().get(next_scene_name).unwrap(), lines_per_page.clone() as usize),
            current_page_actors: Vec::new(),
            current_page_index: 0,
            next_scene,
            lines_per_page,
            print_page_timer_seconds: 0.0,
        }
    }

    fn create_arrow(world: &mut World, is_p1: bool) {
        let mut pos = &(V2::one() * SCREEN_SIZE as f32) - &(V2::one() * 1.5);
        if is_p1 {
            pos.y -= SCREEN_SIZE as f32 / 2.0;
        }

        let arrow_actor_id = create_arrow_actor(world, pos, 3, Color::white(), 500, Some("arrow"));

        if is_p1 {
            let mut pivot = V2::one() * ((SCREEN_SIZE / 2) as f32 - 1.0);
            pivot.y -= (SCREEN_SIZE / 4) as f32;
            if let Some(arrow_transform) = world.get_mut_transform(&arrow_actor_id) {
                arrow_transform.rotate_around(&pivot, &180.0);
            }
        }
    }

    fn paginate(items: &[ControlsData], lines_per_page: usize) -> Vec<Vec<ControlsData>> {
        let mut result: Vec<Vec<ControlsData>> = Vec::new();

        let mut i = 0;
        while i < items.len() {
            result.push(Vec::new());
            let last_index = result.len() - 1;
            for ii in 0..lines_per_page {
                if i + ii < items.len() {
                    result[last_index].push(items[i + ii].clone());
                } else {
                    break;
                }
            }

            i += lines_per_page;
        }

        result
    }

    fn print_page(&mut self, world: &mut World) {
        if self.pages.len() == 0 {
            return;
        }

        for actor_id in &self.current_page_actors {
            world.remove_actor(actor_id);
        }

        if let Some(current_page) = self.pages.get(self.current_page_index as usize) {
            let current_page_length = current_page.iter().len();

            for player_index in 0..2 {
                let mut current_player_actors = Vec::<ActorId>::new();
                for i in 0..current_page_length {
                    let y = (SCREEN_SIZE / 2) - (BUTTON_SIZE + 1) * (i as u8 + 1) + if player_index == 0 { 0 } else { SCREEN_SIZE / 2 };
                    let mut x = 0;
                    let current_line = &current_page[current_page_length - 1usize - i];
                    for key in &current_line.keys {
                        let icon_actor_id = create_button_icon_actor(
                            world,
                            V2::new((BUTTON_SIZE / 2) as f32, y as f32),
                            BUTTON_SIZE,
                            key.clone(),
                            Some("controls button"),
                        );

                        self.current_page_actors.push(icon_actor_id);
                        current_player_actors.push(icon_actor_id);

                        x += BUTTON_SIZE + 1;

                        if let Some(operation_text) = &current_line.operation
                            && key.clone() != current_line.keys[current_line.keys.len() - 1]
                        {
                            let text_actor_id = create_text_actor(
                                world,
                                operation_text.clone(),
                                V2::new(x as f32, (y - (BUTTON_SIZE / 2)) as f32),
                                V2::new(operation_text.len() as f32 * 4.0, 5.0),
                                Color::white(),
                                None,
                                Some("controls label"),
                            );
                            self.current_page_actors.push(text_actor_id);
                            current_player_actors.push(text_actor_id);
                        }
                    }

                    x += 2;

                    let text_actor_id = create_text_actor(
                        world,
                        current_page[current_page.len() - 1usize - i].text.clone(),
                        V2::new(x as f32, (y - (BUTTON_SIZE / 2)) as f32),
                        V2::new((SCREEN_SIZE - x) as f32, BUTTON_SIZE as f32),
                        Color::white(),
                        None,
                        Some("controls text"),
                    );
                    self.current_page_actors.push(text_actor_id);
                    current_player_actors.push(text_actor_id);
                }
                if player_index == 0 {
                    let mut pivot = V2::one() * ((SCREEN_SIZE / 2) as f32 - 1.0);
                    pivot.y -= (SCREEN_SIZE / 4) as f32;

                    for actor_id in &self.current_page_actors {
                        if let Some(actor_transform) = world.get_mut_transform(actor_id) {
                            actor_transform.rotate_around(&pivot, &180.0);
                        }
                    }
                }
            }
        }
    }
}

static POSSIBLE_CONTROL_SETS: LazyLock<HashMap<String, Vec<ControlsData>>> = LazyLock::new(|| {
    HashMap::from([
        (
            String::from("pong"),
            crate::my_vec![
                ControlsData::new(crate::my_vec![Key::P1Left], "move left", None),
                ControlsData::new(crate::my_vec![Key::P1Right], "move right", None),
            ],
        ),
        (
            String::from("tetris"),
            crate::my_vec![
                ControlsData::new(crate::my_vec![Key::P1Left], "move left", None),
                ControlsData::new(crate::my_vec![Key::P1Right], "move right", None),
                ControlsData::new(crate::my_vec![Key::P1Down], "fall", None),
                ControlsData::new(crate::my_vec![Key::P1Up], "drop", None),
                ControlsData::new(crate::my_vec![Key::P1Blue], "rotate left", None),
                ControlsData::new(crate::my_vec![Key::P1Green], "rotate right", None),
                ControlsData::new(crate::my_vec![Key::P1Blue, Key::P1Green], "swap block", Some("+")),
            ],
        ),
        (
            String::from("tanks"),
            crate::my_vec![
                ControlsData::new(crate::my_vec![Key::P1AnyDirection], "move", None),
                ControlsData::new(crate::my_vec![Key::P1Blue, Key::P1Green], "fire", Some("or")),
            ],
        ),
    ])
});
