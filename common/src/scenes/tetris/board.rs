extern crate alloc;

use alloc::boxed::Box;
use rand::RngCore;
use rand::{SeedableRng, rngs::SmallRng};

use crate::engine::components::transform::Transform;
use crate::engine::components::world::World;
use crate::engine::engine::{ActorId, SCREEN_SIZE};
use crate::engine::input::gesture::{Gesture, State};
use crate::engine::input::input::Input;
use crate::engine::input::key::Key;
use crate::scenes::tetris::world::TetrisWorld;
use crate::{
    engine::{color::Color, color_matrix::ColorMatrix, matrix::Matrix, v2::V2},
    scenes::tetris::{block::Block, garbage_bar::GarbageBar, hold_logic::HoldLogic, shape::Shape},
};

#[cfg(feature = "esp")]
use esp_println::println;

/// Rendering scale. Logic stays at 1×; all render matrices are pre-scaled at creation.
/// Toggle between 1 and 2 for debugging — no per-frame scaling ever happens.
pub const SCALE: u8 = 2;

const BOARD_WIDTH: u8 = 10;
const BOARD_HEIGHT: u8 = 20;
const DROPPING_DELAY_SEC: f32 = 1.0;
const FASTER_DROPPING_DELAY_SEC: f32 = 0.1;
const LOCK_DELAY_SEC: f32 = 1.0;

pub const BLOCKS_COLORS: &[Color; 7] = &[
    Color::new(0, 255, 255, 255), // Cyan
    Color::new(255, 255, 0, 255), // Yellow
    Color::new(127, 0, 127, 255), // Purple
    Color::new(0, 255, 0, 255),   // Green
    Color::new(255, 0, 0, 255),   // Red
    Color::new(0, 0, 255, 255),   // Blue
    Color::new(255, 163, 0, 255), // Orange
];

pub struct Board {
    is_cell_taken: Matrix<bool>,
    current_agent: Option<Block>,
    current_agent_shadow: Option<Block>,
    garbage_bar: GarbageBar,
    hold_logic: HoldLogic,
    can_drop: bool,
    /// Single SCALE× buffer combining border + dropped blocks.
    /// Updated only on piece landing / line clear / garbage pop — never every frame.
    /// Co klatkę: jeden write_at_origin() zamiast fill() + write(border) + write(dropped_blocks).
    static_buf: ColorMatrix,
    /// SCALE× pixel origin of the board area inside static_buf.
    /// Computed once in new() — used by drop/clear_lines/pop_garbage_lines to write directly.
    board_area_x: u8,
    board_area_y: u8,
    /// Cached garbage hole column — same deterministic value every call, so compute once.
    garbage_hole: u8,
    continue_dropping: bool,
    drop_timer: f32,
    lock_delay_timer: f32,
    dropping_delay_value: f32,
    already_switched_pieces: bool,
    /// Fixed-size bag — no heap allocation
    spawn_bag: [Shape; 7],
    spawn_bag_len: usize,
    do_play: bool,
    pub is_dead: bool,
    opacity: i16,
    pub size: V2,
    seed: u32,
    is_p1: bool,
}

