use std::sync::{Arc, Mutex};

use crate::engine::{
    actor::{actor::TActor, rectangle_actor::RectangleActor},
    color::Color,
    engine::{ActorId, EngineView, SCREEN_SIZE},
    input::key::Key,
    scene::Scene,
    v2::V2,
};

pub struct PongScene {
    actors: Vec<Box<dyn TActor>>,
    score: (u8, u8),
    paddle: (Option<ActorId>, Option<ActorId>),
    score_zone: (Option<ActorId>, Option<ActorId>),
    ball: Option<ActorId>,
    paddle_speed: f32,
    max_bounce_speed: f32,
    original_ball_speed: f32,
    ball_speed: V2,
    ball_speed_multiplier: f32,
    size_factor: f32,
    can_score: bool,
    can_collide: (bool, bool),
    can_bounce: bool,
    do_play: bool,
}

impl PongScene {
    pub fn new(engine: &EngineView) -> Self {
        let size_factor = SCREEN_SIZE as f32 / 32.0;
        Self {
            actors: Vec::new(),
            score: (0, 0),
            paddle: (None, None),
            score_zone: (None, None),
            ball: None,
            paddle_speed: 3.0,
            max_bounce_speed: 0.03,
            original_ball_speed: 0.007,
            ball_speed: V2::one(),
            ball_speed_multiplier: 1.0,
            size_factor: 1.0,
            can_score: true,
            can_collide: (true, true),
            can_bounce: true,
            do_play: true,
        }
    }

    fn move_paddle(paddle: &mut RectangleActor, delta: f32) {
        paddle.move_by(V2::right() * delta);
        let left = paddle.get_center().x - paddle.get_size().x / 2.0;
        let right = paddle.get_center().x + paddle.get_size().x / 2.0;
        if left < 0.0 {
            paddle.set_center(V2::new(paddle.get_size().x / 2.0, paddle.get_center().y));
        }
        if right > SCREEN_SIZE as f32 {
            paddle.set_center(V2::new(
                SCREEN_SIZE as f32 - paddle.get_size().x / 2.0,
                paddle.get_center().y,
            ));
        }
    }

    fn handle_input(&self, engine: &EngineView) {
        let input = engine.1;
        if input.is_key_press(Key::P1Left) ^ input.is_key_press(Key::P1Right) {
            let paddle = &mut self.paddle.0.lock().unwrap();
            PongScene::move_paddle(
                paddle,
                if input.is_key_press(Key::P1Left) {
                    -1.0
                } else {
                    1.0
                } * self.paddle_speed
                    * self.size_factor
                    * engine.0,
            );
        }

        if input.is_key_press(Key::P2Left) ^ input.is_key_press(Key::P2Right) {
            let paddle = &mut self.paddle.1.lock().unwrap();
            PongScene::move_paddle(
                paddle,
                if input.is_key_press(Key::P2Left) {
                    -1.0
                } else {
                    1.0
                } * self.paddle_speed
                    * self.size_factor
                    * engine.0,
            );
        }
    }
}

impl Scene for PongScene {
    fn get_actors(&self) -> &[Box<dyn TActor>] {
        &self.actors
    }

    fn init(&mut self) {
        let size_factor = SCREEN_SIZE as f32 / 32.0;
        let paddle = (
            RectangleActor::new(
                V2::new(SCREEN_SIZE as f32 / 2.0, 3.0 * size_factor),
                &V2::new(7.0, 1.0),
                Color::white(),
                Some(String::from("paddle1")),
            ),
            RectangleActor::new(
                V2::new(
                    SCREEN_SIZE as f32 / 2.0,
                    SCREEN_SIZE as f32 - 4.0 * size_factor,
                ),
                &V2::new(7.0, 1.0),
                Color::white(),
                Some(String::from("paddle2")),
            ),
        );
        let score_zone = (
            RectangleActor::new(
                V2::zero(),
                &V2::one(),
                Color::white(),
                Some(String::from("score_zone1")),
            ),
            RectangleActor::new(
                V2::zero(),
                &V2::one(),
                Color::white(),
                Some(String::from("score_zone2")),
            ),
        );
        let ball = RectangleActor::new(
            V2::zero(),
            &V2::one(),
            Color::white(),
            Some(String::from("ball")),
        );


    }

    fn update(&self, engine: &EngineView) {
        self.handle_input(engine);
    }
}
