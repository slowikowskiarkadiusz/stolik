#![cfg_attr(not(feature = "std"), no_std)]

use crate::{
    engine::{
        color::Color,
        color_matrix::ColorMatrix,
        components::world::World,
        input::input::Input,
        scene::{EmptyScene, Scene},
        threading_provider::Thread,
    },
    scenes::pong::pong_scene::PongScene,
};
use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

pub static SCREEN_SIZE: u8 = 64;
pub type TempActorId = u16;
pub type ActorId = u16;

pub struct Engine {
    last_timestamp: u128,
    pub delta_time: f32,
    is_blue: bool,
    world: World,
    current_scene: Box<dyn Scene>,
    pub input: Box<dyn Input>,
}

impl Engine {
    pub fn new(input: Box<dyn Input>) -> Self {
        let mut engine = Self {
            delta_time: 0.0,
            last_timestamp: 0,
            is_blue: false,
            world: World::new(),
            current_scene: Box::new(EmptyScene::new()),
            input: input,
        };

        engine.close_scene();
        let pong_scene = PongScene::new();
        engine.open_scene(Box::new(pong_scene));

        engine
    }

    pub fn run<T: Thread>(&mut self, on_frame_finished: Arc<dyn Fn(ColorMatrix) + Send + Sync + 'static>) {
        loop {
            let mut now_ms = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
            self.delta_time = (now_ms - self.last_timestamp) as f32 / 1000.0;
            self.last_timestamp = now_ms;
            self.is_blue = !self.is_blue;

            let delta_time = self.delta_time;
            let last_timestamp = self.last_timestamp;

            let mut_scene = self.current_scene.as_mut();

            self.input.as_mut().update(delta_time);

            mut_scene.tick(&self.input, &mut self.world, delta_time);

            let mut screen = ColorMatrix::new(SCREEN_SIZE, SCREEN_SIZE, Color::none());
            for actor_id in self.world.get_all_actors() {
                if let Some(render) = self.world.get_render(actor_id)
                    && let Some(transform) = self.world.get_transform(actor_id)
                {
                    // println!("rendering actor: {}", actor_id);
                    // println!("{}", render);
                    screen.write(
                        render,
                        &transform.center,
                        Some(transform.rotation.clone()),
                        Some(transform.anchor_offset.clone()),
                        Some(true),
                    );
                }
            }

            on_frame_finished(screen);

            self.input.as_mut().late_update(delta_time);

            now_ms = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();

            T::sleep_for(33 - (now_ms as u64 - last_timestamp as u64));
        }
    }

    pub fn open_scene(&mut self, obj: Box<dyn Scene>) {
        self.current_scene = obj;
        self.current_scene.as_mut().init(&mut self.world);
    }

    pub fn close_scene(&mut self) {
        // self.actor_map.get_mut().clear();
    }
}