impl Board {
    pub fn new(seed: u32, is_p1: bool) -> Self {
        let size = V2::new(
            (1 + BOARD_WIDTH + 1 + 1 + 1) as f32,
            (1 + BOARD_HEIGHT + 1 + HoldLogic::SIZE.y as u8 + 1) as f32,
        );

        let garbage_bar_size = V2::new(1.0, BOARD_HEIGHT as f32);
        let mut obj = Self {
            is_cell_taken: Matrix::new(BOARD_WIDTH, BOARD_HEIGHT, false),
            current_agent: None,
            current_agent_shadow: None,
            garbage_bar: GarbageBar::new(
                V2::new((BOARD_WIDTH + 2) as f32, (size.y as u8 - 1 - BOARD_HEIGHT / 2) as f32),
                garbage_bar_size.clone(),
                Color::white().a(127).clone(),
            ),
            hold_logic: HoldLogic::new(V2::new(
                HoldLogic::SIZE.x / 2.0 + 1.0,
                size.y - 1.0 - BOARD_HEIGHT as f32 - 1.0 - HoldLogic::SIZE.y / 2.0,
            )),
            can_drop: true,
            static_buf: ColorMatrix::new(size.x as u8 * SCALE, size.y as u8 * SCALE, Color::none()),
            // Board area top-left in static_buf (SCALE× coords).
            // Derived from write() centering: final_x = x - W*S/2 + board_off_x*S
            // For BOARD_WIDTH=10 (even): board_area_x = SCALE
            board_area_x: SCALE,
            board_area_y: (size.y as u8 - 1 - BOARD_HEIGHT) * SCALE,
            garbage_hole: (SmallRng::from_seed([(seed % 255) as u8; core::mem::size_of::<SmallRng>()]).next_u32() % BOARD_WIDTH as u32)
                as u8,
            continue_dropping: true,
            drop_timer: 0.0,
            lock_delay_timer: 0.0,
            dropping_delay_value: DROPPING_DELAY_SEC,
            already_switched_pieces: false,
            spawn_bag: [Shape::I; 7],
            spawn_bag_len: 0,
            do_play: true,
            is_dead: false,
            opacity: 255,
            size: size.clone(),
            seed,
            is_p1,
        };

        obj.write_border(
            V2::new(0.0, size.y - 1.0),
            V2::new(BOARD_WIDTH as f32 + 1.0, size.y - 1.0 - BOARD_HEIGHT as f32 - 1.0),
        );

        obj.write_border(
            V2::new(BOARD_WIDTH as f32 + 1.0, size.y - 1.0),
            V2::new(
                BOARD_WIDTH as f32 + 1.0 + garbage_bar_size.x + 1.0,
                size.y - 1.0 - BOARD_HEIGHT as f32 - 1.0,
            ),
        );

        obj.write_border(
            V2::new(0.0, size.y - 1.0 - BOARD_HEIGHT as f32 - 1.0 - HoldLogic::SIZE.y - 1.0),
            V2::new(1.0 + HoldLogic::SIZE.x, size.y - 1.0 - BOARD_HEIGHT as f32 - 1.0),
        );

        obj.spawn(None, None);

        obj
    }

    pub fn tick(&mut self, input: &Box<dyn Input>, delta_time: f32) -> u8 {
        if !self.do_play {
            return 0;
        }

        self.garbage_bar.tick(delta_time);

        let mut damage_to_do = 0;

        if let Some(current_agent) = self.current_agent.as_mut() {
            if input.gestures().is(
                if self.is_p1 { Key::P1Blue } else { Key::P2Blue },
                State::Press,
                Gesture::Prolonged,
                None,
            ) && !self.already_switched_pieces
            {
                let held_shape = self.hold_logic.swap(current_agent.shape.clone());
                let center = current_agent.center.clone();
                self.spawn(Some(center), held_shape);
                self.already_switched_pieces = true;
            } else if input.gestures().is(
                if self.is_p1 { Key::P1Down } else { Key::P2Down },
                State::Press,
                Gesture::Once,
                None,
            ) {
                self.dropping_delay_value = FASTER_DROPPING_DELAY_SEC;
            } else {
                self.dropping_delay_value = DROPPING_DELAY_SEC;
                if input
                    .gestures()
                    .is(if self.is_p1 { Key::P1Left } else { Key::P2Left }, State::Down, Gesture::Once, None)
                    || input.gestures().is(
                        if self.is_p1 { Key::P1Left } else { Key::P2Left },
                        State::Press,
                        Gesture::Repeating,
                        None,
                    )
                {
                    self.move_block_by(V2::left());
                }

                if input.gestures().is(
                    if self.is_p1 { Key::P1Right } else { Key::P2Right },
                    State::Down,
                    Gesture::Once,
                    None,
                ) || input.gestures().is(
                    if self.is_p1 { Key::P1Right } else { Key::P2Right },
                    State::Press,
                    Gesture::Repeating,
                    None,
                ) {
                    self.move_block_by(V2::right());
                }
            }

            if input
                .gestures()
                .is(if self.is_p1 { Key::P1Up } else { Key::P2Up }, State::Down, Gesture::Once, None)
            {
                self.drop();
            }

            if input
                .gestures()
                .is(if self.is_p1 { Key::P1Blue } else { Key::P2Blue }, State::Down, Gesture::Once, None)
            {
                self.rotate_block(1);
            }

            if input.gestures().is(
                if self.is_p1 { Key::P1Green } else { Key::P2Green },
                State::Down,
                Gesture::Once,
                None,
            ) {
                self.rotate_block(-1);
            }

            let damage_to_deal = self.fall(delta_time);
            damage_to_do = self.garbage_bar.decrease_and_get_left(damage_to_deal);
        }

        if let Some(current_agent_shadow) = self.current_agent_shadow.as_mut()
            && let Some(current_agent) = self.current_agent.as_ref()
        {
            let center = current_agent.center;
            let current_spots = current_agent.get_taken_spots();
            current_agent_shadow.center = V2::new(center.x, center.y + Board::calc_drop(&self.is_cell_taken, 1, current_spots) as f32);
        }

        return damage_to_do;
    }

