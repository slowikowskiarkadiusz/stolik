extern crate alloc;
use alloc::{boxed::Box, vec::Vec};
use libm::cosf;
use rand::{Rng, SeedableRng, rngs::SmallRng};

use crate::{
    engine::{
        actor::rectangle_actor::create_rectangle_actor,
        ai::neat_genome::DataForAi,
        asyncable::{AsyncableType, add_asyncable},
        color::Color,
        color_matrix::ColorMatrix,
        components::{
            camera::Camera,
            collider::{ColliderType, CollisionResult},
            world::World,
        },
        engine::{ActorId, SCREEN_SIZE, SCREEN_SIZEF32, open_scene},
        hash_map::HashMap,
        input::{
            input::Input,
            key::{KEYS_LENGTH, Key},
        },
        scene::Scene,
        v2::V2,
    },
    scenes::utils::{P1_COLOR, P2_COLOR, print_score, print_victory_text},
};

#[cfg(feature = "esp")]
use esp_println::println;

static MAX_SCORE: u8 = 3;
static BOARD_SIZE: u8 = 32;
static SIZE_FACTOR: f32 = (SCREEN_SIZE / BOARD_SIZE) as f32;
static INIT_SNAKE_LENGTH: u8 = 3;
static MOVEMENT_TIMER_SECONDS: f32 = 0.5;

struct SnakeNode {
    pub x: u8,
    pub y: u8,
    pub next: Option<Box<SnakeNode>>,
}

impl SnakeNode {
    pub fn get_all_points(&self) -> Vec<V2> {
        let mut result = Vec::<V2>::new();
        let mut maybe_current: &Option<Box<SnakeNode>> = &self.next;
        while let Some(current) = maybe_current {
            result.push(V2::new(current.x as f32, current.y as f32));
            maybe_current = &current.next
        }

        result
    }
}

pub struct SnakeScene {
    score: [u8; 2],
    snake: Vec<SnakeNode>,
    snake_timers: Vec<f32>,
    point_position: V2,
    do_play: bool,
    is_solo: bool,
    rng: SmallRng,
    data_for_ai: DataForAi,
}

fn create_initial_snake(is_p1: bool) -> SnakeNode {
    fn internal(is_p1: bool, offset: u8) -> SnakeNode {
        SnakeNode {
            x: BOARD_SIZE / 2 - offset,
            y: (if is_p1 { BOARD_SIZE / 2 + 10 } else { BOARD_SIZE / 2 - 10 }),
            next: if offset < INIT_SNAKE_LENGTH {
                Some(Box::new(internal(is_p1, offset + 1)))
            } else {
                None
            },
        }
    }

    internal(is_p1, 0)
}

impl Scene for SnakeScene {
    fn init(&mut self, world: &mut World) {
        self.snake.push(create_initial_snake(false));
        self.snake_timers.push(0.0);

        self.reset_point(world);
    }

    fn tick(&mut self, inputs: [&Box<dyn Input>; 2], world: &mut World, delta_time: f32) {
        self.handle_input(inputs, world, delta_time);

        self.save_ai_data(world);
    }

    fn render(&mut self, camera: &Camera, world: &mut World, _delta_time: f32) -> ColorMatrix {
        let mut result = ColorMatrix::new(BOARD_SIZE, BOARD_SIZE, Color::none());

        for i in 0..self.snake.len() {
            let color = if i == 0 { P1_COLOR } else { P2_COLOR };
            for point in self.snake[i].get_all_points() {
                result.set(point.x as u8, point.y as u8, color);
            }
        }

        print_score(self.score[0], self.score[1], &mut result);

        // if self.score.iter().any(|x| x == &MAX_SCORE) {
        //     self.do_play = false;
        //     print_victory_text(&mut result, if self.score[0] > self.score[1] { 1 } else { 2 }, camera, true);
        //     add_asyncable(
        //         Box::new(move |_, _| {
        //             open_scene(Box::new(|| Box::new(SnakeScene::new())), None);
        //         }),
        //         10.0,
        //         AsyncableType::Timeout,
        //     );

        //     if let Some(ball_id) = self.ball {
        //         world.murder(&ball_id);
        //     }
        // }

        // if camera.can_see_actor(self.ball.unwrap(), world) {
        //     if let Some(transform) = world.get_mut_transform(&self.ball.unwrap()) {
        //         result.write(
        //             &ColorMatrix::new(transform.size.x as u8, transform.size.y as u8, Color::white()),
        //             &transform.center,
        //             None,
        //             None,
        //             None,
        //             Some(camera),
        //         );
        //     }
        // }

        result
    }

    fn on_overlaps(&mut self, overlaps: &HashMap<ActorId, Vec<ActorId>>, world: &mut World, _delta_time: f32) {}

    fn on_collisions(&mut self, _collisions: &HashMap<u16, Vec<(u16, CollisionResult)>>, _world: &mut World, _delta_time: f32) {}

    fn get_data_for_ai(&self) -> DataForAi {
        DataForAi {
            inputs: self.data_for_ai.inputs.clone(),
            points: self.data_for_ai.points,
            is_gameover: self.data_for_ai.is_gameover,
            outputs_to_keys: snake_outputs_to_keys,
        }
    }

