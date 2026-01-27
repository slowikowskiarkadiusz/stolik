pub mod desktop_input;
pub mod desktop_threading_provider;

use crate::{desktop_input::DesktopInput, desktop_threading_provider::DesktopThread, engine::color::Color};
use core::engine::{self, color_matrix::ColorMatrix, engine::Engine};
use minifb::{Key, Window, WindowOptions};
use std::sync::{Arc, Mutex};

const DOTS_EDGE_COUNT: usize = 64;
const DOT_SIZE: usize = 10;
const SCREEN_WIDTH: usize = DOTS_EDGE_COUNT * DOT_SIZE;
const SCREEN_HEIGHT: usize = DOTS_EDGE_COUNT * DOT_SIZE;

struct Shared {
    color_matrix: Option<ColorMatrix>,
}

fn main() {
    let mut window = Window::new(
        "Circle",
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        WindowOptions::default(),
    )
    .unwrap();

    let mut buffer = vec![0u32; SCREEN_WIDTH * SCREEN_HEIGHT];

    let shared = Arc::new(Mutex::new(Shared { color_matrix: None }));
    let shared_engine_copy = shared.clone();

    std::thread::spawn(move || {
        let mut engine = Engine::new(Box::new(DesktopInput::new()));
        let on_frame_func = Arc::new(move |mat: ColorMatrix| {
            let mut s = shared_engine_copy.lock().unwrap();
            s.color_matrix = Some(mat);
        });

        engine.run::<DesktopThread>(on_frame_func);
    });

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let m = {
            let mut s = shared.lock().unwrap();
            let clone = s.color_matrix.clone();
            s.color_matrix = None;
            clone
        };

        if let Some(matrix) = m {
            for p in buf.iter_mut() {
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
                        matrix.get(x as u8, y as u8).clone(),
                    );
                }
            }
        }

        window
            .update_with_buffer(&buffer, SCREEN_WIDTH, SCREEN_HEIGHT)
            .unwrap();
    }
}

fn draw_circle(buf: &mut [u32], w: usize, h: usize, cx: i32, cy: i32, r: i32, color: Color) {
    for y in -r..=r {
        for x in -r..=r {
            if x * x + y * y <= r * r {
                let px = cx + x;
                let py = cy + y;
                if px >= 0 && py >= 0 && px < w as i32 && py < h as i32 {
                    buf[py as usize * w + px as usize] = make_color(&color);
                }
            }
        }
    }
}

fn make_color(color: &Color) -> u32 {
    let a = (((color.r as f32 * (color.a as f32 / 255.0)) as u32) << 16)
        | (((color.g as f32 * (color.a as f32 / 255.0)) as u32) << 8)
        | ((color.b as f32 * (color.a as f32 / 255.0)) as u32);
    return a;
}