    /// Composites the board directly into `dst` (the actor's render matrix).
    /// static_buf (border + dropped blocks) is copied in one shot — never rebuilt per frame.
    /// Only active pieces are composited on top each frame.
    pub fn render_into(&mut self, dst: &mut ColorMatrix) {
        // One full-frame copy — replaces fill() + write(border) + write(dropped_blocks).
        dst.write_at_origin(&self.static_buf, &V2::zero());

        let si = SCALE as i16;
        // Garbage bar and hold: blit() instead of write() — no sin/cos, no float math.
        // Top-left = center*SCALE - matrix_size/2  (integer, matches write()'s ceilf behaviour)
        {
            let m = self.garbage_bar.render();
            let tlx = self.garbage_bar.center.x as i16 * si - m.width as i16 / 2;
            let tly = self.garbage_bar.center.y as i16 * si - m.height as i16 / 2;
            dst.blit(m, tlx, tly);
        }
        {
            let m = self.hold_logic.render();
            let tlx = self.hold_logic.center.x as i16 * si - m.width as i16 / 2;
            let tly = self.hold_logic.center.y as i16 * si - m.height as i16 / 2;
            dst.blit(m, tlx, tly);
        }

        // Active pieces: direct set() using same integer formula as get_taken_spots().
        // Avoids write()'s ceilf/sin/cos which misalign with integer board coords.
        let si = SCALE as i16;
        let bx = self.board_area_x as i16;
        let by = self.board_area_y as i16;
        let dw = dst.width as i16;
        let dh = dst.height as i16;

        if let Some(shadow) = &self.current_agent_shadow {
            let scaled = shadow.render_scaled();
            let cx = shadow.center.x as i16;
            let cy = shadow.center.y as i16;
            let wh = (shadow.matrix.width / 2) as i16;
            let hh = (shadow.matrix.height / 2) as i16;
            for my in 0..scaled.height as i16 {
                for mx in 0..scaled.width as i16 {
                    let c = *scaled.get(mx as u8, my as u8);
                    if c.a == 0 {
                        continue;
                    }
                    let px = (cx - wh) * si + mx + bx;
                    let py = (cy - hh) * si + my + by;
                    if px >= 0 && py >= 0 && px < dw && py < dh {
                        dst.set(px as u8, py as u8, c);
                    }
                }
            }
        }

        if let Some(agent) = &self.current_agent {
            let scaled = agent.render_scaled();
            let cx = agent.center.x as i16;
            let cy = agent.center.y as i16;
            let wh = (agent.matrix.width / 2) as i16;
            let hh = (agent.matrix.height / 2) as i16;
            for my in 0..scaled.height as i16 {
                for mx in 0..scaled.width as i16 {
                    let c = *scaled.get(mx as u8, my as u8);
                    if c.a == 0 {
                        continue;
                    }
                    let px = (cx - wh) * si + mx + bx;
                    let py = (cy - hh) * si + my + by;
                    if px >= 0 && py >= 0 && px < dw && py < dh {
                        dst.set(px as u8, py as u8, c);
                    }
                }
            }
        }
    }

