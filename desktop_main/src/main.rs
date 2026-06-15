pub mod desktop_input;
pub mod desktop_threading_provider;

use crate::{desktop_input::DesktopInput, desktop_threading_provider::DesktopThread};
use common::engine::{
    color::Color,
    color_matrix::ColorMatrix,
    components::{
        camera::Viewport,
        collider::{ColliderPartDebug, ColliderShape},
    },
    engine::Engine,
};
use minifb::{Key, Window, WindowOptions};
use std::{
    collections::HashMap,
    panic,
    sync::{Arc, Mutex},
};

const DOTS_EDGE_COUNT: usize = 64;
const DOT_SIZE: usize = 10;
const SCREEN_WIDTH: usize = DOTS_EDGE_COUNT * DOT_SIZE;
const SCREEN_HEIGHT: usize = DOTS_EDGE_COUNT * DOT_SIZE;

struct Shared {
    color_matrix: Option<ColorMatrix>,
    colliders: Option<Vec<ColliderPartDebug>>,
}

fn main() {
    let debug = std::env::var("STOLIK_DEBUG").map(|v| v == "1").unwrap_or(false);

    panic::set_hook(Box::new(|info| {
        eprintln!("PANIC: {}", info);
        eprintln!("Backtrace:\n{:?}", std::backtrace::Backtrace::force_capture());
    }));
    let mut window = Window::new("Circle", SCREEN_WIDTH, SCREEN_HEIGHT, WindowOptions::default()).unwrap();

    let mut buffer = vec![0u32; SCREEN_WIDTH * SCREEN_HEIGHT];

    let shared = Arc::new(Mutex::new(Shared {
        color_matrix: None,
        colliders: None,
    }));
    let shared_engine_copy = shared.clone();
    let one_more_copy = shared.clone();

    let input_state = Arc::new(Mutex::new(init_input_state()));
    let cloned_input_state = input_state.clone();

    std::thread::spawn(move || {
        let mut engine = Engine::new(Box::new(DesktopInput::new(cloned_input_state)));
        let on_frame_func = Arc::new(move |mat: &ColorMatrix| {
            let mut s = shared_engine_copy.lock().unwrap();
            s.color_matrix = Some(mat.clone());
        });

        let mut debug_func: Option<Arc<dyn Fn(Vec<ColliderPartDebug>) + Send + Sync + 'static>> = None;
        if debug {
            debug_func = Some(Arc::new(move |colliders: Vec<ColliderPartDebug>| {
                let mut s = one_more_copy.lock().unwrap();
                s.colliders = Some(colliders.clone());
            }));
        }

        engine.run::<DesktopThread>(on_frame_func, debug_func);
    });

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let mut lock = input_state.lock();
        let locked_input_state = lock.as_mut().unwrap();
        update_input_state(&mut *locked_input_state, &window);
        drop(lock);

        let m = {
            let mut s = shared.lock().unwrap();
            let clone = s.color_matrix.clone();
            s.color_matrix = None;
            clone
        };

        if let Some(matrix) = m {
            for p in buffer.iter_mut() {
                *p = 0x000000;
            }

            for x in 0..DOTS_EDGE_COUNT {
                for y in 0..DOTS_EDGE_COUNT {
                    draw_circle(
                        &mut buffer,
                        SCREEN_WIDTH,
                        SCREEN_HEIGHT,
                        (x * DOT_SIZE + DOT_SIZE / 2) as i32,
                        (y * DOT_SIZE + DOT_SIZE / 2) as i32,
                        DOT_SIZE as i32 / 3,
                        matrix.get(x as u8, y as u8),
                        false,
                    );
                }
            }
        }

        let c = {
            let mut s = shared.lock().unwrap();
            let clone = s.colliders.clone();
            s.colliders = None;
            clone
        };

        if let Some(parts) = c {
            for part in &parts {
                let color = if part.is_overlap { Color::blue() } else { Color::green() };
                let scale = DOT_SIZE as f32;
                let cx = (part.center.x * scale) as i32;
                let cy = (part.center.y * scale) as i32;

                match part.shape {
                    ColliderShape::Circle => {
                        let radius = (part.extend.x / 2.0 * scale) as i32;
                        draw_circle(&mut buffer, SCREEN_WIDTH, SCREEN_HEIGHT, cx, cy, radius, &color, true);
                    }
                    ColliderShape::Rect => {
                        let hw = (part.extend.x / 2.0 * scale) as i32;
                        let hh = (part.extend.y / 2.0 * scale) as i32;
                        draw_rect_outline(&mut buffer, SCREEN_WIDTH, SCREEN_HEIGHT, cx - hw, cy - hh, cx + hw, cy + hh, &color);
                    }
                }
            }
        }

        window.update_with_buffer(&buffer, SCREEN_WIDTH, SCREEN_HEIGHT).unwrap();
    }
}

