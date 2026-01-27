use crate::engine::{
    actor::{
        actor::{InnerActor, TActor},
        rectangle_actor::RectangleActor,
    },
    color::Color,
    engine::{ActorId, Engine, SCREEN_SIZE},
    scene::Scene,
    v2::{self, V2},
};

pub struct PongScene {
    actors: Vec<Box<dyn TActor>>,
    score: (u8, u8),
    paddle: (ActorId, ActorId),
    score_zone: (ActorId, ActorId),
    ball: ActorId,
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
        let size_factor = SCREEN_SIZE as f32;
        Self {
            actors: Vec::new(),
            score: (0, 0),
            // p1Paddle = engine::instantiate<rectangle_actor>(v2(screen.x / 2, 3 * size_factor), v2(7, 1) * size_factor);
            // p2Paddle = engine::instantiate<rectangle_actor>(v2(screen.x / 2, screen.y - 4 * size_factor), v2(7, 1) * size_factor);
            // p1ScoreZone = engine::instantiate<rectangle_actor>(v2(screen.x / 2, -4 * size_factor), v2(screen.x, 10), color::none());
            // p2ScoreZone = engine::instantiate<rectangle_actor>(v2(screen.x / 2, screen.y + 4 * size_factor), v2(screen.x, 10), color::none());
            // ball = engine::instantiate<rectangle_actor>(screen / 2, v2(2, 2) * size_factor, color::white());
            paddle: (
                RectangleActor::spawn(engine, V2::new(SCREEN_SIZE as f32 / 2.0, 3.0 * size_factor), &V2::new(7.0, 1.0), Color::white()),
                RectangleActor::spawn(engine, V2::new(SCREEN_SIZE as f32 / 2.0, SCREEN_SIZE as f32 - 4.0 * size_factor), &V2::new(7.0, 1.0), Color::white()),
            ),
            score_zone: (
                RectangleActor::spawn(engine, V2::zero(), &V2::one(), Color::white()),
                RectangleActor::spawn(engine, V2::zero(), &V2::one(), Color::white()),
            ),
            ball: RectangleActor::spawn(engine, V2::zero(), &V2::one(), Color::white()),
            ball_speed: V2::one(),
            ball_speed_multiplier: 1.0,
            size_factor: 1.0,
            can_score: true,
            can_collide: (true, true),
            can_bounce: true,
            do_play: true,
        }
    }
}

impl Scene for PongScene {
    fn get_actors(&self) -> &[Box<dyn TActor>] {
        &self.actors
    }

    fn init(&mut self) {}

    fn update(&self, _delta_time: f32) {}
}
