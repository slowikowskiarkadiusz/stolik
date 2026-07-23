extern crate alloc;
use alloc::{boxed::Box, string::String, vec, vec::Vec};
use embassy_sync::lazy_lock::LazyLock;
// use esp_println::println;
use crate::{
    engine::{
        actor::{arrow_actor::render_arrow, rectangle_actor::create_rectangle_actor, text::render_text}, ai::neat_genome::DataForAi, color::Color, color_matrix::ColorMatrix, components::{camera::Camera, collider::CollisionResult, world::World}, engine::{ActorId, SCREEN_SIZE, SceneFactory, open_scene}, hash_map::HashMap, input::{input::Input, key::Key}, scene::Scene, v2::V2,
    }, scenes::{controls::button_icon_actor::make_button_matrix, menu::menu_scene::MenuScene},
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
    can_proceed: bool,
    divider_actor_id: ActorId,
    pages: Vec<Vec<ControlsData>>,
    current_page_index: u8,
    next_scene: SceneFactory,
    #[allow(dead_code)]
    lines_per_page: u8,
    print_page_timer_seconds: f32,
    allow_proceeding_timer_sec: f32,
    arrow_blink_timer: f32,
    arrow_visible: bool,
}

impl Scene for ControlsScene {
    fn init(&mut self, world: &mut crate::engine::components::world::World) {
        // println!("[ControlsScene] init start");

        self.divider_actor_id = create_rectangle_actor(
            world,
            V2::one() * (SCREEN_SIZE / 2) as f32,
            V2::new(SCREEN_SIZE as f32, 2.0),
            // Color::white().a(127).clone(),
            None,
            Some("divider"),
        );

        // println!("[ControlsScene] divider created");

        // println!("[ControlsScene] arrows created");
        // self.print_page(world);
        // println!("[ControlsScene] init done");
    }

    fn tick(&mut self, inputs: [&Box<dyn Input>; 2], _world: &mut World, _delta_time: f32) {
        // println!(
        //     "[ControlsScene] can_proceed: {}, any_key_down: {}",
        //     self.can_proceed,
        //     input.is_any_key_down()
        // );

        if self.can_proceed && (inputs[0].is_any_key_down() || inputs[1].is_any_key_down()) {
            // println!("[ControlsScene] opening next scene");
            let factory = core::mem::replace(&mut self.next_scene, Box::new(|| Box::new(MenuScene::new())));
            open_scene(factory);
        }
    }

    fn render(&mut self, camera: &Camera, world: &mut World, delta_time: f32) -> ColorMatrix {
        let mut result = ColorMatrix::new(
            camera.get_viewport().get_size().x as u8,
            camera.get_viewport().get_size().y as u8,
            Color::none(),
        );

        if !self.can_proceed {
            self.allow_proceeding_timer_sec -= delta_time;
            if self.allow_proceeding_timer_sec <= 0.0 {
                self.allow_proceeding_timer_sec = 0.0;
                self.can_proceed = true;
            }
        }

        if self.can_proceed {
            self.arrow_blink_timer -= delta_time;
            if self.arrow_blink_timer <= 0.0 {
                self.arrow_visible = !self.arrow_visible;
                self.arrow_blink_timer = 0.4;
            }
            if self.arrow_visible {
                let corner = V2::new((SCREEN_SIZE - 2) as f32, (SCREEN_SIZE - 2) as f32);
                render_arrow(world, corner, 3, Color::white(), 0, camera, &mut result);
            }
        }

        self.print_page_timer_seconds += delta_time;

        if self.print_page_timer_seconds > 1.6 {
            self.print_page_timer_seconds = 0.0;
            self.current_page_index = (self.current_page_index + 1) % self.pages.len() as u8;
        }
        self.print_page(&mut result, camera);

        if camera.can_see_actor(self.divider_actor_id, world) {
            if let Some(transform) = world.get_mut_transform(&self.divider_actor_id) {
                result.write(
                    &ColorMatrix::new(transform.size.x as u8, transform.size.y as u8, Color::white()),
                    &transform.center,
                    None,
                    None,
                    None,
                    Some(camera),
                );
            }
        }

        result
    }

    fn on_overlaps(&mut self, _: &HashMap<ActorId, Vec<ActorId>>, _: &mut World, _: f32) {}

    fn on_collisions(&mut self, _collisions: &HashMap<u16, Vec<(u16, CollisionResult)>>, _world: &mut World, _delta_time: f32) {}

    fn get_ai_inputs(&self) -> DataForAi {
        todo!()
    }
}

