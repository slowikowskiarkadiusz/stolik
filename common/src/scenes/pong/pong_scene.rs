extern crate alloc;
use alloc::{boxed::Box, string::ToString, vec::Vec};
use rand::{Rng, SeedableRng, rngs::SmallRng};

use crate::{
    engine::{
        actor::{
            rectangle_actor::create_rectangle_actor,
            text::{LETTER_HEIGHT, MAX_LETTER_WIDTH, create_text_actor_at_center},
        },
        asyncable::{AsyncableType, add_asyncable},
        color::Color,
        components::{collider::ColliderType, world::World},
        engine::{ActorId, SCREEN_SIZE, open_scene},
        hash_map::HashMap,
        input::{input::Input, key::Key},
        scene::Scene,
        v2::V2,
    },
    scenes::utils::print_victory_text,
};

static ORIGINAL_BALL_SPEED: f32 = 7.0;
static ORIGINAL_BALL_SPEED_MULTIPLIER: f32 = 1.0;
static MAX_SCORE: u8 = 1;

pub struct PongScene {
    score: [u8; 2],
    paddle: [Option<ActorId>; 2],
    score_zone: [Option<ActorId>; 2],
    score_text: [Option<ActorId>; 2],
    ball: Option<ActorId>,
    paddle_speed: f32,
    max_bounce_speed: f32,
    ball_speed: V2,
    ball_speed_multiplier: f32,
    size_factor: f32,
    can_score: bool,
    can_collide: [bool; 2],
    can_bounce: bool,
    do_play: bool,
    rng: SmallRng,
}

impl Scene for PongScene {
    fn init(&mut self, world: &mut World) {
        let screen_size = SCREEN_SIZE as f32;
        let size_factor = screen_size / 32.0;
        self.paddle = [
            Some(create_rectangle_actor(
                world,
                V2::new(screen_size / 2.0, 3.0 * size_factor),
                V2::new(7.0, 1.0) * size_factor,
                Color::white(),
                Some(ColliderType::Overlapping),
                Some("paddle1"),
            )),
            Some(create_rectangle_actor(
                world,
                V2::new(screen_size / 2.0, screen_size - 4.0 * size_factor),
                V2::new(7.0, 1.0) * size_factor,
                Color::white(),
                Some(ColliderType::Overlapping),
                Some("paddle2"),
            )),
        ];
        self.score_zone = [
            Some(create_rectangle_actor(
                world,
                V2::new(screen_size / 2.0, -4.0 * size_factor),
                V2::new(screen_size, 10.0),
                Color::none(),
                Some(ColliderType::Overlapping),
                Some("score_zone1"),
            )),
            Some(create_rectangle_actor(
                world,
                V2::new(screen_size / 2.0, screen_size + 4.0 * size_factor),
                V2::new(screen_size, 10.0),
                Color::none(),
                Some(ColliderType::Overlapping),
                Some("score_zone2"),
            )),
        ];
        self.ball = Some(create_rectangle_actor(
            world,
            V2::one() * screen_size / 2.0,
            V2::one() * 2.0 * size_factor,
            Color::white(),
            Some(ColliderType::Overlapping),
            Some("ball"),
        ));

        self.print_score(world);

        self.reset_ball(world);
    }

    fn tick(&mut self, input: &Box<dyn Input>, world: &mut World, delta_time: f32) {
        self.handle_input(input, world, delta_time);
        if let Some(ball) = self.ball
            && let Some(ball_transform) = world.get_mut_transform(&ball)
        {
            ball_transform.center += &self.ball_speed * delta_time;
            self.bounce_off_wall(world);
            self.check_scoring(world);
        }
    }

    fn on_overlaps(&mut self, overlaps: &HashMap<ActorId, Vec<ActorId>>, world: &mut World, _delta_time: f32) {
        self.bounce_off_paddle(overlaps, world);
    }
}

impl PongScene {
    pub fn new() -> Self {
        Self {
            score: [0, 0],
            paddle: [None, None],
            score_zone: [None, None],
            score_text: [None, None],
            ball: None,
            paddle_speed: 15.0,
            max_bounce_speed: 10.0,
            ball_speed: V2::one() * ORIGINAL_BALL_SPEED,
            ball_speed_multiplier: ORIGINAL_BALL_SPEED_MULTIPLIER,
            size_factor: SCREEN_SIZE as f32 / 32.0,
            can_score: true,
            can_collide: [true, true],
            can_bounce: true,
            do_play: true,
            rng: SmallRng::seed_from_u64(embassy_time::Instant::now().as_micros()),
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
                self.can_collide[i] = false;
                self.can_bounce = true;
                let ball_transform = &world.get_transform(&self.ball.unwrap()).unwrap();
                let paddle_transform = &world.get_transform(&self.paddle[i].unwrap()).unwrap();
                let x_offset = (&ball_transform.center.x - &paddle_transform.center.x) / &paddle_transform.size.x;
                let new_ball_speed = V2::new(
                    x_offset * self.max_bounce_speed * self.size_factor,
                    ORIGINAL_BALL_SPEED * self.size_factor * self.ball_speed_multiplier * if i == 0 { 1.0 } else { -1.0 },
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
            scored = true;
        } else if &ball.center.y > &p2_zone.center.y {
            self.score[0] += 1;
            scored = true;
        }

        if self.score.iter().any(|x| x == &MAX_SCORE) {
            self.do_play = false;
            print_victory_text(world, if self.score[0] > self.score[1] { 1 } else { 2 });
            add_asyncable(
                Box::new(|_, _| {
                    open_scene(Box::new(|| Box::new(PongScene::new())));
                }),
                10.0,
                AsyncableType::Timeout,
            );

            if let Some(ball_id) = self.ball {
                world.remove_actor(&ball_id);
            }
        }

        if scored {
            self.print_score(world);
            self.reset_ball(world);
        }
    }

    fn handle_input(&mut self, input: &Box<dyn Input + 'static>, world: &mut World, delta_time: f32) {
        if input.is_key_press(Key::P1Left) ^ input.is_key_press(Key::P1Right) {
            if let Some(paddle_p1_id) = self.paddle[0] {
                PongScene::move_paddle(
                    &paddle_p1_id,
                    world,
                    if input.is_key_press(Key::P1Left) { -1.0 } else { 1.0 } * self.paddle_speed * self.size_factor * delta_time,
                );
            }
        }

        if input.is_key_press(Key::P2Left) ^ input.is_key_press(Key::P2Right) {
            if let Some(paddle_p2_id) = self.paddle[1] {
                PongScene::move_paddle(
                    &paddle_p2_id,
                    world,
                    if input.is_key_press(Key::P2Left) { -1.0 } else { 1.0 } * self.paddle_speed * self.size_factor * delta_time,
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

    fn print_score(&mut self, world: &mut World) {
        for i in 0..2 {
            if let Some(score_text_actor) = self.score_text[i] {
                world.remove_actor(&score_text_actor);
            }

            let score_text_actor = create_text_actor_at_center(
                world,
                self.score[i].to_string(),
                V2::new(5.0, SCREEN_SIZE as f32 / 2.0 + (if i == 0 { -5.0 } else { 3.0 })),
                V2::new(MAX_LETTER_WIDTH as f32, LETTER_HEIGHT as f32),
                Color::blue().a(127).clone(),
                None,
                Some("score_text"),
            );

            if let Some(a) = world.get_mut_render(&score_text_actor) {
                a.rotate(-90.0, Color::none());
            }

            self.score_text[i] = Some(score_text_actor);
        }
    }
}