fn draw_circle(buf: &mut [u32], w: usize, h: usize, cx: i32, cy: i32, r: i32, color: &Color, outline: bool) {
    for y in -r..=r {
        for x in -r..=r {
            let dist_sq = x * x + y * y;
            let inside = dist_sq <= r * r;
            let draw = if outline { inside && dist_sq > (r - 1) * (r - 1) } else { inside };
            if draw {
                let px = cx + x;
                let py = cy + y;
                if px >= 0 && py >= 0 && px < w as i32 && py < h as i32 {
                    buf[py as usize * w + px as usize] = make_color(&color);
                }
            }
        }
    }
}

fn draw_line(buffer: &mut [u32], width: usize, height: usize, x0: i32, y0: i32, x1: i32, y1: i32, color: &Color) {
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx = if x1 >= x0 { 1 } else { -1 };
    let sy = if y1 >= y0 { 1 } else { -1 };
    let mut x = x0;
    let mut y = y0;
    let mut err = dx - dy;

    loop {
        if x >= 0 && (x as usize) < width && y >= 0 && (y as usize) < height {
            buffer[y as usize * width + x as usize] = make_color(color);
        }
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }
}

fn draw_rect_outline(buffer: &mut [u32], width: usize, height: usize, x0: i32, y0: i32, x1: i32, y1: i32, color: &Color) {
    draw_line(buffer, width, height, x0, y0, x1, y0, color);
    draw_line(buffer, width, height, x1, y0, x1, y1, color);
    draw_line(buffer, width, height, x1, y1, x0, y1, color);
    draw_line(buffer, width, height, x0, y1, x0, y0, color);
}

fn make_color(color: &Color) -> u32 {
    let a = (((color.r as f32 * (color.a as f32 / 255.0)) as u32) << 16)
        | (((color.g as f32 * (color.a as f32 / 255.0)) as u32) << 8)
        | ((color.b as f32 * (color.a as f32 / 255.0)) as u32);
    return a;
}

fn init_input_state() -> HashMap<Key, (bool, bool)> {
    let mut input_state = HashMap::new();
    input_state.insert(Key::Space, (false, false));
    input_state.insert(Key::S, (false, false));
    input_state.insert(Key::W, (false, false));
    input_state.insert(Key::A, (false, false));
    input_state.insert(Key::D, (false, false));
    input_state.insert(Key::F, (false, false));
    input_state.insert(Key::G, (false, false));
    input_state.insert(Key::Down, (false, false));
    input_state.insert(Key::Up, (false, false));
    input_state.insert(Key::Left, (false, false));
    input_state.insert(Key::Right, (false, false));
    input_state.insert(Key::O, (false, false));
    input_state.insert(Key::P, (false, false));
    input_state
}

fn update_input_state(input_state: &mut HashMap<Key, (bool, bool)>, window: &Window) {
    for (k, v) in input_state {
        *v = (
            v.0 || window.is_key_pressed(k.clone(), minifb::KeyRepeat::No),
            v.1 || window.is_key_released(k.clone()),
        )
    }
}
