extern crate alloc;
use alloc::{boxed::Box, vec::Vec};
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
        engine::{ActorId, SCREEN_SIZE, open_scene},
        hash_map::HashMap,
        input::{input::Input, key::Key},
        scene::Scene,
        v2::V2,
    },
    scenes::utils::{P1_COLOR, P2_COLOR, print_score, print_victory_text},
};

#[cfg(feature = "esp")]
use esp_println::println;

static ORIGINAL_BALL_SPEED: f32 = 10.0;
static ORIGINAL_BALL_SPEED_MULTIPLIER: f32 = 1.0;
static MAX_SCORE: u8 = 3;
static PADDLE_WIDTH: u8 = 5;
static PADDLE_SPEED: f32 = 20.0;
static MAX_BOUNCE_SPEED: f32 = 15.0;

pub struct PongScene {
    score: [u8; 2],
    paddle: [Option<ActorId>; 2],
    score_zone: [Option<ActorId>; 2],
    ball: Option<ActorId>,
    ball_speed: V2,
    ball_speed_multiplier: f32,
    size_factor: f32,
    can_score: bool,
    can_collide: [bool; 2],
    can_bounce: bool,
    do_play: bool,
    rng: SmallRng,
    play_against_ai: bool,

    data_for_ai: DataForAi,
}

impl Scene for PongScene {
    fn init(&mut self, world: &mut World) {
        let screen_size = SCREEN_SIZE as f32;
        let size_factor = screen_size / 32.0;
        self.paddle = [
            Some(create_rectangle_actor(
                world,
                V2::new(screen_size / 2.0, screen_size - 4.0 * size_factor),
                V2::new(PADDLE_WIDTH as f32, 1.0) * size_factor,
                // Color::white(),
                Some(ColliderType::Blocking),
                Some("paddle1"),
            )),
            Some(create_rectangle_actor(
                world,
                V2::new(screen_size / 2.0, 3.0 * size_factor),
                V2::new(PADDLE_WIDTH as f32, 1.0) * size_factor,
                // Color::white(),
                Some(ColliderType::Blocking),
                Some("paddle2"),
            )),
        ];
        self.score_zone = [
            Some(create_rectangle_actor(
                world,
                V2::new(screen_size / 2.0, -4.0 * size_factor),
                V2::new(screen_size, 10.0),
                // Color::none(),
                Some(ColliderType::Overlapping),
                Some("score_zone1"),
            )),
            Some(create_rectangle_actor(
                world,
                V2::new(screen_size / 2.0, screen_size + 4.0 * size_factor),
                V2::new(screen_size, 10.0),
                // Color::none(),
                Some(ColliderType::Overlapping),
                Some("score_zone2"),
            )),
        ];
        self.ball = Some(create_rectangle_actor(
            world,
            V2::one() * screen_size / 2.0,
            V2::one() * 2.0 * size_factor,
            // Color::white(),
            Some(ColliderType::Blocking),
            Some("ball"),
        ));

        // self.print_score(world);

        self.reset_ball(world);
    }

    fn tick(&mut self, inputs: [&Box<dyn Input>; 2], world: &mut World, delta_time: f32) {
        self.handle_input(inputs, world, delta_time);
        if let Some(ball) = self.ball
            && let Some(ball_transform) = world.get_mut_transform(&ball)
        {
            ball_transform.center += &self.ball_speed * delta_time;
            self.bounce_off_wall(world);
            self.check_scoring(world);
        }

        self.save_ai_data(world);
    }

    fn render(&mut self, camera: &Camera, world: &mut World, _delta_time: f32) -> ColorMatrix {
        let mut result = ColorMatrix::new(
            camera.get_viewport().get_size().x as u8,
            camera.get_viewport().get_size().y as u8,
            Color::none(),
        );

        for i in 0..2 {
            if camera.can_see_actor(self.paddle[i].unwrap(), world) {
                if let Some(transform) = world.get_mut_transform(&self.paddle[i].unwrap()) {
                    let color = if i == 0 { P1_COLOR } else { P2_COLOR };
                    result.write(
                        &ColorMatrix::new(transform.size.x as u8, transform.size.y as u8, color),
                        &transform.center,
                        None,
                        None,
                        None,
                        Some(camera),
                    );
                }
            }

            print_score(self.score[0], self.score[1], &mut result);

            if self.score.iter().any(|x| x == &MAX_SCORE) {
                self.do_play = false;
                print_victory_text(&mut result, if self.score[0] > self.score[1] { 1 } else { 2 }, camera, true);
                let play_against_ai = self.play_against_ai;
                add_asyncable(
                    Box::new(move |_, _| {
                        open_scene(Box::new(move || Box::new(PongScene::new(play_against_ai))));
                    }),
                    10.0,
                    AsyncableType::Timeout,
                );

                if let Some(ball_id) = self.ball {
                    world.murder(&ball_id);
                }
            }
        }

        if camera.can_see_actor(self.ball.unwrap(), world) {
            if let Some(transform) = world.get_mut_transform(&self.ball.unwrap()) {
                result.write(
                    &ColorMatrix::new(transform.size.x as u8, transform.size.y as u8, Color::white()),
                    &transform.center,
                    None,
                    None,
                    None,
                    Some(camera),
                );
            }
        }

        result
    }

