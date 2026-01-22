pub mod desktop_threading_provider;

use core::engine;
use crate::engine::threading_provider::ThreadingProvider;
use crate::engine::color::Color;
use crate::desktop_threading_provider::DesktopThreadProvider;
use minifb::{Key, Window, WindowOptions};

const DOTS_EDGE_COUNT: usize = 64;
const DOT_SIZE: usize = 10;
const SCREEN_WIDTH: usize = DOTS_EDGE_COUNT * DOT_SIZE;
const SCREEN_HEIGHT: usize = DOTS_EDGE_COUNT * DOT_SIZE;

fn main() {
    let threading_provider = DesktopThreadProvider::new();
    let dt = threading_provider.start(||{});
    dt.thread.join().unwrap();

    let mut window = Window::new(
        "Circle",
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        WindowOptions::default(),
    )
    .unwrap();

    let mut buffer = vec![0u32; SCREEN_WIDTH * SCREEN_HEIGHT];

    while window.is_open() && !window.is_key_down(Key::Escape) {
        clear(&mut buffer);

        for x in 0..DOTS_EDGE_COUNT {
            for y in 0..DOTS_EDGE_COUNT {
                draw_circle(
                    &mut buffer,
                    SCREEN_WIDTH,
                    SCREEN_HEIGHT,
                    (x * DOT_SIZE + DOT_SIZE / 2) as i32,
                    (y * DOT_SIZE + DOT_SIZE / 2) as i32,
                    DOT_SIZE as i32 / 3,
                    Color::green().a(255 / 2).clone(),
                );
            }
        }

        window
            .update_with_buffer(&buffer, SCREEN_WIDTH, SCREEN_HEIGHT)
            .unwrap();
    }
}

fn clear(buf: &mut [u32]) {
    for p in buf.iter_mut() {
        *p = 0x000000;
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
