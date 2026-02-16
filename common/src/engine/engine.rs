#![cfg_attr(not(feature = "std"), no_std)]

use crate::{
    engine::{
        asyncable::AsyncableStorage,
        color::Color,
        color_matrix::ColorMatrix,
        components::{blinker::Blinker, collider::Collider, world::World},
        input::input::Input,
        scene::{EmptyScene, Scene},
        threading_provider::Thread,
    },
    scenes::menu::menu_scene::MenuScene,
};
extern crate alloc;
use alloc::boxed::Box;
use alloc::sync::Arc;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::Duration;
use embassy_time::Instant;

pub static SCREEN_SIZE: u8 = 64;
pub type TempActorId = u16;
pub type ActorId = u16;

pub type SceneFactory = Box<dyn FnOnce() -> Box<dyn Scene> + Send + Sync>;

static SCENE_CHANNEL: Channel<CriticalSectionRawMutex, SceneFactory, 8> = Channel::new();

pub struct Engine {
    pub delta_time: f32,
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

        if !self.is_any_scene {
            self.change_scene(|| Box::new(MenuScene::new()));
            self.is_any_scene = true;
        }

        loop {
            let frame_start = Instant::now();
            let dt = frame_start.duration_since(last);
            last = frame_start;
            let delta_time = dt.as_millis() as f32 / 1000.0;

            self.delta_time = delta_time;

            let receiver = SCENE_CHANNEL.receiver();
            if let Ok(factory) = receiver.try_receive() {
                self.change_scene(factory);
            }

            {
                let mut_scene = self.current_scene.as_mut();
                self.input.as_mut().update(delta_time);
                mut_scene.tick(&self.input, &mut self.world, delta_time);
                self.asyncable_storage.update(&mut self.world, delta_time);
            }

            {
                let overlaps = Collider::detect_overlaps(&self.world);
                Blinker::tick(&mut self.world, delta_time);
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
                let mut do_render = true;
                if let Some(blinker) = self.world.get_blinker(actor_id) {
                    do_render = blinker.is_on;
                }

                if do_render {
                    screen.write(
                        render,
                        &transform.center,
                        Some(transform.rotation.clone()),
                        Some(transform.anchor_offset.clone()),
                        Some(true),
                    );
                }
            }
        }
        screen
    }

    pub fn change_scene<F>(&mut self, new_scene_func: F)
    where
        F: FnOnce() -> Box<dyn Scene>,
    {
        self.world.clear_all();
        let obj = new_scene_func();
        self.current_scene = obj;
        self.current_scene.as_mut().init(&mut self.world);
    }
}

pub fn open_scene(factory: SceneFactory) {
    let sender = SCENE_CHANNEL.sender();
    sender.try_send(factory).ok();
}
