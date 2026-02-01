#![cfg_attr(not(feature = "std"), no_std)]

use crate::{
    engine::{
        asyncable::AsyncableStorage,
        color::Color,
        color_matrix::ColorMatrix,
        components::{collider::Collider, world::World},
        input::input::Input,
        scene::{EmptyScene, Scene},
        threading_provider::Thread,
    },
    scenes::pong::pong_scene::PongScene,
};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

pub static SCREEN_SIZE: u8 = 64;
pub type TempActorId = u16;
pub type ActorId = u16;

pub struct Engine {
    pub delta_time: f32,
    is_blue: bool,
    world: World,
    current_scene: Box<dyn Scene>,
    is_any_scene: bool,
    pub input: Box<dyn Input>,
    asyncable_storage: AsyncableStorage,
}

impl Engine {
    pub fn new(input: Box<dyn Input>) -> Self {
        Self {
            delta_time: 0.0,
            is_blue: false,
            world: World::new(),
            current_scene: Box::new(EmptyScene::new()),
            is_any_scene: false,
            input: input,
            asyncable_storage: AsyncableStorage::new(),
        }
    }

    pub fn run<T: Thread>(&mut self, on_frame_finished: Arc<dyn Fn(ColorMatrix) + Send + Sync + 'static>) {
        let mut last = Instant::now();
        let target_frame = Duration::from_millis(33);

        loop {
            let frame_start = Instant::now();
            let dt = frame_start.duration_since(last);
            last = frame_start;
            let delta_time = dt.as_secs_f32();

            self.delta_time = delta_time;
            self.is_blue = !self.is_blue;

            if !self.is_any_scene {
                self.open_scene(|| Box::new(PongScene::new()));
                self.is_any_scene = true;
            }

            {
                let mut_scene = self.current_scene.as_mut();
                self.input.as_mut().update(delta_time);
                mut_scene.tick(&self.input, &mut self.world, delta_time);
                self.asyncable_storage.update(&mut self.world, delta_time);
            }

            {
                let overlaps = Collider::detect_overlaps(&self.world);
                let mut_scene = self.current_scene.as_mut();
                mut_scene.on_overlaps(&overlaps, &mut self.world, delta_time);

                on_frame_finished(self.combine_color_matrixes());

                self.input.as_mut().late_update(delta_time);
            }

            let frame_time = frame_start.elapsed();
            if frame_time < target_frame {
                T::sleep_for((target_frame - frame_time).as_millis() as u64);
            }
        }
    }

    fn combine_color_matrixes(&mut self) -> super::matrix::Matrix<Color> {
        let mut screen = ColorMatrix::new(SCREEN_SIZE, SCREEN_SIZE, Color::none());
        for actor_id in &self.world.all_actors {
            if let Some(render) = self.world.get_render(actor_id)
                && let Some(transform) = self.world.get_transform(actor_id)
            {
                screen.write(
                    render,
                    &transform.center,
                    Some(transform.rotation.clone()),
                    Some(transform.anchor_offset.clone()),
                    Some(true),
                );
            }
        }
        screen
    }

    pub fn open_scene<F>(&mut self, new_scene_func: F)
    where
        F: Fn() -> Box<dyn Scene>,
    {
        self.world.clear_all();
        let obj = new_scene_func();
        self.current_scene = obj;
        self.current_scene.as_mut().init(&mut self.world);
    }
}