    pub fn dim(&mut self, opacity: i16) {
        self.opacity = opacity;
    }

    fn spawn(&mut self, center: Option<V2>, shape: Option<Shape>) {
        self.already_switched_pieces = false;
        let new_shape = shape.unwrap_or_else(|| self.generate_block());
        let agent_center = center.unwrap_or(V2::new((BOARD_WIDTH / 2) as f32, 0.0));
        let new_agent = Block::new(agent_center, new_shape.clone(), false);
        let drop_pos = V2::new(
            new_agent.center.x,
            new_agent.center.y + Board::calc_drop(&self.is_cell_taken, 1, new_agent.get_taken_spots()) as f32,
        );
        self.current_agent = Some(new_agent);
        self.current_agent_shadow = Some(Block::new(drop_pos, new_shape, true));
    }

    fn move_block_by(&mut self, by: V2) {
        if let Some(current_agent) = self.current_agent.as_mut() {
            let spots = current_agent.get_taken_spots();

            if !spots
                .iter()
                .any(|f| Board::is_position_taken(&self.is_cell_taken, f.x as i16 + by.x as i16, f.y.max(0.0) as i16 + by.y as i16))
            {
                current_agent.center = &current_agent.center + &by;

                if let Some(current_agent_shadow) = self.current_agent_shadow.as_mut() {
                    current_agent_shadow.center = &current_agent.center
                        + &(V2::down() * Board::calc_drop(&self.is_cell_taken, 0, current_agent.get_taken_spots()) as f32);
                }
            }
        }
    }

    fn rotate_block(&mut self, dir: i32) {
        if let Some(current_agent) = self.current_agent.as_mut()
            && let Some(current_agent_shadow) = self.current_agent_shadow.as_mut()
        {
            let rotation = current_agent.rotation;
            let kicks = current_agent.get_kicks(rotation as i32 + 90 * dir);
            let pre_transform_center = current_agent.center.clone();
            current_agent.rotate_block(90 * dir);
            let post_transform_center = current_agent.center.clone();
            let mut did_kick = false;
            // let mut kick_attempts = 0;
            for kick in kicks {
                // kick_attempts += 1;
                current_agent.center = &post_transform_center + kick;
                let spots = current_agent.get_taken_spots();
                if !spots
                    .iter()
                    .any(|f| Board::is_position_taken(&self.is_cell_taken, f.x as i16, f.y as i16))
                {
                    did_kick = true;
                    break;
                }
            }
            // println!("kick attempts: {}", kick_attempts);
            if !did_kick {
                current_agent.rotate_block(-90 * dir);
                current_agent.center = pre_transform_center;
            } else {
                current_agent_shadow.rotate_block(90 * dir);
                current_agent_shadow.center = V2::new(
                    current_agent.center.x,
                    current_agent.center.y + Board::calc_drop(&self.is_cell_taken, 1, current_agent.get_taken_spots()) as f32,
                );
            }
        }
    }

    pub fn take_damage(&mut self, count: u8) {
        self.garbage_bar.add_lines(count);
    }

    fn generate_block(&mut self) -> Shape {
        let seed = self.seed;
        let mut rng = SmallRng::from_seed([(seed % 255) as u8; core::mem::size_of::<SmallRng>()]);

        if self.spawn_bag_len == 0 {
            self.spawn_bag = [Shape::I, Shape::O, Shape::T, Shape::S, Shape::Z, Shape::J, Shape::L];
            self.spawn_bag_len = 7;
            // Fisher-Yates shuffle
            for i in (1..7usize).rev() {
                let j = (rng.next_u32() as usize) % (i + 1);
                self.spawn_bag.swap(i, j);
            }
        }

        self.spawn_bag_len -= 1;
        self.spawn_bag[self.spawn_bag_len]
    }

