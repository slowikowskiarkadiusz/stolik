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

use super::{
    astro_obstacle::{AstroObstacleMap, BOARD_CELLS, CELL_SIZEF32},
    bullet::Bullet,
    power_ups::{reflector, mine::PlacedMine, ray_gun::RayGunBeam, power_up::PowerUp},
    ship::{Ship, ShipAction},
};

const POWER_UP_RESPAWN_DELAY: f32 = 5.0;

pub struct AstroDuelScene {
    obstacle: Option<AstroObstacleMap>,
    ship_p1: Option<Ship>,
    ship_p2: Option<Ship>,
    bullets: Vec<Bullet>,
    power_ups: Vec<PowerUp>,
    placed_mines: Vec<PlacedMine>,
    ray_gun_blasts: Vec<RayGunBeam>,
    score: [u8; 2],
    winner: Option<u8>,
    game_over_timer: f32,
    death_timer: Option<(f32, usize)>,
    power_up_respawn_timer: Option<f32>,
}

impl AstroDuelScene {
    pub fn new() -> Self {
        Self {
            obstacle: None,
            ship_p1: None,
            ship_p2: None,
            bullets: Vec::new(),
            power_ups: Vec::new(),
            placed_mines: Vec::new(),
            ray_gun_blasts: Vec::new(),
            score: [0, 0],
            winner: None,
            game_over_timer: 0.0,
            death_timer: None,
            power_up_respawn_timer: None,
        }
    }

    fn reset_round(&mut self, world: &mut World) {
        world.clear_all();
        let mut rng = SmallRng::seed_from_u64(embassy_time::Instant::now().as_micros());
        self.obstacle = Some(AstroObstacleMap::new(&mut rng, world));
        self.ship_p1 = Some(Ship::new(world, true));
        self.ship_p2 = Some(Ship::new(world, false));
        self.bullets = Vec::new();
        self.placed_mines = Vec::new();
        self.ray_gun_blasts = Vec::new();
        self.death_timer = None;

        self.power_ups = Vec::new();
        self.power_up_respawn_timer = None;
        self.spawn_power_ups(world);
    }

    fn spawn_power_ups(&mut self, world: &mut World) {
        use rand::{Rng, SeedableRng};
        let mut rng = SmallRng::seed_from_u64(embassy_time::Instant::now().as_micros());
        let interior = (BOARD_CELLS - 2) as u32;
        let gx = (rng.gen_range(0..interior) + 1) as u8;
        let gy = (rng.gen_range(0..interior) + 1) as u8;
        let cell_center = |g: u8| g as f32 * CELL_SIZEF32 + CELL_SIZEF32 / 2.0;
        let pos_a = V2::new(cell_center(gx), cell_center(gy));
        let pos_b = V2::new(cell_center(BOARD_CELLS - 1 - gx), cell_center(BOARD_CELLS - 1 - gy));
        self.power_ups.push(PowerUp::new(pos_a, &mut rng, world));
        self.power_ups.push(PowerUp::new(pos_b, &mut rng, world));
    }

    fn ship_center(ship: &Option<Ship>, world: &World) -> Option<V2> {
        ship.as_ref()
            .filter(|s| s.lives > 0)
            .and_then(|s| world.get_transform(&s.actor_id))
            .map(|t| t.center)
    }

    fn picking_up_power_ups(&mut self, overlaps: &HashMap<u16, Vec<u16>>, world: &mut World) {
        let power_up_ids: Vec<(ActorId, usize)> = self.power_ups.iter().enumerate()
            .map(|(i, pu)| (pu.actor_id, i))
            .collect();

        let mut picked_power_up: Option<(usize, bool)> = None; // (power_up_idx, is_p1)
        'power_up_outer: for &(power_up_actor, power_up_idx) in &power_up_ids {
            let ships = [(&self.ship_p1, true), (&self.ship_p2, false)];
            for (ship_opt, is_p1) in &ships {
                if let Some(ship) = ship_opt {
                    let hit = overlaps.get(&ship.actor_id).map_or(false, |l| l.contains(&power_up_actor))
                        || overlaps.get(&power_up_actor).map_or(false, |l| l.contains(&ship.actor_id));
                    if hit {
                        picked_power_up = Some((power_up_idx, *is_p1));
                        break 'power_up_outer;
                    }
                }
            }
        }

