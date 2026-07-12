extern crate alloc;
use alloc::{boxed::Box, vec::Vec};

use crate::engine::{
    color::Color, color_matrix::ColorMatrix, components::{camera::Camera, collider::CollisionResult, world::World}, engine::{ActorId, SCREEN_SIZE, SCREEN_SIZEF32, open_scene}, hash_map::HashMap, input::input::Input, scene::Scene, v2::V2,
};
use crate::scenes::menu::menu_scene::MenuScene;
use super::world::TanksWorld;

const GAME_OVER_DELAY: f32 = 5.0;

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

        let spawn1 = w.tank1.tick(input.as_ref(), &w.obstacle, w.bullet_p1.is_some(), delta_time);
        let spawn2 = w.tank2.tick(input.as_ref(), &w.obstacle, w.bullet_p2.is_some(), delta_time);

        if let Some(s) = spawn1 {
            w.bullet_p1 = Some(crate::scenes::tanks::bullet::Bullet::new(s.pos, s.dir, s.level, true));
        }
        if let Some(s) = spawn2 {
            w.bullet_p2 = Some(crate::scenes::tanks::bullet::Bullet::new(s.pos, s.dir, s.level, false));
        }

        if let Some(b) = w.bullet_p1.as_mut() {
            b.tick(&mut w.obstacle, &mut w.tank1, &mut w.tank2, delta_time);
        }
        if w.bullet_p1.as_ref().map(|b| !b.alive).unwrap_or(false) {
            w.bullet_p1 = None;
        }

        if let Some(b) = w.bullet_p2.as_mut() {
            b.tick(&mut w.obstacle, &mut w.tank1, &mut w.tank2, delta_time);
        }
        if w.bullet_p2.as_ref().map(|b| !b.alive).unwrap_or(false) {
            w.bullet_p2 = None;
        }

        if !w.tank1.alive && w.winner.is_none() {
            w.winner = Some(2);
            w.game_over_timer = GAME_OVER_DELAY;
        }
        if !w.tank2.alive && w.winner.is_none() {
            w.winner = Some(1);
            w.game_over_timer = GAME_OVER_DELAY;
        }
    }

    fn render(&mut self, _camera: &Camera, world: &mut World, _delta_time: f32) -> ColorMatrix {
        world.get_mut_camera().set_viewport((V2::zero(), V2::new(SCREEN_SIZEF32, SCREEN_SIZEF32)));

        let w = match self.world.as_ref() {
            Some(w) => w,
            None => return ColorMatrix::new(SCREEN_SIZE, SCREEN_SIZE, Color::none()),
        };

        let mut result = w.obstacle.render();

        let t1_sprite = w.tank1.render();
        result.write(&t1_sprite, &w.tank1.pos, None, None, None, None);

        let t2_sprite = w.tank2.render();
        result.write(&t2_sprite, &w.tank2.pos, None, None, None, None);

        if let Some(b) = w.bullet_p1.as_ref() {
            let sprite = b.render();
            result.write(&sprite, &b.pos, None, None, None, None);
        }
        if let Some(b) = w.bullet_p2.as_ref() {
            let sprite = b.render();
            result.write(&sprite, &b.pos, None, None, None, None);
        }

        result
    }

    fn on_overlaps(&mut self, _: &HashMap<ActorId, Vec<ActorId>>, _: &mut World, _: f32) {}
    fn on_collisions(&mut self, _: &HashMap<u16, Vec<(u16, CollisionResult)>>, _: &mut World, _: f32) {}
}