impl ControlsScene {
    pub fn new(next_scene_name: &str, next_scene: SceneFactory) -> Self {
        let lines_per_page = (SCREEN_SIZE / 2 - 5) / (BUTTON_SIZE + 1);
        Self {
            can_proceed: false,
            divider_actor_id: 0,
            pages: ControlsScene::paginate(
                POSSIBLE_CONTROL_SETS.get().get(next_scene_name).unwrap(),
                lines_per_page.clone() as usize,
            ),
            // current_icon_actors: Vec::new(),
            // current_text_actors: Vec::new(),
            current_page_index: 0,
            next_scene,
            lines_per_page,
            print_page_timer_seconds: 0.0,
            allow_proceeding_timer_sec: 2.0,
            arrow_blink_timer: 0.4,
            arrow_visible: false,
        }
    }

    fn paginate(items: &[ControlsData], lines_per_page: usize) -> Vec<Vec<ControlsData>> {
        let mut result: Vec<Vec<ControlsData>> = Vec::new();

        let mut i = 0;
        while i < items.len() {
            result.push(vec![]);
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

    fn print_page(&mut self, result: &mut ColorMatrix, camera: &Camera) {
        if self.pages.len() == 0 {
            return;
        }

        if let Some(current_page) = self.pages.get(self.current_page_index as usize) {
            let current_page_length = current_page.len();
            let half = SCREEN_SIZE / 2;

            for i in 0..current_page_length {
                let y = half + (BUTTON_SIZE + 1) * (i as u8 + 1);
                let mut x = 0;
                let current_line = &current_page[i];

                for key in &current_line.keys {
                    result.write(
                        &make_button_matrix(BUTTON_SIZE, key.clone()),
                        &V2::new((BUTTON_SIZE / 2) as f32 + 1.0, y as f32),
                        None,
                        None,
                        None,
                        Some(camera),
                    );

                    x += BUTTON_SIZE + 1;

                    if let Some(operation_text) = &current_line.operation
                        && key.clone() != current_line.keys[current_line.keys.len() - 1]
                    {
                        let _text_actor_id = render_text(
                            operation_text.as_str(),
                            V2::new(x as f32, (y - (BUTTON_SIZE / 2)) as f32),
                            V2::new(operation_text.len() as f32 * 4.0, 5.0),
                            None,
                            None,
                            Color::white(),
                            Some(camera),
                            result,
                        );
                    }
                }

                x += 2;

                let _text_actor_id = render_text(
                    current_line.text.as_str(),
                    V2::new(x as f32, (y - (BUTTON_SIZE / 2)) as f32),
                    V2::new((SCREEN_SIZE - x) as f32, BUTTON_SIZE as f32),
                    None,
                    None,
                    Color::white(),
                    Some(camera),
                    result,
                );
            }

            let copy = result.clone();
            result.write(
                &copy,
                &V2::new(half as f32 - 1.0, half as f32 - 1.0),
                Some(180.0),
                None,
                None,
                Some(camera),
            );
        }
    }
}

static POSSIBLE_CONTROL_SETS: LazyLock<HashMap<String, Vec<ControlsData>>> = LazyLock::new(|| {
    HashMap::from([
        (
            String::from("pong"),
            vec![
                ControlsData::new(vec![Key::Left], "move left", None),
                ControlsData::new(vec![Key::Right], "move right", None),
            ],
        ),
        (
            String::from("tetris"),
            vec![
                ControlsData::new(vec![Key::Left], "move left", None),
                ControlsData::new(vec![Key::Right], "move right", None),
                ControlsData::new(vec![Key::Down], "fall", None),
                ControlsData::new(vec![Key::Up], "drop", None),
                ControlsData::new(vec![Key::Blue], "rotate left", None),
                ControlsData::new(vec![Key::Green], "rotate right", None),
                ControlsData::new(vec![Key::Blue, Key::Green], "swap block", Some("+")),
            ],
        ),
        (
            String::from("tanks"),
            vec![
                ControlsData::new(vec![Key::AnyDirection], "move", None),
                ControlsData::new(vec![Key::Blue, Key::Green], "fire", Some("or")),
            ],
        ),
        (
            String::from("astro_duel"),
            vec![
                ControlsData::new(vec![Key::AnyDirection], "thrust", None),
                ControlsData::new(vec![Key::Blue], "fire", None),
                ControlsData::new(vec![Key::Green], "dash", None),
            ],
        ),
        (
            String::from("game_of_life"),
            vec![
                ControlsData::new(vec![Key::AnyDirection], "move selection", None),
                ControlsData::new(vec![Key::Blue], "select", None),
                ControlsData::new(vec![Key::Green], "start stop playing", None),
                ControlsData::new(vec![Key::Down, Key::Up], "faster slower", None),
            ],
        ),
    ])
});