    fn on_overlaps(&mut self, overlaps: &HashMap<ActorId, Vec<ActorId>>, world: &mut World, _delta_time: f32) {
        self.bounce_off_paddle(overlaps, world);
    }

    fn on_collisions(&mut self, _collisions: &HashMap<u16, Vec<(u16, CollisionResult)>>, _world: &mut World, _delta_time: f32) {}

    fn get_data_for_ai(&self) -> DataForAi {
        self.data_for_ai
    }

    fn is_game_over(&self) -> bool {
        self.current_scene.is_game_over()
    }
}

impl PongScene {
    pub fn new(play_against_ai: bool) -> Self {
        Self {
            score: [0, 0],
            paddle: [None, None],
            score_zone: [None, None],
            ball: None,
            ball_speed: V2::one() * ORIGINAL_BALL_SPEED,
            ball_speed_multiplier: ORIGINAL_BALL_SPEED_MULTIPLIER,
            size_factor: SCREEN_SIZE as f32 / 32.0,
            can_score: true,
            can_collide: [true, true],
            can_bounce: true,
            do_play: true,
            rng: SmallRng::seed_from_u64(embassy_time::Instant::now().as_micros()),
            play_against_ai: play_against_ai,

            data_for_ai: DataForAi {
                inputs: [Vec::new(), Vec::new()],
                points: [0.0, 0.0],
                is_gameover: false,
            },
        }
    }

    fn move_paddle(paddle: &ActorId, world: &mut World, delta: f32) {
        let transform = world.get_mut_transform(paddle).unwrap();
        let diff = V2::right() * delta;
        transform.center = &transform.center + &diff;
        let left = transform.center.x - transform.size.x / 2.0;
        let right = transform.center.x + transform.size.x / 2.0;
        if left < 0.0 {
            transform.center = V2::new(transform.size.x / 2.0, transform.center.y);
        }
        if right > SCREEN_SIZE as f32 {
            transform.center = V2::new(SCREEN_SIZE as f32 - transform.size.x / 2.0, transform.center.y);
        }
    }

    fn bounce_off_paddle(&mut self, overlaps: &HashMap<u16, Vec<u16>>, world: &mut World) {
        // TODO dobrze wiadomo kto bedzie kolejny odbijal pilke wiec nie ma sensu robic loopa tutaj
        for i in 0..2 {
            if overlaps.contains_key(&self.ball.unwrap()) && overlaps[&self.ball.unwrap()].contains(&self.paddle[i].unwrap()) {
                self.data_for_ai.points[i] += 1.0;
                self.can_collide[i] = false;
                self.can_bounce = true;
                let ball_transform = &world.get_transform(&self.ball.unwrap()).unwrap();
                let paddle_transform = &world.get_transform(&self.paddle[i].unwrap()).unwrap();
                let x_offset = (&ball_transform.center.x - &paddle_transform.center.x) / &paddle_transform.size.x * 2.0;
                let new_ball_speed = V2::new(
                    x_offset * MAX_BOUNCE_SPEED * self.size_factor,
                    ORIGINAL_BALL_SPEED * self.size_factor * self.ball_speed_multiplier * if i == 0 { -1.0 } else { 1.0 },
                );

                self.ball_speed = new_ball_speed;

                self.ball_speed_multiplier = f32::min(self.ball_speed_multiplier + 0.1, 5.0);
            }
        }
    }

    fn bounce_off_wall(&mut self, world: &mut World) {
        let screen_size = SCREEN_SIZE as f32;

        if self.can_bounce {
            let ball_transform = world.get_mut_transform(&self.ball.unwrap()).unwrap();
            if ball_transform.center.x + ball_transform.size.x / 2.0 >= screen_size {
                ball_transform.center.x = screen_size - ball_transform.size.x / 2.0 - 0.1;
                self.ball_speed.x *= -1.0;
                self.can_bounce = false;
                // TODO timeout
                // engine::set_timeout([this]() { canBounce = true; }, 1000);
            } else if ball_transform.center.x - ball_transform.size.x / 2.0 <= 0.0 {
                self.ball_speed.x *= -1.0;
                self.can_bounce = false;
                // ball_transform.center.x = 0.1;
                // TODO timeout
                // engine::set_timeout([this]() { canBounce = true; }, 1000);
            }
        }
    }

