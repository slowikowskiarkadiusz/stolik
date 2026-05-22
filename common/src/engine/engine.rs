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
    scenes::{menu::menu_scene::MenuScene, tetris::tetris_scene::TetrisScene},
};
extern crate alloc;
use alloc::boxed::Box;
use alloc::sync::Arc;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::Duration;
use embassy_time::Instant;
// use esp_println::println;

pub const SCREEN_SIZE: u8 = 64;
pub type TempActorId = u16;
pub type ActorId = u16;

pub type SceneFactory = Box<dyn FnOnce() -> Box<dyn Scene> + Send + Sync>;

static SCENE_CHANNEL: Channel<CriticalSectionRawMutex, SceneFactory, 1> = Channel::new();

pub struct Engine {
    pub delta_time: f32,
    world: World,
    current_scene: Box<dyn Scene>,
    is_any_scene: bool,
    pub input: Box<dyn Input>,
    asyncable_storage: AsyncableStorage,
    screen: ColorMatrix,
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
            screen: ColorMatrix::new(SCREEN_SIZE, SCREEN_SIZE, Color::none()),
        }
    }

    pub fn run<T: Thread>(&mut self, on_frame_finished: Arc<dyn Fn(&ColorMatrix) + Send + Sync + 'static>) {
        let mut last = Instant::now();
        let target_frame = Duration::from_millis(33);

        self.ensure_scene();

        let mut engine_timer = 0.0;
        let mut last: Instant = Instant::now();

        loop {
            let frame_start = Instant::now();
            let dt = frame_start.duration_since(last).as_millis() as f32 / 1000.0;
            last = frame_start;

            engine_timer += dt;

            if engine_timer >= 0.033 {
                self.tick_frame(engine_timer, &on_frame_finished);
                self.input.as_mut().late_update(dt);
                engine_timer = 0.0;
            } else {
                self.input.as_mut().update(dt);
            }

            // Timer::after(Duration::from_millis(5)).await;
            T::sleep_for(5);
        }
    }

    pub fn ensure_scene(&mut self) {
        if !self.is_any_scene {
            self.change_scene(|| Box::new(TetrisScene::new()));
            self.is_any_scene = true;
        }
    }

    pub fn tick_frame(&mut self, delta_time: f32, on_frame_finished: &Arc<dyn Fn(&ColorMatrix) + Send + Sync + 'static>) {
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
            self.world.tick_blinkers(delta_time);
            let mut_scene = self.current_scene.as_mut();
            mut_scene.on_overlaps(&overlaps, &mut self.world, delta_time);

            self.combine_color_matrixes();
            on_frame_finished(&self.screen);

            self.input.as_mut().late_update(delta_time);
        }
    }

    fn combine_color_matrixes(&mut self) {
        self.screen.fill(Color::none());
        for i in 0..self.world.actor_count {
            let actor_id = self.world.all_actors[i];
            if let Some(render) = self.world.get_render(&actor_id)
                && let Some(transform) = self.world.get_transform(&actor_id)
            {
                let mut do_render = true;
                if let Some(blinker) = self.world.get_blinker(&actor_id) {
                    do_render = blinker.is_on;
                }

                if do_render {
                    let center = transform.center.clone();
                    let rotation = transform.rotation;
                    let anchor = transform.anchor_offset.clone();
                    self.screen.write(render, &center, Some(rotation), Some(anchor), Some(true));
                }
            }
        }
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
