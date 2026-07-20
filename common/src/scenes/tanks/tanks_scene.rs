extern crate alloc;
use alloc::{boxed::Box, vec::Vec};

use super::world::TanksWorld;
use crate::scenes::menu::menu_scene::MenuScene;
use crate::scenes::tanks::bullet::Bullet;
use crate::write_m;
use crate::{
    engine::{
        color::Color,
        color_matrix::ColorMatrix,
        components::{camera::Camera, collider::CollisionResult, world::World},
        engine::{ActorId, SCREEN_SIZE, SCREEN_SIZEF32, open_scene},
        hash_map::HashMap,
        input::input::Input,
        scene::Scene,
        v2::V2,
    },
    scenes::utils::print_victory_text,
};

const GAME_OVER_DELAY: f32 = 5.0;
const HEART_HIT_DAMAGE: u8 = 4;

pub struct TanksScene {
    world: Option<TanksWorld>,
}

impl TanksScene {
    pub fn new() -> Self {
        Self { world: None }
    }
}

impl Scene for TanksScene {
    fn init(&mut self, _world: &mut World) {
        self.world = Some(TanksWorld::new());
    }

    fn tick(&mut self, input: &Box<dyn Input>, _world: &mut World, delta_time: f32) {
        let w = match self.world.as_mut() {
            Some(w) => w,
            None => return,
        };

        if w.winner.is_some() {
            w.game_over_timer -= delta_time;
            if w.game_over_timer <= 0.0 {
                open_scene(Box::new(|| Box::new(MenuScene::new())));
            }
            return;
        }

        w.heart_p1.tick(delta_time);
        w.heart_p2.tick(delta_time);

        let p1_blockers = {
            let mut b: [Option<(V2, V2)>; 3] = [None; 3];
            if w.tank_p2.alive {
                b[0] = Some((w.tank_p2.pos - V2::new(1.5, 1.5), w.tank_p2.pos + V2::new(2.5, 2.5)));
            }
            b[1] = w.heart_p1.blocker_box();
            b[2] = w.heart_p2.blocker_box();
            b
        };
        let p1_blockers_slice: Vec<(V2, V2)> = p1_blockers.iter().filter_map(|x| *x).collect();

        let p2_blockers = {
            let mut b: [Option<(V2, V2)>; 3] = [None; 3];
            if w.tank_p1.alive {
                b[0] = Some((w.tank_p1.pos - V2::new(1.5, 1.5), w.tank_p1.pos + V2::new(2.5, 2.5)));
            }
            b[1] = w.heart_p1.blocker_box();
            b[2] = w.heart_p2.blocker_box();
            b
        };
        let p2_blockers_slice: Vec<(V2, V2)> = p2_blockers.iter().filter_map(|x| *x).collect();

        let spawn1 = w
            .tank_p1
            .tick(input.as_ref(), &w.obstacle, &p1_blockers_slice, w.bullet_p1.is_some(), delta_time);
        let spawn2 = w
            .tank_p2
            .tick(input.as_ref(), &w.obstacle, &p2_blockers_slice, w.bullet_p2.is_some(), delta_time);

        if let Some(s) = spawn1 {
            w.bullet_p1 = Some(Bullet::new(s.pos, s.dir, s.level, true));
        }
        if let Some(s) = spawn2 {
            w.bullet_p2 = Some(Bullet::new(s.pos, s.dir, s.level, false));
        }

        if let Some(b) = w.bullet_p1.as_mut() {
            b.tick(&mut w.obstacle, &mut w.tank_p1, &mut w.tank_p2, delta_time);
        }
        let p1_heart_hit = w.bullet_p1.as_ref().and_then(|b| {
            if w.heart_p1.overlaps_point(b.pos) {
                Some(false)
            } else if w.heart_p2.overlaps_point(b.pos) {
                Some(true)
            } else {
                None
            }
        });
        if let Some(hit_bottom) = p1_heart_hit {
            if hit_bottom {
                w.heart_p2.take_hit(HEART_HIT_DAMAGE);
            } else {
                w.heart_p1.take_hit(HEART_HIT_DAMAGE);
            }
            if let Some(b) = w.bullet_p1.as_mut() {
                b.alive = false;
            }
        }
        if w.bullet_p1.as_ref().map(|b| !b.alive).unwrap_or(false) {
            w.bullet_p1 = None;
        }

        if let Some(b) = w.bullet_p2.as_mut() {
            b.tick(&mut w.obstacle, &mut w.tank_p1, &mut w.tank_p2, delta_time);
        }
        let p2_heart_hit = w.bullet_p2.as_ref().and_then(|b| {
            if !b.alive {
                return None;
            }
            if w.heart_p1.overlaps_point(b.pos) {
                Some(false)
            } else if w.heart_p2.overlaps_point(b.pos) {
                Some(true)
            } else {
                None
            }
        });
        if let Some(hit_bottom) = p2_heart_hit {
            if hit_bottom {
                w.heart_p2.take_hit(HEART_HIT_DAMAGE);
            } else {
                w.heart_p1.take_hit(HEART_HIT_DAMAGE);
            }
            if let Some(b) = w.bullet_p2.as_mut() {
                b.alive = false;
            }
        }
        if w.bullet_p2.as_ref().map(|b| !b.alive).unwrap_or(false) {
            w.bullet_p2 = None;
        }

        if !w.heart_p1.alive && w.winner.is_none() {
            w.winner = Some(2);
            w.game_over_timer = GAME_OVER_DELAY;
        }
        if !w.heart_p2.alive && w.winner.is_none() {
            w.winner = Some(1);
            w.game_over_timer = GAME_OVER_DELAY;
        }
    }

    fn render(&mut self, camera: &Camera, world: &mut World, _delta_time: f32) -> ColorMatrix {
        world
            .get_mut_camera()
            .set_viewport((V2::zero(), V2::new(SCREEN_SIZEF32, SCREEN_SIZEF32)));

        let w = match self.world.as_ref() {
            Some(w) => w,
            None => return ColorMatrix::new(SCREEN_SIZE, SCREEN_SIZE, Color::none()),
        };

        let mut result = w.obstacle.render();

        w.heart_p1.draw(&mut result);
        w.heart_p2.draw(&mut result);

        if w.tank_p1.alive {
            let s = w.tank_p1.render();
            write_m!(&mut result, &s, &w.tank_p1.pos);
        }
        if w.tank_p2.alive {
            let s = w.tank_p2.render();
            write_m!(&mut result, &s, &w.tank_p2.pos);
        }

        if let Some(b) = w.bullet_p1.as_ref() {
            let s = b.render();
            write_m!(&mut result, &s, &b.pos);
        }
        if let Some(b) = w.bullet_p2.as_ref() {
            let s = b.render();
            write_m!(&mut result, &s, &b.pos);
        }

        if let Some(winner) = w.winner {
            print_victory_text(&mut result, winner, camera, true);
        }

        result
    }

    fn on_overlaps(&mut self, _: &HashMap<ActorId, Vec<ActorId>>, _: &mut World, _: f32) {}
    fn on_collisions(&mut self, _: &HashMap<u16, Vec<(u16, CollisionResult)>>, _: &mut World, _: f32) {}
}