    fn write_border(&mut self, from: V2, to: V2) {
        let start_x = from.x.min(to.x) as u8;
        let start_y = from.y.min(to.y) as u8;
        let end_x = from.x.max(to.x) as u8;
        let end_y = from.y.max(to.y) as u8;
        let s = SCALE as u8;
        for lx in start_x..=end_x {
            for ly in start_y..=end_y {
                if lx == start_x || lx == end_x || ly == start_y || ly == end_y {
                    for dy in 0..s {
                        for dx in 0..s {
                            self.static_buf.set(lx * s + dx, ly * s + dy, Color::white());
                        }
                    }
                }
            }
        }
    }

    /// Iterative drop calculation — avoids stack overflow risk of the recursive version.
    fn calc_drop(is_cell_taken: &Matrix<bool>, start: i16, spots: [V2; 4]) -> i16 {
        let mut i = start;
        loop {
            if spots
                .iter()
                .any(|f| Board::is_position_taken(is_cell_taken, f.x as i16, (f.y + i as f32) as i16))
            {
                return i - 1;
            }
            i += 1;
            if i > BOARD_HEIGHT as i16 + BOARD_WIDTH as i16 {
                return i - 1;
            }
        }
    }

    fn is_position_taken(is_cell_taken: &Matrix<bool>, x: i16, y: i16) -> bool {
        let a = x < 0;
        let b = x >= BOARD_WIDTH as i16;
        let c = y >= BOARD_HEIGHT as i16;
        let d = y < 0;
        a || b || c || d || *is_cell_taken.get(x as u8, y as u8)
    }

    fn fall(&mut self, delta_time: f32) -> u8 {
        let mut damage_to_deal = 0;
        if let Some(current_agent) = self.current_agent.as_mut()
            && (self.continue_dropping || self.lock_delay_timer > 0.0)
        {
            let mut dropped = false;
            let spots = current_agent.get_taken_spots();
            if spots
                .iter()
                .any(|f| Board::is_position_taken(&self.is_cell_taken, f.x as i16, f.y as i16 + 1))
            {
                self.lock_delay_timer += delta_time;
                if self.lock_delay_timer > LOCK_DELAY_SEC as f32 {
                    damage_to_deal = self.drop();
                    self.lock_delay_timer = 0.0;
                    dropped = true;
                }
            } else {
                current_agent.center.y += 1.0;
            }

            if !dropped {
                self.continue_dropping = false;
            }
        } else {
            self.drop_timer += delta_time;

            if self.drop_timer > self.dropping_delay_value {
                self.drop_timer = 0.0;
                self.continue_dropping = true;
            }
        }

        return damage_to_deal;
    }

    fn drop(&mut self) -> u8 {
        if let Some(current_agent) = self.current_agent.as_mut()
            && let Some(current_agent_shadow) = self.current_agent_shadow.as_mut()
            && self.can_drop
        {
            current_agent.center = current_agent_shadow.center.clone();

            let s = SCALE as u8;
            let color = BLOCKS_COLORS[current_agent.shape as usize];
            let bx = self.board_area_x;
            let by = self.board_area_y;
            for spot in current_agent.get_taken_spots() {
                self.is_cell_taken.set(spot.x as u8, spot.y as u8, true);
                let sx = spot.x as u8 * s + bx;
                let sy = spot.y as u8 * s + by;
                for dy in 0..s {
                    for dx in 0..s {
                        self.static_buf.set(sx + dx, sy + dy, color);
                    }
                }

                if spot.y <= 0.0 {
                    self.is_dead = true;
                    self.do_play = false;
                    return 0;
                }
            }

            self.can_drop = false;

            self.current_agent = None;
            self.current_agent_shadow = None;
            self.spawn(None, None);
            self.can_drop = true;

            let damage_to_deal = self.clear_lines();
            self.pop_garbage_lines();

            return damage_to_deal;
        }

        return 0;
    }

