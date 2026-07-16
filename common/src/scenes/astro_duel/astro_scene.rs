extern crate alloc;
use alloc::{boxed::Box, vec::Vec};

use crate::{
    engine::{
        color::Color,
        color_matrix::ColorMatrix,
        components::{camera::Camera, collider::CollisionResult, physics::Physics, world::World},
        engine::{ActorId, SCREEN_SIZE, SCREEN_SIZEF32, open_scene},
        hash_map::HashMap,
        input::input::Input,
        scene::Scene,
        v2::V2,
    },
    scenes::{menu::menu_scene::MenuScene, utils::print_victory_text},
};
use rand::{SeedableRng, rngs::SmallRng};

use super::{astro_obstacle::AstroObstacleMap, bullet::Bullet, ship::Ship};

const GAME_OVER_DELAY: f32 = 4.0;
const HIT_RADIUS: f32 = 3.5;

pub struct AstroDuelScene {
    obstacle: Option<AstroObstacleMap>,
    ship_p1: Option<Ship>,
    ship_p2: Option<Ship>,
    bullets: Vec<Bullet>,
    winner: Option<u8>,
    game_over_timer: f32,
}

impl AstroDuelScene {
    pub fn new() -> Self {
        Self {
            obstacle: None,
            ship_p1: None,
            ship_p2: None,
            bullets: Vec::new(),
            winner: None,
            game_over_timer: 0.0,
        }
    }
}

impl Scene for AstroDuelScene {
    fn init(&mut self, world: &mut World) {
        Physics::set_gravity(V2::zero());
        let mut rng = SmallRng::seed_from_u64(embassy_time::Instant::now().as_micros());
        self.obstacle = Some(AstroObstacleMap::new(&mut rng, world));
        self.ship_p1 = Some(Ship::new(world, true));
        self.ship_p2 = Some(Ship::new(world, false));
        self.bullets = Vec::new();
        self.winner = None;
        self.game_over_timer = 0.0;
    }

    fn tick(&mut self, input: &Box<dyn Input>, world: &mut World, delta_time: f32) {
        if self.winner.is_some() {
            self.game_over_timer -= delta_time;
            if self.game_over_timer <= 0.0 {
                open_scene(Box::new(|| Box::new(MenuScene::new())));
            }
            return;
        }

        let obstacle = match self.obstacle.as_ref() {
            Some(o) => o,
            None => return,
        };

        let spawn1 = self
            .ship_p1
            .as_mut()
            .and_then(|s| s.tick(input.as_ref(), world, obstacle, delta_time));
        let spawn2 = self
            .ship_p2
            .as_mut()
            .and_then(|s| s.tick(input.as_ref(), world, obstacle, delta_time));

        if let Some(sp) = spawn1 {
            self.bullets.push(Bullet::new(world, sp.pos, sp.velocity, sp.owner_is_p1));
        }
        if let Some(sp) = spawn2 {
            self.bullets.push(Bullet::new(world, sp.pos, sp.velocity, sp.owner_is_p1));
        }

        let p1_pos = self.ship_p1.as_ref().and_then(|s| s.pos(world));
        let p2_pos = self.ship_p2.as_ref().and_then(|s| s.pos(world));

        let mut hit_p1 = false;
        let mut hit_p2 = false;
        let mut i = 0;
        while i < self.bullets.len() {
            let expired = self.bullets[i].tick(world, obstacle, delta_time);
            if expired {
                self.bullets[i].destroy(world);
                self.bullets.remove(i);
                continue;
            }

            if let Some(bpos) = self.bullets[i].pos(world) {
                let is_p1_bullet = self.bullets[i].owner_is_p1;
                let mut destroyed = false;

                if !is_p1_bullet {
                    if let Some(p) = p1_pos {
                        if self.ship_p1.as_ref().map(|s| s.alive).unwrap_or(false) && bpos.distance(&p) < HIT_RADIUS {
                            hit_p1 = true;
                            destroyed = true;
                        }
                    }
                } else {
                    if let Some(p) = p2_pos {
                        if self.ship_p2.as_ref().map(|s| s.alive).unwrap_or(false) && bpos.distance(&p) < HIT_RADIUS {
                            hit_p2 = true;
                            destroyed = true;
                        }
                    }
                }

                if destroyed {
                    self.bullets[i].destroy(world);
                    self.bullets.remove(i);
                    continue;
                }
            }
            i += 1;
        }

        if hit_p1 {
            if let Some(s) = self.ship_p1.as_mut() {
                s.take_hit(world);
                if s.lives == 0 {
                    self.winner = Some(2);
                    self.game_over_timer = GAME_OVER_DELAY;
                }
            }
        }
        if hit_p2 {
            if let Some(s) = self.ship_p2.as_mut() {
                s.take_hit(world);
                if s.lives == 0 {
                    self.winner = Some(1);
                    self.game_over_timer = GAME_OVER_DELAY;
                }
            }
        }
    }

    fn render(&mut self, _camera: &Camera, world: &mut World, _delta_time: f32) -> ColorMatrix {
        world
            .get_mut_camera()
            .set_viewport((V2::zero(), V2::new(SCREEN_SIZEF32, SCREEN_SIZEF32)));

        let mut result = self
            .obstacle
            .as_ref()
            .map(|o| o.render(world))
            .unwrap_or_else(|| ColorMatrix::new(SCREEN_SIZE, SCREEN_SIZE, Color::black()));

        if let Some(s) = self.ship_p1.as_ref() {
            s.render(world, _camera, &mut result);
        }
        if let Some(s) = self.ship_p2.as_ref() {
            s.render(world, _camera, &mut result);
        }
        for b in &self.bullets {
            b.render(world, _camera, &mut result);
        }

        if let Some(winner) = self.winner {
            print_victory_text(&mut result, winner);
        }

        result
    }

    fn on_overlaps(&mut self, _: &HashMap<ActorId, Vec<ActorId>>, _: &mut World, _: f32) {}

    fn on_collisions(&mut self, _: &HashMap<u16, Vec<(u16, CollisionResult)>>, _: &mut World, _: f32) {}
}
