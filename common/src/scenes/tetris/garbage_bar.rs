use crate::engine::{color::Color, color_matrix::ColorMatrix, v2::V2};
use crate::scenes::tetris::board::SCALE;

const BLINKING_DELAY: f32 = 100.0;

pub struct GarbageBar {
    pub center: V2,
    color_matrix: ColorMatrix,
    blink_off_color_matrix: ColorMatrix,
    current_level: u8,
    last_drawn_level: u8, // skip draw_level_on_matrix() when level hasn't changed
    // anim_current_level: u8,
    max_level: u8, // logical (1×) height
    is_ready_to_be_popped: bool,
    is_blink_on: bool,
    blinking_timer: f32,
    skip_next_pop: bool,
    color: Color,
}

impl GarbageBar {
    pub fn new(center: V2, size: V2, color: Color) -> Self {
        let s = SCALE as u8;
        Self {
            center,
            // Pre-scaled at SCALE× — render() returns directly, no per-frame scaling.
            color_matrix: ColorMatrix::new(size.x as u8 * s, size.y as u8 * s, color.clone()),
            blink_off_color_matrix: ColorMatrix::new(size.x as u8 * s, size.y as u8 * s, color.clone()),
            current_level: 0,
            last_drawn_level: u8::MAX, // force first draw
            max_level: size.y as u8, // logical units
            is_ready_to_be_popped: false,
            is_blink_on: true,
            blinking_timer: 0.0,
            skip_next_pop: false,
            color,
        }
    }

    pub fn decrease_and_get_left(&mut self, count: u8) -> u8 {
        self.skip_next_pop = true;
        let left = count.saturating_sub(self.current_level);
        self.current_level = self.current_level.saturating_sub(count);
        self.draw_level_on_matrix();
        left
    }

    pub fn add_lines(&mut self, count: u8) {
        self.skip_next_pop = false;
        self.current_level += count;
        self.draw_level_on_matrix();
        //TODO animation
    }

    fn draw_level_on_matrix(&mut self) {
        if self.current_level == self.last_drawn_level {
            return; // nothing changed — skip the full matrix rewrite
        }
        self.last_drawn_level = self.current_level;

        let color = Color::lerp(&Color::yellow(), &Color::red(), self.current_level as f32 / self.max_level as f32);
        let s = SCALE as u8;
        let h = self.color_matrix.height;
        let w = self.color_matrix.width;
        let filled = self.current_level * s;

        for sy in 0..h {
            for sx in 0..w {
                let c = if sy >= h - filled { color.clone() } else { self.color.clone() };
                self.color_matrix.set(sx, sy, c);
            }
        }
    }

    pub fn pop(&mut self) -> bool {
        if self.is_ready_to_be_popped && self.current_level > 0 && !self.skip_next_pop {
            self.current_level -= 1;

            if self.current_level <= 0 {
                self.current_level = 0;
                self.is_ready_to_be_popped = false;
                self.color_matrix.write_at_origin(&self.blink_off_color_matrix, &V2::zero());
            }
            return true;
        } else {
            self.skip_next_pop = false;
            return false;
        }
    }

    pub fn tick(&mut self, delta_time: f32) {
        if self.is_ready_to_be_popped && self.current_level > 0 {
            self.blinking_timer += delta_time;
            if self.blinking_timer > BLINKING_DELAY {
                self.is_blink_on = !self.is_blink_on;
                self.blinking_timer = 0.0;
            }
        } else {
            self.is_blink_on = true;
        }
    }

    pub fn render(&self) -> &ColorMatrix {
        if self.is_blink_on {
            &self.color_matrix
        } else {
            &self.blink_off_color_matrix
        }
    }
}
