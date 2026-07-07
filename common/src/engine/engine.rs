use crate::{
    engine::{
        asyncable::AsyncableStorage,
        color::Color,
        color_matrix::ColorMatrix,
        components::{
            collider::{Collider, ColliderPartDebug},
            physics::Physics,
            world::World,
        },
        input::input::Input,
        scene::{EmptyScene, Scene},
        threading_provider::Thread,
        v2::V2,
    },
    scenes::menu::menu_scene::MenuScene,
};
extern crate alloc;
use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::Instant;
#[cfg(feature = "esp")]
use esp_println::println;

pub const SCREEN_SIZE: u8 = 64;
pub const SCREEN_SIZEF32: f32 = SCREEN_SIZE as f32;
pub type TempActorId = u16;
pub type ActorId = u16;

pub type SceneFactory = Box<dyn FnOnce() -> Box<dyn Scene> + Send + Sync>;

static SCENE_CHANNEL: Channel<CriticalSectionRawMutex, SceneFactory, 1> = Channel::new();
static TICK_COUNTER: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);

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

    pub fn run<T: Thread>(
        &mut self,
        on_frame_finished: Arc<dyn Fn(&ColorMatrix) + Send + Sync + 'static>,
        colliders_debug: Option<Arc<dyn Fn(Vec<ColliderPartDebug>) + Send + Sync + 'static>>,
    ) {
        self.ensure_scene();

        const TARGET_MS: u64 = 33; // ~30 fps
        let mut last: Instant = Instant::now();

        loop {
            let frame_start = Instant::now();
            let dt = frame_start.duration_since(last).as_millis() as f32 / 1000.0;
            last = frame_start;

            self.tick_frame(dt, &on_frame_finished);
            if let Some(ref func) = colliders_debug {
                func(self.world._debug_get_collider_parts(self.world.get_camera().get_viewport().clone()));
            }
            self.input.as_mut().late_update(dt);

            let elapsed_ms = frame_start.elapsed().as_millis() as u64;
            if elapsed_ms < TARGET_MS {
                T::sleep_for(TARGET_MS - elapsed_ms);
            }
        }
    }

    pub fn ensure_scene(&mut self) {
        if !self.is_any_scene {
            self.change_scene(|| Box::new(MenuScene::new()));
            self.is_any_scene = true;
        }
    }

    pub fn tick_frame(&mut self, delta_time: f32, on_frame_finished: &Arc<dyn Fn(&ColorMatrix) + Send + Sync + 'static>) {
        self.delta_time = delta_time;
        TICK_COUNTER.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        #[cfg(feature = "esp")]
        let n = TICK_COUNTER.load(core::sync::atomic::Ordering::Relaxed);

        #[cfg(feature = "esp")]
        if n <= 5 {
            println!("[engine] tick {} start dt={}", n, delta_time);
        }

        let receiver = SCENE_CHANNEL.receiver();
        if let Ok(factory) = receiver.try_receive() {
            #[cfg(feature = "esp")]
            println!("[engine] tick {} change_scene", n);
            self.change_scene(factory);
        }

        let frame: ColorMatrix;

        {
            let mut_scene = self.current_scene.as_mut();
            #[cfg(feature = "esp")]
            if n <= 5 {
                println!("[engine] tick {} input.update", n);
            }
            self.input.as_mut().update(delta_time);
            #[cfg(feature = "esp")]
            if n <= 5 {
                println!("[engine] tick {} scene.tick", n);
            }
            mut_scene.tick(&self.input, &mut self.world, delta_time);
            #[cfg(feature = "esp")]
            if n <= 5 {
                println!("[engine] tick {} scene.render", n);
            }
            let camera = self.world.get_camera();
            frame = mut_scene.render(&camera, &mut self.world, delta_time);
            #[cfg(feature = "esp")]
            if n <= 5 {
                println!("[engine] tick {} asyncable_storage.update", n);
            }
            self.asyncable_storage.update(&mut self.world, delta_time);
        }

        {
            let mut_scene = self.current_scene.as_mut();
            #[cfg(feature = "esp")]
            if n <= 5 {
                println!("[engine] tick {} physics.update", n);
            }
            let collisions = Physics::update(&mut self.world, delta_time);
            #[cfg(feature = "esp")]
            if n <= 5 {
                println!("[engine] tick {} on_collisions", n);
            }
            mut_scene.on_collisions(&collisions, &mut self.world, delta_time);
        }

        {
            #[cfg(feature = "esp")]
            if n <= 5 {
                println!("[engine] tick {} detect_overlaps", n);
            }
            let overlaps = Collider::detect_overlaps(&self.world);
            self.world.tick_blinkers(delta_time);
            let mut_scene = self.current_scene.as_mut();
            mut_scene.on_overlaps(&overlaps, &mut self.world, delta_time);

            #[cfg(feature = "esp")]
            if n <= 5 {
                println!("[engine] tick {} combine_color_matrixes", n);
            }
            self.combine_color_matrixes(frame);
            #[cfg(feature = "esp")]
            if n <= 5 {
                println!("[engine] tick {} on_frame_finished", n);
            }
            on_frame_finished(&self.screen);

            #[cfg(feature = "esp")]
            if n <= 5 {
                println!("[engine] tick {} late_update", n);
            }
            self.input.as_mut().late_update(delta_time);
            #[cfg(feature = "esp")]
            if n <= 5 {
                println!("[engine] tick {} done", n);
            }
        }
    }

    fn combine_color_matrixes(&mut self, frame: ColorMatrix) {
        self.screen.fill(Color::none());
        let camera = self.world.get_camera();

        self.screen.write_at_origin(&frame, &V2::zero());

        let size = 1.0 / camera.get_viewport_size_relative_to_screen();
        self.screen.scale(size, Color::none(), false);
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
