use crate::{
    engine::{
        ai::neat_genome::{DataForAi, NeatGenome}, asyncable::AsyncableStorage, color::Color, color_matrix::ColorMatrix, components::{
            collider::{Collider, ColliderPartDebug, CollisionResult},
            physics::Physics,
            world::World,
        }, hash_map::HashMap, input::{input::Input, key::KEYS_LENGTH}, scene::{EmptyScene, Scene}, threading_provider::Thread, v2::V2,
    }, scenes::menu::menu_scene::MenuScene,
};
extern crate alloc;
use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::Instant;

use crate::engine::ai::ai_input::AiInput;
use spin::Mutex;

pub const SCREEN_SIZE: u8 = 64;
pub const SCREEN_SIZEF32: f32 = SCREEN_SIZE as f32;
pub const SCREEN_SIZEUSIZE: usize = SCREEN_SIZE as usize;
pub type TempActorId = u16;
pub type ActorId = u16;

pub type SceneFactory = Box<dyn FnOnce() -> Box<dyn Scene> + Send + Sync>;

static SCENE_CHANNEL: Channel<CriticalSectionRawMutex, (SceneFactory, Option<NeatGenome>), 1> = Channel::new();

struct AiState {
    genome: NeatGenome,
    held: Arc<Mutex<[bool; KEYS_LENGTH as usize]>>,
}

pub struct Engine {
    pub delta_time: f32,
    world: World,
    current_scene: Box<dyn Scene>,
    is_any_scene: bool,
    pub inputs: [Box<dyn Input>; 2],
    asyncable_storage: AsyncableStorage,
    screen: ColorMatrix,
    overlaps: HashMap<ActorId, Vec<ActorId>>,
    collisions: HashMap<ActorId, Vec<(ActorId, CollisionResult)>>,
    ai_state: Option<AiState>,
    p1_backup: Option<Box<dyn Input>>,
}

impl Engine {
    pub fn new(inputs: [Box<dyn Input>; 2], open_on_scene: Option<Box<dyn Scene>>) -> Self {
        let is_a_scene = open_on_scene.is_some();
        let mut scene = open_on_scene.unwrap_or_else(|| Box::new(MenuScene::new()));
        let mut world = World::new();
        scene.init(&mut world);
        Self {
            delta_time: 0.0,
            world: world,
            current_scene: scene,
            is_any_scene: is_a_scene,
            inputs,
            asyncable_storage: AsyncableStorage::new(),
            screen: ColorMatrix::new(SCREEN_SIZE, SCREEN_SIZE, Color::none()),
            overlaps: HashMap::new(),
            collisions: HashMap::new(),
            ai_state: None,
            p1_backup: None,
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

            self.inputs[0].as_mut().late_update(dt);
            self.inputs[1].as_mut().late_update(dt);

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

        let receiver = SCENE_CHANNEL.receiver();
        if let Ok((scene_factory, p1_genome)) = receiver.try_receive() {
            self.set_ai_player(p1_genome);
            self.change_scene(scene_factory);
        }

        self.tick_ai_player();

        let frame: ColorMatrix;

        {
            let mut_scene = self.current_scene.as_mut();

            self.inputs[0].as_mut().update(delta_time);
            self.inputs[1].as_mut().update(delta_time);

            mut_scene.tick([&self.inputs[0], &self.inputs[1]], &mut self.world, delta_time);
            let camera = self.world.get_camera();
            frame = mut_scene.render(&camera, &mut self.world, delta_time);
            self.asyncable_storage.update(&mut self.world, delta_time);
        }

        {
            let mut_scene = self.current_scene.as_mut();
            Physics::update(&mut self.world, delta_time, &mut self.collisions);
            mut_scene.on_collisions(&self.collisions, &mut self.world, delta_time);
        }

        {
            Collider::detect_overlaps(&self.world, &mut self.overlaps);
            let mut_scene = self.current_scene.as_mut();
            mut_scene.on_overlaps(&self.overlaps, &mut self.world, delta_time);
            self.combine_color_matrixes(frame);
            on_frame_finished(&self.screen);

            self.inputs[0].as_mut().late_update(delta_time);
            self.inputs[1].as_mut().late_update(delta_time);
        }
    }

    fn set_ai_player(&mut self, genome: Option<NeatGenome>) {
        match genome {
            Some(g) => {
                let held = Arc::new(Mutex::new([false; KEYS_LENGTH as usize]));
                let backup = core::mem::replace(&mut self.inputs[1], Box::new(AiInput::new(held.clone())));
                self.p1_backup = Some(backup);
                self.ai_state = Some(AiState { genome: g, held });
            }
            None => {
                if let Some(backup) = self.p1_backup.take() {
                    self.inputs[1] = backup;
                }
                self.ai_state = None;
            }
        }
    }

    fn tick_ai_player(&mut self) {
        if let Some(state) = &mut self.ai_state {
            let data = self.current_scene.get_data_for_ai();
            if !data.inputs[1].is_empty() {
                let outputs = state.genome.activate(data.inputs[1].clone());
                *state.held.lock() = ai_outputs_to_keys(&outputs);
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

    pub fn get_scene_data_for_ai(&self) -> DataForAi {
        self.current_scene.get_data_for_ai()
    }

    pub fn is_game_over(&self) -> bool {
        self.current_scene.is_game_over()
    }
}

fn ai_outputs_to_keys(outputs: &[f64]) -> [bool; KEYS_LENGTH as usize] {
    let mut keys = [false; KEYS_LENGTH as usize];
    if let Some(&v) = outputs.first() {
        keys[2] = v > 0.5;
        keys[3] = v < 0.5;
    }
    keys
}

pub fn open_scene(factory: SceneFactory, p1_genome: Option<NeatGenome>) {
    let sender = SCENE_CHANNEL.sender();
    sender.try_send((factory, p1_genome)).ok();
}
