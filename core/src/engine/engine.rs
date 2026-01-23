#![cfg_attr(not(feature = "std"), no_std)]

use crate::engine::{color::Color, color_matrix::ColorMatrix, threading_provider::TThread};
use std::{
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

struct LoopThreadShared {
    last_timestamp: u128,
    delta_time: f32,
    on_frame_finished: Option<Arc<dyn Fn(ColorMatrix) + Send + Sync + 'static>>,
    is_blue: bool,
}

pub struct Engine<T: TThread> {
    update_values: Arc<Mutex<LoopThreadShared>>,
    update_thread: T,
    fixed_update_values: Arc<Mutex<LoopThreadShared>>,
    // fixed_update_thread: T,
}

impl<T: TThread> Engine<T> {
    pub fn new(on_frame_finished: Arc<dyn Fn(ColorMatrix) + Send + Sync + 'static>) -> Self {
        let update_values = Arc::new(Mutex::new(LoopThreadShared {
            delta_time: 0.0,
            last_timestamp: 0,
            on_frame_finished: Some(on_frame_finished),
            is_blue: false,
        }));
        let fixed_update_values = Arc::new(Mutex::new(LoopThreadShared {
            delta_time: 0.0,
            last_timestamp: 0,
            on_frame_finished: None,
            is_blue: false,
        }));
        let update_thread_shared = update_values.clone();
        let update_thread = T::start(Box::new(move || {
            while true {
                let now_ms = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis();
                let mut s = update_thread_shared.lock().unwrap();
                s.delta_time = (now_ms - s.last_timestamp) as f32 / 1000.0;
                s.last_timestamp = now_ms;
                s.is_blue = !s.is_blue;

                let delta_time = s.delta_time;
                let last_timestamp = s.last_timestamp;
                let on_frame_finished = s.on_frame_finished.clone();
                let is_blue = s.is_blue;

                drop(s);

                if let Some(func) = on_frame_finished {
                    func(ColorMatrix::new(
                        64,
                        64,
                        if is_blue { Color::blue() } else { Color::red() },
                    ));
                };

                T::sleep_for(33);
            }
        }));

        Self {
            update_values: update_values,
            update_thread: update_thread,
            fixed_update_values: fixed_update_values,
        }
    }
}
