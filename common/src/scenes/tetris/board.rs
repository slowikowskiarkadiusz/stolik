extern crate alloc;

use alloc::{boxed::Box, sync::Arc, vec::Vec};
use rand::RngCore;
use rand::seq::SliceRandom;
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
    scenes::tetris::{
        block::Block,
        garbage_bar::GarbageBar,
        hold_logic::HoldLogic,
        shape::{Shape, get_shape},
    },
};

const BOARD_WIDTH: u8 = 10;
const BOARD_HEIGHT: u8 = 20;
const AFTER_DROP_DELAY: u32 = 250;
const LINE_CLEARING_ANIMATION_FACTOR: u32 = 75;
const DROPPING_DELAY: f32 = 1000.0;
const FASTER_DROPPING_DELAY: f32 = 100.0;
const LOCK_DELAY: u32 = 1000;
pub const BLOCKS_COLORS: &[Color; 7] = &[
    Color::new(0, 255, 255, 0), // Cyan
    Color::new(255, 255, 0, 0), // Yellow
    Color::new(127, 0, 127, 0), // Purple
    Color::new(0, 255, 0, 0),   // Green
    Color::new(255, 0, 0, 0),   // Red
    Color::new(0, 0, 255, 0),   // Blue
    Color::new(255, 163, 0, 0), // Orange
];

pub struct Board {
    is_cell_taken: Matrix<bool>,
    // current_block_index: u32,
    current_agent: Option<Block>,
    current_agent_shadow: Option<Block>,
    garbage_bar: GarbageBar,
    hold_logic: HoldLogic,
    can_drop: bool,
    dropped_blocks_matrix: ColorMatrix,
    border_matrix: ColorMatrix,
    continue_dropping: bool,
    drop_timer: f32,
    lock_delay_timer: f32,
    dropping_delay_value: f32,
    already_switched_pieces: bool,
    spawn_bag: Vec<Shape>,
    do_play: bool,
    pub is_dead: bool,
    opacity: u8,
    // center: V2,
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
        let obj = Self {
            is_cell_taken: Matrix::new(BOARD_WIDTH, BOARD_HEIGHT, false),
            // current_block_index: 0,
            current_agent: None,
            current_agent_shadow: None,
            garbage_bar: GarbageBar::new(
                V2::new((BOARD_WIDTH + 2) as f32, (size.y as u8 - 1 - BOARD_HEIGHT / 2) as f32),
                V2::new(1.0, BOARD_HEIGHT as f32),
                Color::white().a(127).clone(),
            ),
            hold_logic: HoldLogic::new(V2::zero()),
            can_drop: false,
            dropped_blocks_matrix: ColorMatrix::new(BOARD_WIDTH, BOARD_HEIGHT, Color::none()),
            border_matrix: ColorMatrix::new(size.x as u8, size.y as u8, Color::none()),
            continue_dropping: true,
            drop_timer: 0.0,
            lock_delay_timer: 0.0,
            dropping_delay_value: DROPPING_DELAY,
            already_switched_pieces: false,
            spawn_bag: Vec::new(),
            do_play: true,
            is_dead: true,
            opacity: 255,
            // center,
            size,
            seed,
            is_p1,
        };