    fn clear_lines(&mut self) -> u8 {
        // Fixed-size buffer — no heap allocation
        let mut lines = [0u8; BOARD_HEIGHT as usize];
        let mut line_count = 0usize;

        for y in 0..self.is_cell_taken.height {
            let mut is_whole_line_taken = false;

            for x in 0..self.is_cell_taken.width {
                is_whole_line_taken = *self.is_cell_taken.get(x, y);
                if !is_whole_line_taken {
                    break;
                }
            }

            if is_whole_line_taken {
                lines[line_count] = y;
                line_count += 1;
            }
        }

        if line_count > 0 {
            let s = SCALE as u8;
            let bx = self.board_area_x;
            let by = self.board_area_y;
            for &line in &lines[..line_count] {
                for x in 0..self.is_cell_taken.width {
                    for dy in 0..s {
                        for dx in 0..s {
                            self.static_buf.set(x * s + bx + dx, line * s + by + dy, Color::none());
                        }
                    }
                }
            }

            for &line in &lines[..line_count] {
                for y in (1..=line as usize).rev() {
                    for x in 0..self.is_cell_taken.width {
                        let above = *self.is_cell_taken.get(x, (y - 1) as u8);
                        self.is_cell_taken.set(x, y as u8, above);
                        for dy in 0..s {
                            for dx in 0..s {
                                let c = *self.static_buf.get(x * s + bx + dx, (y - 1) as u8 * s + by + dy);
                                self.static_buf.set(x * s + bx + dx, y as u8 * s + by + dy, c);
                            }
                        }
                    }
                }
            }

            return line_count as u8;
        }

        return 0;
    }

    fn pop_garbage_lines(&mut self) {
        let hole = self.garbage_hole as i16;

        let s = SCALE as u8;
        let bx = self.board_area_x;
        let by = self.board_area_y;
        while self.garbage_bar.pop() {
            for x in 0..BOARD_WIDTH {
                for y in 0..BOARD_HEIGHT - 1 {
                    let val = *self.is_cell_taken.get(x, y + 1);
                    self.is_cell_taken.set(x, y, val);
                    for dy in 0..s {
                        for dx in 0..s {
                            let c = *self.static_buf.get(x * s + bx + dx, (y + 1) * s + by + dy);
                            self.static_buf.set(x * s + bx + dx, y * s + by + dy, c);
                        }
                    }
                }
            }

            for x in 0..BOARD_WIDTH {
                self.is_cell_taken.set(x, BOARD_HEIGHT - 1, x != hole as u8);
                let color = if x != hole as u8 {
                    Color::white().a(127).clone()
                } else {
                    Color::none()
                };
                for dy in 0..s {
                    for dx in 0..s {
                        self.static_buf.set(x * s + bx + dx, (BOARD_HEIGHT - 1) * s + by + dy, color);
                    }
                }
            }
        }
    }

    pub fn stop(&mut self) {
        self.do_play = false;
    }
}

pub fn create_board_actor(world: &mut World, tetris_world: &mut TetrisWorld, is_p1: bool, seed: u32) -> ActorId {
    let board = Board::new(seed, is_p1);

    let center = if is_p1 {
        V2::new(board.size.x, SCREEN_SIZE as f32 - board.size.y)
    } else {
        V2::new(SCREEN_SIZE as f32 - board.size.x - 1.0, board.size.y - 1.0)
    };

    let mut transform = Transform::new(center, board.size.clone());
    if !is_p1 {
        transform.rotation = 180.0;
    }

    let actor_id = world.add_new_actor(
        Some(transform),
        None,
        None,
        None,
        // Some(ColorMatrix::new(
        //     board.size.x as u8 * SCALE,
        //     board.size.y as u8 * SCALE,
        //     Color::none(),
        // )),
    );

    tetris_world.add_new_actor(actor_id, Some(board));

    actor_id
}