        if let Some((idx, is_p1)) = picked_power_up {
            let power_up = self.power_ups.remove(idx);
            let power_up_type = power_up.power_up_type();
            world.murder(&power_up.actor_id);
            let ship = if is_p1 { &mut self.ship_p1 } else { &mut self.ship_p2 };
            if let Some(s) = ship {
                s.give_power_up(power_up_type);
            }
        }
    }
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
            self.death_timer = None;
            if self.winner.is_none() {
                self.reset_round(world);
            }
            return;
        }

        let obstacle = match self.obstacle.as_ref() {
            Some(o) => o,
            None => return,
        };

        let actions1 = self.ship_p1.as_mut().map(|s| s.tick(input.as_ref(), world, obstacle, delta_time)).unwrap_or_default();
        let actions2 = self.ship_p2.as_mut().map(|s| s.tick(input.as_ref(), world, obstacle, delta_time)).unwrap_or_default();

        if self.death_timer.is_none() {
            for action in actions1.into_iter().chain(actions2.into_iter()) {
                match action {
                    ShipAction::SpawnBullet(sp) => {
                        self.bullets.push(Bullet::new(world, sp.pos, sp.velocity, sp.owner_is_p1));
                    }
                    ShipAction::PlaceMine(mine) => {
                        self.placed_mines.push(mine);
                    }
                    ShipAction::FireRayGun(blast) => {
                        self.ray_gun_blasts.push(blast);
                    }
                }
            }
        }

        let mut i = 0;
        while i < self.bullets.len() {
            if self.bullets[i].tick(world, obstacle, delta_time) {
                self.bullets[i].destroy(world);
                self.bullets.remove(i);
            } else {
                i += 1;
            }
        }

        let mut i = 0;
        while i < self.placed_mines.len() {
            if self.placed_mines[i].tick(delta_time) {
                self.placed_mines.remove(i);
            } else {
                i += 1;
            }
        }

        let mut i = 0;
        while i < self.ray_gun_blasts.len() {
            if self.ray_gun_blasts[i].tick(delta_time) {
                self.ray_gun_blasts.remove(i);
            } else {
                i += 1;
            }
        }

        for pu in &mut self.power_ups {
            pu.tick(delta_time);
        }

        // Respawn power-ups 5s after both are collected
        if self.power_ups.is_empty() {
            let timer = self.power_up_respawn_timer.get_or_insert(POWER_UP_RESPAWN_DELAY);
            *timer -= delta_time;
            if *timer <= 0.0 {
                self.power_up_respawn_timer = None;
                self.spawn_power_ups(world);
            }
        } else {
            self.power_up_respawn_timer = None;
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

        for mine in &self.placed_mines {
            mine.render(&mut result);
        }
        for blast in &self.ray_gun_blasts {
            blast.render(&mut result);
        }
        for pu in &self.power_ups {
            pu.render(world, &mut result);
        }
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
            print_victory_text(&mut result, winner, camera, true);
        }

        result
    }

    fn on_overlaps(&mut self, overlaps: &HashMap<ActorId, Vec<ActorId>>, world: &mut World, _: f32) {
        if self.death_timer.is_some() || self.winner.is_some() {
            return;
        }

        let p1_id = self.ship_p1.as_ref().filter(|s| s.lives > 0).map(|s| s.actor_id);
        let p2_id = self.ship_p2.as_ref().filter(|s| s.lives > 0).map(|s| s.actor_id);

        self.picking_up_power_ups(overlaps, world);

        reflector::apply(&self.ship_p1, &self.ship_p2, &mut self.bullets, world);

        let p1_center = Self::ship_center(&self.ship_p1, world);
        let p2_center = Self::ship_center(&self.ship_p2, world);

        PlacedMine::apply(&mut self.placed_mines, p1_center, p2_center, &mut self.ship_p1, &mut self.ship_p2, &mut self.score, &mut self.winner, &mut self.game_over_timer, &mut self.death_timer);

        RayGunBeam::apply(&mut self.ray_gun_blasts, p1_center, p2_center, &mut self.ship_p1, &mut self.ship_p2, &mut self.score, &mut self.winner, &mut self.game_over_timer, &mut self.death_timer);

        Bullet::apply_all(&mut self.bullets, overlaps, world, p1_id, p2_id, &mut self.obstacle, &mut self.ship_p1, &mut self.ship_p2, &mut self.score, &mut self.winner, &mut self.game_over_timer, &mut self.death_timer);
    }

    fn on_collisions(&mut self, _: &HashMap<u16, Vec<(u16, CollisionResult)>>, _: &mut World, _: f32) {}

    fn get_ai_inputs(&self) -> [Vec<f64>; 2] {
        todo!()
    }
}