        obj
    }

    pub fn tick(&mut self, input: &Box<dyn Input>, delta_time: f32) -> u8 {
        if !self.do_play {
            return 0;
        }

        self.garbage_bar.tick(delta_time);

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
                self.dropping_delay_value = FASTER_DROPPING_DELAY;
            } else {
                self.dropping_delay_value = DROPPING_DELAY;
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
                return self.garbage_bar.decrease_and_get_left(damage_to_deal);
            }

            return 0;
        }

        return 0;
    }

    pub fn render(&mut self) -> ColorMatrix {
        let mut render_matrix = ColorMatrix::new(self.size.x as u8, self.size.y as u8, Color::none());

        render_matrix.write_at_origin(&self.border_matrix, &V2::zero());
        render_matrix.write(&self.garbage_bar.render(), &self.garbage_bar.center, None, None, None);
        render_matrix.write(&self.hold_logic.render(), &self.hold_logic.center, None, None, None);

        // matrix boardMatrix(board_width + 1, board_height);
        // if (current_agent_shadow)
        //   boardMatrix.write(current_agent_shadow->render(),
        //                     current_agent_shadow->center, 0);
        // if (current_agent) {
        //   boardMatrix.write(current_agent->render(), current_agent->center, 0);
        // }

        if let Some(current_agent_shadow) = &self.current_agent_shadow {
            render_matrix.write(&current_agent_shadow.render(), &current_agent_shadow.center, None, None, None);
        }

        if let Some(current_agent) = &self.current_agent {
            render_matrix.write(&current_agent.render(), &current_agent.center, None, None, None);
        }

        let board_offset = V2::new(
            (1 + BOARD_WIDTH / 2) as f32,
            self.size.y - 1 as f32 - (BOARD_HEIGHT + BOARD_HEIGHT) as f32 / 2.0,
        );

        render_matrix.write(&self.dropped_blocks_matrix, &board_offset, None, None, None);

        render_matrix.scale((SCREEN_SIZE / 32) as f32, Color::none());
        render_matrix.dim(self.opacity as u8);

        render_matrix
    }

    pub fn dim(&mut self, opacity: u8) {
        self.opacity = opacity;
    }

    fn spawn(&mut self, center: Option<V2>, shape: Option<Shape>) {
        self.already_switched_pieces = false;
        let new_shape = shape.unwrap_or(self.generate_block(self.seed.clone()));
        let agent_center = center.unwrap_or(V2::new((BOARD_WIDTH / 2) as f32, 0.0));
        let new_agent = Block::new(agent_center, new_shape.clone(), false);
        let drop_pos = V2::new(
            new_agent.center.x,
            new_agent.center.y + Board::calc_drop(&self.is_cell_taken, 0, &new_agent) as f32,
        );
        self.current_agent = Some(new_agent);
        self.current_agent_shadow = Some(Block::new(drop_pos, new_shape, true));
    }

    fn move_block_by(&mut self, by: V2) {
        if let Some(current_agent) = self.current_agent.as_mut() {
            let spots = current_agent.get_taken_spots();

            if !spots
                .iter()
                .any(|f| Board::is_position_taken(&self.is_cell_taken, f.x as i8, f.y as i8))
            {
                current_agent.center = &current_agent.center + &by;

                if let Some(current_agent_shadow) = self.current_agent_shadow.as_mut() {
                    current_agent_shadow.center =
                        &current_agent.center + &(V2::down() * Board::calc_drop(&self.is_cell_taken, 0, &current_agent) as f32);
                }
            }
        }
    }

    fn rotate_block(&mut self, dir: i32) {
        if let Some(current_agent) = self.current_agent.as_mut()
            && let Some(current_agent_shadow) = self.current_agent_shadow.as_mut()
        {
            let rotation = current_agent.rotation;
            let kicks = current_agent.get_kicks(rotation as i32 + 90i32 * dir);
            let pre_transform_center = current_agent.center.clone();
            current_agent.rotate_block(0i32);
            let post_transform_center = current_agent.center.clone();
            let mut did_kick = false;
            for kick in kicks {
                current_agent.center = &post_transform_center + &kick;
                let spots = current_agent.get_taken_spots();
                if !spots
                    .iter()
                    .any(|f| Board::is_position_taken(&self.is_cell_taken, f.x as i8, f.y as i8))
                {
                    did_kick = true;
                    break;
                }
            }

            if !did_kick {
                current_agent.rotate_block(90);
                current_agent.center = pre_transform_center;
            } else {
                current_agent_shadow.rotate_block(90 * dir);
                current_agent_shadow.center = V2::new(
                    current_agent.center.x,
                    current_agent.center.y + Board::calc_drop(&self.is_cell_taken, 0, &current_agent) as f32,
                );
            }
        }
    }

    pub fn take_damage(&mut self, count: u8) {
        self.garbage_bar.add_lines(count);
    }

    fn generate_block(&mut self, seed: u32) -> Shape {
        let mut rng = SmallRng::from_seed([(seed % 255) as u8; 32]);
        if self.spawn_bag.is_empty() {
            for i in 0..(Shape::L as u8 + 1) {
                self.spawn_bag.push(get_shape(i));
            }

            self.spawn_bag.shuffle(&mut rng);
        }

        self.spawn_bag.pop().unwrap()
    }

    fn write_border(&mut self, from: V2, to: V2) {
        let start_x = from.x.min(to.x) as u8;
        let start_y = from.y.min(to.y) as u8;
        let end_x = from.x.max(to.x) as u8;
        let end_y = from.y.max(to.y) as u8;
        for x in start_x..=end_x {
            for y in start_y..=end_y {
                if x == start_x || x == end_x || y == start_y || y == end_y {
                    self.border_matrix.set(x, y, Color::white());
                }
            }
        }
    }

    fn calc_drop(is_cell_taken: &Matrix<bool>, i: u8, agent: &Block) -> u8 {
        let spots = agent.get_taken_spots();
        if !spots.iter().any(|f| Board::is_position_taken(is_cell_taken, f.x as i8, f.y as i8)) {
            let next = Board::calc_drop(is_cell_taken, i + 1, agent);
            return if next != 0 { next } else { i };
        } else {
            i - 1
        }
    }

    fn is_position_taken(is_cell_taken: &Matrix<bool>, x: i8, y: i8) -> bool {
        let a = x < 0;
        let b = x >= BOARD_WIDTH as i8;
        let c = y >= BOARD_HEIGHT as i8;
        let d = a || b || c || is_cell_taken.get(x as u8, y as u8).clone();

        a || b || c || d
    }

    fn fall(&mut self, delta_time: f32) -> u8 {
        let mut damage_to_deal = 0;
        if let Some(current_agent) = self.current_agent.as_mut()
            && self.continue_dropping
            && self.lock_delay_timer > 0.0
        {
            let mut dropped = false;
            let spots = current_agent.get_taken_spots();
            if !spots
                .iter()
                .any(|f| Board::is_position_taken(&self.is_cell_taken, f.x as i8, f.y as i8))
            {
                self.lock_delay_timer += delta_time;
                if self.lock_delay_timer > LOCK_DELAY as f32 {
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

            for spot in current_agent.get_taken_spots() {
                self.is_cell_taken.set(spot.x as u8, spot.y as u8, true);
                self.dropped_blocks_matrix.set(
                    spot.x as u8,
                    spot.y as u8,
                    BLOCKS_COLORS[current_agent.shape.clone() as usize].clone(),
                );

                if spot.y <= 0.0 {
                    self.is_dead = true;
                    self.do_play = false;
                    return 0;
                }
            }

            self.can_drop = false;

            // TODO animation
            let damage_to_deal = self.clear_lines();
            self.pop_garbage_lines();

            return damage_to_deal;
        }

        return 0;
    }

    fn clear_lines(&mut self) -> u8 {
        let mut lines = Vec::<u8>::new();
        for y in 0..self.is_cell_taken.height {
            let mut is_whole_line_taken = false;

            for x in 0..self.is_cell_taken.width {
                is_whole_line_taken = self.is_cell_taken.get(x, y).clone();
                if !is_whole_line_taken {
                    break;
                }
            }

            if is_whole_line_taken {
                lines.push(y);
            }
        }

        if !lines.is_empty() {
            for line in &lines {
                for x in 0..self.is_cell_taken.width {
                    self.dropped_blocks_matrix.set(x, line.clone(), Color::none());
                    // TODO
                    // wait
                }
            }

            let lines_length = lines.len();

            for line in lines {
                for y in line..0 {
                    for x in 0..self.is_cell_taken.width {
                        self.is_cell_taken.set(x, y, self.is_cell_taken.get(x, y - 1).clone());
                        self.dropped_blocks_matrix
                            .set(x, line, self.dropped_blocks_matrix.get(x, y - 1).clone());
                        // TODO
                        // wait
                    }
                }
            }

            return lines_length as u8;
        }

        return 0;
    }

    fn pop_garbage_lines(&mut self) {
        let hole = (SmallRng::from_seed([(self.seed % 255) as u8; 32]).next_u32() % BOARD_WIDTH as u32) as u8;

        while self.garbage_bar.pop() {
            for x in 0..BOARD_WIDTH {
                for y in 0..BOARD_HEIGHT - 1 {
                    self.is_cell_taken.set(x, y, self.is_cell_taken.get(x, y + 1).clone());
                    self.dropped_blocks_matrix
                        .set(x, y, self.dropped_blocks_matrix.get(x, y + 1).clone());
                }
            }

            for x in 0..BOARD_WIDTH {
                self.is_cell_taken.set(x, BOARD_HEIGHT - 1, x != hole as u8);
                self.dropped_blocks_matrix.set(
                    x,
                    BOARD_HEIGHT - 1,
                    if x != hole as u8 {
                        Color::white().a(127).clone()
                    } else {
                        Color::none()
                    },
                );
            }

            // TODO
            // wait
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
        V2::new(SCREEN_SIZE as f32 - board.size.x, board.size.y)
    };

    let mut transform = Transform::new(center, board.size.clone());
    if !is_p1 {
        transform.rotation = 180.0;
    }

    let actor_id = world.add_new_actor(
        Some("board actor"),
        Some(transform),
        None,
        None,
        None,
        Some(ColorMatrix::new(board.size.x as u8, board.size.y as u8, Color::none())),
    );

    tetris_world.add_new_actor(actor_id, Some(board));

    actor_id
}
