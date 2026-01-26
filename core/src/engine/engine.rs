#![cfg_attr(not(feature = "std"), no_std)]

use crate::{
    engine::{
        actor::actor::{InnerActor, TActor},
        color::Color,
        color_matrix::ColorMatrix,
        scene::Scene,
        threading_provider::Thread,
        v2::V2,
    },
    scenes::pong::pong_scene::PongScene,
};
use std::{
    cell::RefCell,
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

pub static SCREEN_SIZE: u8 = 64;
thread_local! {
    static ACTOR_MAP: RefCell<HashMap<u16, Box<dyn TActor>>> = RefCell::new(HashMap::new());
    static CURRENT_SCENE: RefCell<Box<dyn Scene>> = RefCell::new(Box::new(PongScene::new()));
}

pub struct Engine {
    last_timestamp: u128,
    delta_time: f32,
    is_blue: bool,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            delta_time: 0.0,
            last_timestamp: 0,
            is_blue: false,
        }
    }

    pub fn run<T: Thread>(
        &mut self,
        on_frame_finished: Arc<dyn Fn(ColorMatrix) + Send + Sync + 'static>,
    ) {
        open_scene(Box::new(PongScene::new()));

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

            CURRENT_SCENE.with(|f| f.borrow().update(delta_time));

            let mut screen = ColorMatrix::new(SCREEN_SIZE, SCREEN_SIZE, Color::none());

            ACTOR_MAP.with(|f| {
                for (_key, actor) in f.borrow_mut().iter_mut() {
                    actor.update(delta_time);
                }

                for (_key, actor) in f.borrow().iter() {
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
            });

            on_frame_finished(screen);

            now_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();

            T::sleep_for(33 - (last_timestamp as u64 - now_ms as u64));
        }
    }
}

pub fn open_scene(obj: Box<dyn Scene>) {
    ACTOR_MAP.with(|f| {
        f.borrow_mut().clear();
    });

    CURRENT_SCENE.with_borrow_mut(|f| {
        *f = obj;
        f.init();
    });
}

pub fn register_actor(mut actor: Box<dyn TActor>) -> u16 {
    let mut actor_id: u16 = 0;
    ACTOR_MAP.with(|x| {
        let mut map = x.borrow_mut();
        let iter = map.iter();

        for (k, _v) in iter {
            if k - actor_id > 1 {
                actor_id = k.clone();
                break;
            }

            actor_id = k.clone();
        }

        actor.as_mut().set_id(actor_id as u16);
        map.insert(actor_id as u16, actor);

        actor_id
    });

    actor_id
}

pub fn unregister_actor(actor: &Box<dyn TActor>) {
    ACTOR_MAP.with(|x| {
        let mut map = x.borrow_mut();
        map.remove(&actor.get_id());
    });
}
