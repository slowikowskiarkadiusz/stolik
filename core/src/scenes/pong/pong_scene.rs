use crate::engine::{
    actor::rectangle_actor::create_rectangle_actor,
    color::Color,
    components::world::World,
    engine::{ActorId, SCREEN_SIZE},
    input::{input::Input, key::Key},
    scene::Scene,
    v2::V2,
};

pub struct PongScene {
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
    pub fn new() -> Self {
        Self {
            score: (0, 0),
            paddle: (None, None),
            score_zone: (None, None),
            ball: None,
            paddle_speed: 3.0,
            max_bounce_speed: 0.03,
            original_ball_speed: 0.007,
            ball_speed: V2::one(),
            ball_speed_multiplier: 1.0,
            size_factor: SCREEN_SIZE as f32 / 32.0,
            can_score: true,
            can_collide: (true, true),
            can_bounce: true,
            do_play: true,
        }
    }

    fn move_paddle(paddle: &ActorId, world: &mut World, delta: f32) {
        let transform = world.get_mut_transform(paddle).unwrap();
        transform.center = V2::right() * delta;
        let left = transform.center.x - transform.size.x / 2.0;
        let right = transform.center.x + transform.size.x / 2.0;
        if left < 0.0 {
            transform.center = V2::new(transform.size.x / 2.0, transform.center.y);
        }
        if right > SCREEN_SIZE as f32 {
            transform.center = V2::new(SCREEN_SIZE as f32 - transform.size.x / 2.0, transform.center.y);
        }
    }
}

impl Scene for PongScene {
    fn init(&mut self, world: &mut World) {
        let size_factor = SCREEN_SIZE as f32 / 32.0;
        self.paddle = (
            Some(create_rectangle_actor(
                world,
                V2::new(SCREEN_SIZE as f32 / 2.0, 3.0 * size_factor),
                V2::new(7.0, 1.0),
                Color::white(),
                Some(String::from("paddle1")),
            )),
            Some(create_rectangle_actor(
                world,
                V2::new(SCREEN_SIZE as f32 / 2.0, SCREEN_SIZE as f32 - 4.0 * size_factor),
                V2::new(7.0, 1.0),
                Color::white(),
                Some(String::from("paddle2")),
            )),
        );
        self.score_zone = (
            Some(create_rectangle_actor(
                world,
                V2::zero(),
                V2::one(),
                Color::white(),
                Some(String::from("score_zone1")),
            )),
            Some(create_rectangle_actor(
                world,
                V2::zero(),
                V2::one(),
                Color::white(),
                Some(String::from("score_zone2")),
            )),
        );
        self.ball = Some(create_rectangle_actor(world, V2::zero(), V2::one(), Color::white(), Some(String::from("ball"))));
    }

    fn tick(&mut self, input: &Box<dyn Input>, world: &mut World, delta_time: f32) {
        if input.is_key_press(Key::P1Left) ^ input.is_key_press(Key::P1Right) {
            if let Some(paddle_p1_id) = self.paddle.0 {
                PongScene::move_paddle(
                    &paddle_p1_id,
                    world,
                    if input.is_key_press(Key::P1Left) { -1.0 } else { 1.0 } * self.paddle_speed * self.size_factor * delta_time,
                );
            }
        }

        if input.is_key_press(Key::P2Left) ^ input.is_key_press(Key::P2Right) {
            if let Some(paddle_p2_id) = self.paddle.1 {
                PongScene::move_paddle(
                    &paddle_p2_id,
                    world,
                    if input.is_key_press(Key::P2Left) { -1.0 } else { 1.0 } * self.paddle_speed * self.size_factor * delta_time,
                );
            }
        }
    }
}