    fn is_game_over(&self) -> bool {
        !self.do_play
    }
}

impl SnakeScene {
    pub fn new(is_solo: bool) -> Self {
        Self {
            score: [0, 0],
            snake: alloc::vec![create_initial_snake(true)],
            snake_timers: alloc::vec![0.0],
            point_position: V2::zero(),
            do_play: true,
            is_solo: is_solo,
            rng: SmallRng::seed_from_u64(embassy_time::Instant::now().as_micros()),
            data_for_ai: DataForAi {
                inputs: [Vec::new(), Vec::new()],
                points: [0.0, 0.0],
                is_gameover: false,
                outputs_to_keys: snake_outputs_to_keys,
            },
        }
    }

    /// returns index of snake that lost, otherwise None
    fn check_if_collision(&self) -> Option<usize> {
        for i in 0..self.snake.len() {
            let head_pos = V2::new(self.snake[i].x as f32, self.snake[i].y as f32);
            for j in 0..self.snake.len() {
                if i != j {
                    if self.snake[j]
                        .get_all_points()
                        .iter()
                        .any(|f| f.x == head_pos.x && f.y == head_pos.y)
                    {
                        return Some(i);
                    }
                }
            }
        }

        None
    }

    fn move_snake(head: &mut SnakeNode, by: V2) {
        fn internal(snake_node: &mut SnakeNode, parent_x: u8, parent_y: u8) {
            if let Some(next) = snake_node.next.as_mut() {
                internal(next, snake_node.x, snake_node.y);
            }

            snake_node.x = parent_x;
            snake_node.y = parent_y;
        }

        head.x += by.x as u8;
        head.y += by.y as u8;

        if let Some(next) = head.next.as_mut() {
            internal(next, head.x, head.y);
        }
    }

    fn handle_input(&mut self, inputs: [&Box<dyn Input + 'static>; 2], world: &mut World, delta_time: f32) {
        for i in 0..self.snake.len() {
            if self.snake_timers[i] <= 0.0 && inputs[i].is_key_press(Key::AnyDirection) {
                let mul = if i == 0 { 1.0 } else { -1.0 };

                let by = if inputs[i].is_key_press(Key::Left) {
                    V2::new(-1.0 * mul, 0.0)
                } else if inputs[i].is_key_press(Key::Right) {
                    V2::new(1.0 * mul, 0.0)
                } else if inputs[i].is_key_press(Key::Up) {
                    V2::new(0.0, -1.0 * mul)
                } else if inputs[i].is_key_press(Key::Down) {
                    V2::new(0.0, 1.0 * mul)
                } else {
                    V2::zero()
                };

                SnakeScene::move_snake(&mut self.snake[i], by);

                self.snake_timers[i] = MOVEMENT_TIMER_SECONDS;
            }

            self.snake_timers[i] = self.snake_timers[i] - delta_time;
        }
    }

    fn reset_point(&mut self, world: &mut World) {
        let mut all_taken_points: Vec<V2> = self.snake.iter().flat_map(|f| f.get_all_points()).collect();

        loop {
            let mut point = V2::new(self.rng.gen_range(0..BOARD_SIZE) as f32, self.rng.gen_range(0..BOARD_SIZE) as f32);

            if !all_taken_points.contains(&point) {
                self.point_position = point;
                return;
            }
        }
    }

    fn save_ai_data(&mut self, world: &mut World) {
        // if let Some(ball) = self.ball
        //     && let Some(ball_transform) = world.get_transform(&ball)
        // {
        //     let mut p0_inputs = Vec::<f64>::new();
        //     if let Some(paddle0) = self.paddle[0]
        //         && let Some(paddle0_transform) = world.get_transform(&paddle0)
        //     {
        //         p0_inputs.push(paddle0_transform.center.x as f64);
        //         p0_inputs.push(ball_transform.center.x as f64);
        //         p0_inputs.push(ball_transform.center.y as f64);
        //     }

        //     let mut p1_inputs = Vec::<f64>::new();
        //     if let Some(paddle1) = self.paddle[1]
        //         && let Some(paddle1_transform) = world.get_transform(&paddle1)
        //     {
        //         p1_inputs.push(paddle1_transform.center.x as f64);
        //         p1_inputs.push(ball_transform.center.x as f64);
        //         p1_inputs.push(ball_transform.center.y as f64);
        //     }

        //     self.data_for_ai.inputs = [p0_inputs, p1_inputs];
        // }
    }
}

// fn pong_outputs_to_keys(outputs: &[f64]) -> [bool; KEYS_LENGTH as usize] {
//     let mut keys = [false; KEYS_LENGTH as usize];
//     if let Some(&v) = outputs.first() {
//         keys[2] = v > 0.5; // Left
//         keys[3] = v < 0.5; // Right
//     }
//     keys
// }

fn snake_outputs_to_keys(outputs: &[f64]) -> [bool; KEYS_LENGTH as usize] {
    let mut keys = [false; KEYS_LENGTH as usize];
    if let Some(&v) = outputs.first() {
        keys[2] = v > 0.5; // Left
        keys[3] = v < 0.5; // Right
    }
    keys
}
