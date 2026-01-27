#![cfg_attr(not(feature = "std"), no_std)]

use crate::{
    engine::{
        actor::actor::TActor,
        color::Color,
        color_matrix::ColorMatrix,
        input::input::{EmptyInput, Input},
        scene::{EmptyScene, Scene},
        threading_provider::Thread,
    },
    scenes::pong::pong_scene::PongScene,
};
use std::{
    cell::RefCell,
    collections::HashMap,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

thread_local! {
    static ENGINE: RefCell<Engine> = RefCell::new(Engine::new_dummy());
}

pub fn get_engine<F, R>(f: F) -> R
where
    F: FnOnce(&mut Engine) -> R,
{
    ENGINE.with(|e| f(&mut e.borrow_mut()))
}

pub static SCREEN_SIZE: u8 = 64;
pub type ActorId = u16;

pub struct Engine {
    last_timestamp: u128,
    delta_time: f32,
    is_blue: bool,
    actor_map: RefCell<HashMap<ActorId, Box<dyn TActor>>>,
    current_scene: RefCell<Box<dyn Scene>>,
    input: RefCell<Box<dyn Input>>,
}

impl Engine {
    pub fn new_dummy() -> Self {
        Self {
            delta_time: 0.0,
            last_timestamp: 0,
            is_blue: false,
            actor_map: RefCell::new(HashMap::new()),
            current_scene: RefCell::new(Box::new(EmptyScene::new())),
            input: RefCell::new(Box::new(EmptyInput::new())),
        }
    }

    pub fn new(input: Box<dyn Input>) {
        ENGINE.set(Self {
            delta_time: 0.0,
            last_timestamp: 0,
            is_blue: false,
            actor_map: RefCell::new(HashMap::new()),
            current_scene: RefCell::new(Box::new(EmptyScene::new())),
            input: RefCell::new(input),
        });

        ENGINE.with_borrow_mut(|f| f.open_scene(Box::new(PongScene::new())));
    }

    pub fn run<T: Thread>(
        &mut self,
        on_frame_finished: Arc<dyn Fn(ColorMatrix) + Send + Sync + 'static>,
    ) {
        loop {
            let mut now_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();
            self.delta_time = (now_ms - self.last_timestamp) as f32 / 1000.0;
            self.last_timestamp = now_ms;
            self.is_blue = !self.is_blue;

            let delta_time = self.delta_time;
            let last_timestamp = self.last_timestamp;

            self.input.borrow_mut().update(delta_time);

            self.current_scene.get_mut().update(delta_time);

            let mut screen = ColorMatrix::new(SCREEN_SIZE, SCREEN_SIZE, Color::none());

            for (_key, actor) in self.actor_map.get_mut() {
                actor.update(delta_time);
            }

            for (_key, actor) in self.actor_map.get_mut() {
                if let Some(actor_color_matrix) = actor.get_render_color_matrix() {
                    screen.write(
                        actor_color_matrix,
                        actor.get_center(),
                        Some(actor.get_rotation().clone()),
                        Some(actor.get_anchor_offset().clone()),
                        Some(true),
                    );
                }
            }

            on_frame_finished(screen);

            self.input.borrow_mut().late_update(delta_time);

            now_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();

            T::sleep_for(33 - (last_timestamp as u64 - now_ms as u64));
        }
    }

    pub fn open_scene(&mut self, obj: Box<dyn Scene>) {
        self.actor_map.get_mut().clear();

        *self.current_scene.get_mut() = obj;
        self.current_scene.get_mut().init();
    }

    pub fn register_actor(&mut self, mut actor: Box<dyn TActor>) -> ActorId {
        let map = self.actor_map.get_mut();
        let iter = map.iter();

        let mut actor_id: ActorId = 0;
        for (k, _v) in iter {
            if k - actor_id > 1 {
                actor_id = k.clone();
                break;
            }

            actor_id = k.clone();
        }

        actor.as_mut().set_id(actor_id as ActorId);
        map.insert(actor_id as ActorId, actor);

        actor_id
    }

    pub fn unregister_actor(&mut self, actor: &Box<dyn TActor>) {
        let map = self.actor_map.get_mut();
        map.remove(&actor.get_id());
    }
}
