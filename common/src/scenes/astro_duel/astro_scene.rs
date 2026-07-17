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
    scenes::{
        menu::menu_scene::MenuScene,
        utils::{print_score, print_victory_text},
    },
};
use rand::{SeedableRng, rngs::SmallRng};

use super::{astro_obstacle::AstroObstacleMap, bullet::Bullet, ship::Ship};

const GAME_OVER_DELAY: f32 = 4.0;
const DEATH_DELAY: f32 = 2.5;
const WIN_SCORE: u8 = 5;

pub struct AstroDuelScene {
    obstacle: Option<AstroObstacleMap>,
    ship_p1: Option<Ship>,
    ship_p2: Option<Ship>,
    bullets: Vec<Bullet>,
    score: [u8; 2],
    winner: Option<u8>,
    game_over_timer: f32,
    death_timer: Option<(f32, usize)>,
}

impl AstroDuelScene {
    pub fn new() -> Self {
        Self {
            obstacle: None,
            ship_p1: None,
            ship_p2: None,
            bullets: Vec::new(),
            score: [0, 0],
            winner: None,
            game_over_timer: 0.0,
            death_timer: None,
        }
    }

    fn reset_round(&mut self, world: &mut World) {
        world.clear_all();
        let mut rng = SmallRng::seed_from_u64(embassy_time::Instant::now().as_micros());
        self.obstacle = Some(AstroObstacleMap::new(&mut rng, world));
        self.ship_p1 = Some(Ship::new(world, true));
        self.ship_p2 = Some(Ship::new(world, false));
        self.bullets = Vec::new();
        self.death_timer = None;
    }
}

fn check_winner(score: &[u8; 2]) -> Option<u8> {
    for i in 0..2usize {
        let j = 1 - i;
        if score[i] >= WIN_SCORE && score[i] >= score[j] + 2 {
            return Some((i + 1) as u8);
        }
    }
    None
}

impl Scene for AstroDuelScene {
    fn init(&mut self, world: &mut World) {
        Physics::set_gravity(V2::zero());
        self.score = [0, 0];
        self.winner = None;
        self.game_over_timer = 0.0;
        self.reset_round(world);
    }

    fn tick(&mut self, input: &Box<dyn Input>, world: &mut World, delta_time: f32) {
        if self.winner.is_some() {
            self.game_over_timer -= delta_time;
            if self.game_over_timer <= 0.0 {
                open_scene(Box::new(|| Box::new(MenuScene::new())));
            }
            return;
        }

        let death_expired = if let Some((ref mut timer, _)) = self.death_timer {
            *timer -= delta_time;
            *timer <= 0.0
        } else {
            false
        };

        if death_expired {
            let scorer = self.death_timer.take().map(|(_, s)| s).unwrap();
            self.score[scorer] += 1;
            if let Some(w) = check_winner(&self.score) {
                self.winner = Some(w);
                self.game_over_timer = GAME_OVER_DELAY;
            } else {
                self.reset_round(world);
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

        if self.death_timer.is_none() {
            if let Some(sp) = spawn1 {
                self.bullets.push(Bullet::new(world, sp.pos, sp.velocity, sp.owner_is_p1));
            }
            if let Some(sp) = spawn2 {
                self.bullets.push(Bullet::new(world, sp.pos, sp.velocity, sp.owner_is_p1));
            }
        }

        let mut i = 0;
        while i < self.bullets.len() {
            let expired = self.bullets[i].tick(world, obstacle, delta_time);
            if expired {
                self.bullets[i].destroy(world);
                self.bullets.remove(i);
            } else {
                i += 1;
            }
        }
    }

    fn render(&mut self, camera: &Camera, world: &mut World, _delta_time: f32) -> ColorMatrix {
        world
            .get_mut_camera()
            .set_viewport((V2::zero(), V2::new(SCREEN_SIZEF32, SCREEN_SIZEF32)));

        let mut result = self
            .obstacle
            .as_ref()
            .map(|o| o.render(world))
            .unwrap_or_else(|| ColorMatrix::new(SCREEN_SIZE, SCREEN_SIZE, Color::black()));

        if let Some(s) = self.ship_p1.as_ref() {
            s.render(world, camera, &mut result);
        }
        if let Some(s) = self.ship_p2.as_ref() {
            s.render(world, camera, &mut result);
        }
        for b in &self.bullets {
            b.render(world, camera, &mut result);
        }

        print_score(self.score[0], self.score[1], &mut result);

        if let Some(winner) = self.winner {
            print_victory_text(&mut result, winner);
        }

        result
    }

    fn on_overlaps(&mut self, overlaps: &HashMap<ActorId, Vec<ActorId>>, world: &mut World, _: f32) {
        if self.death_timer.is_some() || self.winner.is_some() {
            return;
        }

        let p1_id = self.ship_p1.as_ref().filter(|s| s.lives > 0).map(|s| s.actor_id);
        let p2_id = self.ship_p2.as_ref().filter(|s| s.lives > 0).map(|s| s.actor_id);
        let mut hit_p1 = false;
        let mut hit_p2 = false;

        for i in 0..self.bullets.len() {
            if self.bullets[i].hit {
                continue;
            }
            let bullet_id = self.bullets[i].actor_id;
            let owner_is_p1 = self.bullets[i].owner_is_p1;
            let Some(list) = overlaps.get(&bullet_id) else { continue };
            let overlapped: Vec<ActorId> = list.iter().copied().collect();
            for overlapped_id in overlapped {
                if !owner_is_p1 && p1_id == Some(overlapped_id) {
                    hit_p1 = true;
                    self.bullets[i].hit = true;
                } else if owner_is_p1 && p2_id == Some(overlapped_id) {
                    hit_p2 = true;
                    self.bullets[i].hit = true;
                } else {
                    let is_destroyable = self
                        .obstacle
                        .as_ref()
                        .map(|o| o.is_destroyable_obstacle(&overlapped_id))
                        .unwrap_or(false);
                    if is_destroyable {
                        if let Some(o) = self.obstacle.as_mut() {
                            o.remove(&overlapped_id);
                        }
                        world.murder(&overlapped_id);
                        self.bullets[i].hit = true;
                    }

                    let is_border = self
                        .obstacle
                        .as_ref()
                        .map(|o| o.is_border_obstacle(&overlapped_id))
                        .unwrap_or(false);
                    if is_border {
                        if let Some(o) = self.obstacle.as_mut() {
                            o.remove(&overlapped_id);
                        }
                        self.bullets[i].hit = true;
                    }
                }
            }
        }

        if hit_p1 {
            if let Some(s) = self.ship_p1.as_mut() {
                let died = s.take_hit();
                if died {
                    self.death_timer = Some((DEATH_DELAY, 1)); // P2 scores
                }
            }
        }
        if hit_p2 {
            if let Some(s) = self.ship_p2.as_mut() {
                let died = s.take_hit();
                if died {
                    self.death_timer = Some((DEATH_DELAY, 0)); // P1 scores
                }
            }
        }
    }

    fn on_collisions(&mut self, _: &HashMap<u16, Vec<(u16, CollisionResult)>>, _: &mut World, _: f32) {}
}
