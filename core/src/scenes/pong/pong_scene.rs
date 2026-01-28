use crate::engine::{
    actor::{
        actor::{InnerActor, TActor},
        rectangle_actor::RectangleActor,
    }, color::Color, engine::{ActorId, Engine, SCREEN_SIZE}, input::key::Key, scene::Scene, v2::{self, V2}
};

pub struct PongScene {
    actors: Vec<Box<dyn TActor>>,
    score: (u8, u8),
    paddle: (ActorId, ActorId),
    score_zone: (ActorId, ActorId),
    ball: ActorId,
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
    pub fn new(engine: &mut Engine) -> Self {
        let size_factor = SCREEN_SIZE as f32 / 32.0;
        Self {
            actors: Vec::new(),
            score: (0, 0),
            paddle: (
                RectangleActor::spawn(
                    engine,
                    V2::new(SCREEN_SIZE as f32 / 2.0, 3.0 * size_factor),
                    &V2::new(7.0, 1.0),
                    Color::white(),
                    Some(String::from("paddle1")),
                ),
                RectangleActor::spawn(
                    engine,
                    V2::new(
                        SCREEN_SIZE as f32 / 2.0,
                        SCREEN_SIZE as f32 - 4.0 * size_factor,
                    ),
                    &V2::new(7.0, 1.0),
                    Color::white(),
                    Some(String::from("paddle2")),
                ),
            ),
            score_zone: (
                RectangleActor::spawn(
                    engine,
                    V2::zero(),
                    &V2::one(),
                    Color::white(),
                    Some(String::from("score_zone1")),
                ),
                RectangleActor::spawn(
                    engine,
                    V2::zero(),
                    &V2::one(),
                    Color::white(),
                    Some(String::from("score_zone2")),
                ),
            ),
            ball: RectangleActor::spawn(
                engine,
                V2::zero(),
                &V2::one(),
                Color::white(),
                Some(String::from("ball")),
            ),
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

    // void pong_scene::handle_input(float deltaTime) {
    //     if (input::is_key_press(key::P1_LEFT) ^ input::is_key_press(key::P1_RIGHT))
    //         move_paddle(p1Paddle, (input::is_key_press(key::P1_RIGHT) ? 1 : -1) * paddle_speed * deltaTime * size_factor);

    //     if (input::is_key_press(key::P2_LEFT) ^ input::is_key_press(key::P2_RIGHT))
    //         move_paddle(p2Paddle, (input::is_key_press(key::P2_RIGHT) ? 1 : -1) * paddle_speed * deltaTime * size_factor);
    // }

    fn handle_input(&self, delta_time: f32, engine: &Engine) {
        let input = engine.input.borrow();
        let actor_map = engine.actor_map.borrow_mut();
        if input.is_key_press(Key::P1Left) ^ input.is_key_press(Key::P1Right) {
            PongScene::move_paddle(actor_map.get_mut(self.paddle.0).unwrap().as_mut() as RectangleActor, delta);
        }
    }
}

impl Scene for PongScene {
    fn get_actors(&self) -> &[Box<dyn TActor>] {
        &self.actors
    }

    fn init(&mut self) {}

    fn update(&self, _delta_time: f32, engine: &Engine) {}
}