    fn check_scoring(&mut self, world: &mut World) {
        if !self.can_score {
            return;
        }

        let mut scored = false;
        let ball = world.get_transform(&self.ball.unwrap()).unwrap();
        let p1_zone = world.get_transform(&self.score_zone[0].unwrap()).unwrap();
        let p2_zone = world.get_transform(&self.score_zone[1].unwrap()).unwrap();

        if &ball.center.y < &p1_zone.center.y {
            self.score[1] += 1;
            self.data_for_ai.points[0] -= 10.0;
            scored = true;
        } else if &ball.center.y > &p2_zone.center.y {
            self.score[0] += 1;
            self.data_for_ai.points[1] -= 10.0;
            scored = true;
        }

        if scored {
            self.reset_ball(world);
        }
    }

    fn handle_input(&mut self, inputs: [&Box<dyn Input + 'static>; 2], world: &mut World, delta_time: f32) {
        // paddle[0] = P1_COLOR (bottom): P1 keys in pvp, AI in ai mode
        if self.play_against_ai {
            if let Some(ball_id) = self.ball
                && let Some(ball_transform) = world.get_transform(&ball_id)
                && let Some(paddle_p1_id) = self.paddle[0]
                && let Some(paddle_p1_transform) = world.get_transform(&paddle_p1_id)
            {
                PongScene::move_paddle(
                    &paddle_p1_id,
                    world,
                    if ball_transform.center.x < paddle_p1_transform.center.x - 1.0 {
                        -1.0
                    } else {
                        1.0
                    } * PADDLE_SPEED
                        * self.size_factor
                        * delta_time,
                );
            }
        } else if inputs[0].is_key_press(Key::Left) ^ inputs[0].is_key_press(Key::Right) {
            if let Some(paddle_p1_id) = self.paddle[0] {
                PongScene::move_paddle(
                    &paddle_p1_id,
                    world,
                    if inputs[0].is_key_press(Key::Left) { -1.0 } else { 1.0 } * PADDLE_SPEED * self.size_factor * delta_time,
                );
            }
        }

        // paddle[1] = P2_COLOR/orange (top): always P2 keys
        if inputs[1].is_key_press(Key::Left) ^ inputs[1].is_key_press(Key::Right) {
            if let Some(paddle_p2_id) = self.paddle[1] {
                PongScene::move_paddle(
                    &paddle_p2_id,
                    world,
                    if inputs[1].is_key_press(Key::Left) { -1.0 } else { 1.0 } * PADDLE_SPEED * self.size_factor * delta_time,
                );
            }
        }
    }

    fn reset_ball(&mut self, world: &mut World) {
        if let Some(ball_id) = self.ball
            && let Some(ball_transform) = world.get_mut_transform(&ball_id)
        {
            self.can_score = true;
            self.can_bounce = true;
            self.can_collide[0] = true;
            self.can_collide[1] = true;
            ball_transform.center = V2::one() * (SCREEN_SIZE / 2) as f32;
            if self.do_play {
                self.ball_speed_multiplier = ORIGINAL_BALL_SPEED_MULTIPLIER;
                self.ball_speed = V2::new(
                    //TODO
                    // 0.5,
                    self.rng.gen_range(0.0..1.0) * 2.0 * ORIGINAL_BALL_SPEED - ORIGINAL_BALL_SPEED,
                    if self.rng.gen_range(0.0..1.0) > 0.5 {
                        ORIGINAL_BALL_SPEED
                    } else {
                        -ORIGINAL_BALL_SPEED
                    },
                );
            } else {
                self.ball_speed_multiplier = 0.0;
                self.ball_speed = V2::zero();
            }
        }
    }

    fn save_ai_data(&mut self, world: &mut World) {
        if let Some(ball) = self.ball
            && let Some(ball_transform) = world.get_transform(&ball)
        {
            let mut p0_inputs = Vec::<f64>::new();
            if let Some(paddle0) = self.paddle[0]
                && let Some(paddle0_transform) = world.get_transform(&paddle0)
            {
                p0_inputs.push(paddle0_transform.center.x as f64);
                p0_inputs.push(ball_transform.center.x as f64);
                p0_inputs.push(ball_transform.center.y as f64);
            }

            let mut p1_inputs = Vec::<f64>::new();
            if let Some(paddle1) = self.paddle[1]
                && let Some(paddle1_transform) = world.get_transform(&paddle1)
            {
                p1_inputs.push(paddle1_transform.center.x as f64);
                p1_inputs.push(ball_transform.center.x as f64);
                p1_inputs.push(ball_transform.center.y as f64);
            }

            self.data_for_ai.inputs = [p0_inputs, p1_inputs];
        }
    }
}
