use common::engine::{
    color_matrix::ColorMatrix,
    engine::{Engine, SCREEN_SIZEUSIZE},
    input::{
        gesture::Gestures,
        input::Input,
        key::{KEYS_LENGTH, Key},
    },
};
use embassy_time_driver::Driver;
use std::sync::Arc;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = performance, js_name = now)]
    fn performance_now() -> f64;
}

struct WasmTimeDriver;

impl Driver for WasmTimeDriver {
    fn now(&self) -> u64 {
        (performance_now() * 1000.0) as u64
    }

    fn schedule_wake(&self, _at: u64, _waker: &core::task::Waker) {}
}

embassy_time_driver::time_driver_impl!(static DRIVER: WasmTimeDriver = WasmTimeDriver);

struct SingleThreadedCriticalSection;
critical_section::set_impl!(SingleThreadedCriticalSection);

// Safe: wasm32-unknown-unknown without atomics is single threaded.
unsafe impl critical_section::Impl for SingleThreadedCriticalSection {
    unsafe fn acquire() -> critical_section::RawRestoreState {}
    unsafe fn release(_: critical_section::RawRestoreState) {}
}

const KEY_COUNT: usize = KEYS_LENGTH as usize;

/// Shared between the JS-facing wrapper and the two `Input` implementations.
#[derive(Default)]
struct KeyBuffers {
    down: [bool; KEY_COUNT],
    up: [bool; KEY_COUNT],
    press: [bool; KEY_COUNT],
}

struct WebInput {
    gestures: Gestures,
    shared: Arc<std::sync::Mutex<KeyBuffers>>,
    down: [bool; KEY_COUNT],
    up: [bool; KEY_COUNT],
    press: [bool; KEY_COUNT],
}

impl WebInput {
    fn new(shared: Arc<std::sync::Mutex<KeyBuffers>>) -> Self {
        Self {
            gestures: Gestures::new(),
            shared,
            down: [false; KEY_COUNT],
            up: [false; KEY_COUNT],
            press: [false; KEY_COUNT],
        }
    }

    fn query(&self, key: Option<Key>, arr: &[bool; KEY_COUNT]) -> bool {
        match key {
            Some(Key::AnyDirection) => [Key::Up, Key::Down, Key::Left, Key::Right].iter().any(|k| arr[*k as usize]),
            Some(Key::Any) => [Key::Up, Key::Down, Key::Left, Key::Right, Key::Blue, Key::Green]
                .iter()
                .any(|k| arr[*k as usize]),
            Some(k) => arr[k as usize],
            None => arr.iter().any(|x| *x),
        }
    }
}

impl Input for WebInput {
    fn gestures(&self) -> &Gestures {
        &self.gestures
    }

    fn update(&mut self, delta_time: f32) {
        {
            let mut buf = self.shared.lock().unwrap();
            for i in 0..KEY_COUNT {
                self.down[i] |= buf.down[i];
                self.up[i] |= buf.up[i];
                self.press[i] = buf.press[i];
            }
            buf.down = [false; KEY_COUNT];
            buf.up = [false; KEY_COUNT];
        }
        self.gestures.tick(self.get_snapshot(), delta_time);
    }

    fn late_update(&mut self, _: f32) {
        self.down = [false; KEY_COUNT];
        self.up = [false; KEY_COUNT];
        self.gestures.late_tick();
    }

    fn is_key_down(&self, key: Key) -> bool {
        self.query(Some(key), &self.down)
    }

    fn is_any_key_down(&self) -> bool {
        self.query(None, &self.down)
    }

    fn is_key_up(&self, key: Key) -> bool {
        self.query(Some(key), &self.up)
    }

    fn is_any_key_up(&self) -> bool {
        self.query(None, &self.up)
    }

    fn is_key_press(&self, key: Key) -> bool {
        self.query(Some(key), &self.press)
    }

    fn is_any_key_press(&self) -> bool {
        self.query(None, &self.press)
    }

    fn clear(&mut self) {
        self.down = [false; KEY_COUNT];
        self.up = [false; KEY_COUNT];
        self.press = [false; KEY_COUNT];
    }
}

fn key_from_u8(k: u8) -> Option<Key> {
    match k {
        0 => Some(Key::Down),
        1 => Some(Key::Up),
        2 => Some(Key::Left),
        3 => Some(Key::Right),
        5 => Some(Key::Blue),
        6 => Some(Key::Green),
        8 => Some(Key::Start),
        _ => None,
    }
}

#[wasm_bindgen]
pub struct Stolik {
    engine: Engine,
    players: [Arc<std::sync::Mutex<KeyBuffers>>; 2],
}

#[wasm_bindgen]
impl Stolik {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Stolik {
        std::panic::set_hook(Box::new(|info| {
            web_error(&info.to_string());
        }));

        let players = [Arc::new(std::sync::Mutex::new(KeyBuffers::default())), Arc::new(std::sync::Mutex::new(KeyBuffers::default()))];
        let mut engine = Engine::new(
            [Box::new(WebInput::new(players[0].clone())), Box::new(WebInput::new(players[1].clone()))],
            None,
        );
        engine.ensure_scene();
        Stolik {
            engine,
            players,
        }
    }

    pub fn size(&self) -> usize {
        SCREEN_SIZEUSIZE
    }

    /// Reports a key change. `player` is 0 or 1, `key` is the numeric value of `common::engine::input::key::Key`.
    pub fn set_key(&mut self, player: usize, key: u8, pressed: bool) {
        let (Some(k), Some(p)) = (key_from_u8(key), self.players.get(player)) else {
            return;
        };
        let mut buf = p.lock().unwrap();
        let i = k as usize;
        if pressed && !buf.press[i] {
            buf.down[i] = true;
        } else if !pressed && buf.press[i] {
            buf.up[i] = true;
        }
        buf.press[i] = pressed;
    }

    /// Advances the engine by `delta_time` seconds and returns RGBA bytes (row-major, y * size + x).
    pub fn tick(&mut self, delta_time: f32) -> Vec<u8> {
        let out = Arc::new(std::sync::Mutex::new(None::<Vec<u8>>));
        let out_cb = out.clone();
        let size = SCREEN_SIZEUSIZE;
        let cb: Arc<dyn Fn(&ColorMatrix) + Send + Sync + 'static> = Arc::new(move |mat: &ColorMatrix| {
            let mut bytes = vec![0u8; size * size * 4];
            for y in 0..size {
                for x in 0..size {
                    let c = mat.get(x as u8, y as u8);
                    let a = c.a as f32 / 255.0;
                    let o = (y * size + x) * 4;
                    bytes[o] = (c.r as f32 * a) as u8;
                    bytes[o + 1] = (c.g as f32 * a) as u8;
                    bytes[o + 2] = (c.b as f32 * a) as u8;
                    bytes[o + 3] = 255;
                }
            }
            *out_cb.lock().unwrap() = Some(bytes);
        });
        self.engine.tick_frame(delta_time, &cb);
        let result = out.lock().unwrap().take();
        result.unwrap_or_default()
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = error)]
    fn web_error(s: &str);
}
