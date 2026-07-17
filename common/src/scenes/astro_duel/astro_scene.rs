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
    power_ups::{PlacedMine, PowerUp, RayGunBlast},
    ship::{Ship, ShipAction},
};

const GAME_OVER_DELAY: f32 = 4.0;
const DEATH_DELAY: f32 = 2.5;
const WIN_SCORE: u8 = 5;
const POWER_UP_RESPAWN_DELAY: f32 = 5.0;

pub struct AstroDuelScene {
    obstacle: Option<AstroObstacleMap>,
    ship_p1: Option<Ship>,
    ship_p2: Option<Ship>,
    bullets: Vec<Bullet>,
    power_ups: Vec<PowerUp>,
    placed_mines: Vec<PlacedMine>,
    ray_gun_blasts: Vec<RayGunBlast>,
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

        // --- Power-up pickup ---
        let pu_ids: Vec<(ActorId, usize)> = self.power_ups.iter().enumerate()
            .map(|(i, pu)| (pu.actor_id, i))
            .collect();

        let mut picked_pu: Option<usize> = None;
        'pu_outer: for &(pu_actor, pu_idx) in &pu_ids {
            for ship_opt in [&self.ship_p1, &self.ship_p2] {
                if let Some(ship) = ship_opt {
                    if let Some(list) = overlaps.get(&ship.actor_id) {
                        if list.contains(&pu_actor) {
                            picked_pu = Some(pu_idx);
                            break 'pu_outer;
                        }
                    }
                    if let Some(list) = overlaps.get(&pu_actor) {
                        if list.contains(&ship.actor_id) {
                            picked_pu = Some(pu_idx);
                            break 'pu_outer;
                        }
                    }
                }
            }
        }

        if let Some(idx) = picked_pu {
            let pu = self.power_ups.remove(idx);
            // Give to whichever ship overlapped it
            let pu_type = pu.power_up_type();
            world.murder(&pu.actor_id);
            for ship_opt in [&mut self.ship_p1, &mut self.ship_p2] {
                if let Some(ship) = ship_opt {
                    if ship.give_power_up(pu_type) {
                        break;
                    }
                }
            }
        }

        // --- Mine: trigger detection and detonation damage ---
        let p1_center = Self::ship_center(&self.ship_p1, world);
        let p2_center = Self::ship_center(&self.ship_p2, world);
        for mine in &mut self.placed_mines {
            if mine.detonated { continue; }
            // Trigger: ship enters detection radius while mine is active and not yet triggered
            if mine.is_active && !mine.triggered {
                let p1_near = p1_center.map(|c| mine.in_blast_radius(c)).unwrap_or(false);
                let p2_near = p2_center.map(|c| mine.in_blast_radius(c)).unwrap_or(false);
                if p1_near || p2_near {
                    mine.trigger();
                }
            }
            // Damage: applied on the single frame just_detonated is true
            if mine.just_detonated {
                if p1_center.map(|c| mine.in_blast_radius(c)).unwrap_or(false) {
                    apply_hit(&mut self.ship_p1, &mut self.score, &mut self.winner, &mut self.game_over_timer, &mut self.death_timer, 1);
                }
                if p2_center.map(|c| mine.in_blast_radius(c)).unwrap_or(false) {
                    apply_hit(&mut self.ship_p2, &mut self.score, &mut self.winner, &mut self.game_over_timer, &mut self.death_timer, 0);
                }
            }
        }

        // --- Ray gun hit check (done in tick already via collider_active flag) ---
        let p1_center = Self::ship_center(&self.ship_p1, world);
        let p2_center = Self::ship_center(&self.ship_p2, world);
        for blast in &self.ray_gun_blasts {
            if !blast.collider_active { continue; }
            if p1_center.map(|c| blast.hits(c)).unwrap_or(false) {
                apply_damage(&mut self.ship_p1, 2, &mut self.score, &mut self.winner, &mut self.game_over_timer, &mut self.death_timer, 1);
            }
            if p2_center.map(|c| blast.hits(c)).unwrap_or(false) {
                apply_damage(&mut self.ship_p2, 2, &mut self.score, &mut self.winner, &mut self.game_over_timer, &mut self.death_timer, 0);
            }
        }

        // --- Bullets: any bullet hits any ship ---
        let bullet_actor_ids: Vec<ActorId> = self.bullets.iter().map(|b| b.actor_id).collect();

        for i in 0..self.bullets.len() {
            if self.bullets[i].hit { continue; }
            let bullet_id = self.bullets[i].actor_id;
            let owner_is_p1 = self.bullets[i].owner_is_p1;
            let Some(list) = overlaps.get(&bullet_id) else { continue };
            let overlapped: Vec<ActorId> = list.iter().copied().collect();

            for overlapped_id in overlapped {
                // Bullet vs bullet
                if let Some(j) = bullet_actor_ids.iter().position(|&id| id == overlapped_id) {
                    if j != i && !self.bullets[j].hit {
                        self.bullets[i].hit = true;
                        self.bullets[j].hit = true;
                    }
                    continue;
                }

                // Bullet vs ship
                let hit_p1_ship = p1_id == Some(overlapped_id);
                let hit_p2_ship = p2_id == Some(overlapped_id);

                if hit_p1_ship {
                    self.bullets[i].hit = true;
                    // scorer: if owner_is_p1 (friendly fire) → P1 loses a point, else P2 scores
                    let scorer = if owner_is_p1 {
                        // P1 shot P1 — P1 loses a point (but not below 0)
                        if self.score[0] > 0 { self.score[0] -= 1; }
                        None // no apply_hit, but still damage the ship
                    } else {
                        Some(1usize) // P2 scores
                    };
                    apply_hit(&mut self.ship_p1, &mut self.score, &mut self.winner, &mut self.game_over_timer, &mut self.death_timer, scorer.unwrap_or(usize::MAX));
                } else if hit_p2_ship {
                    self.bullets[i].hit = true;
                    let scorer = if !owner_is_p1 {
                        if self.score[1] > 0 { self.score[1] -= 1; }
                        None
                    } else {
                        Some(0usize) // P1 scores
                    };
                    apply_hit(&mut self.ship_p2, &mut self.score, &mut self.winner, &mut self.game_over_timer, &mut self.death_timer, scorer.unwrap_or(usize::MAX));
                } else {
                    // Obstacle
                    let is_destroyable = self.obstacle.as_ref()
                        .map(|o| o.is_destroyable_obstacle(&overlapped_id))
                        .unwrap_or(false);
                    if is_destroyable {
                        if let Some(o) = self.obstacle.as_mut() { o.remove(&overlapped_id); }
                        world.murder(&overlapped_id);
                        self.bullets[i].hit = true;
                    }
                    let is_border = self.obstacle.as_ref()
                        .map(|o| o.is_border_obstacle(&overlapped_id))
                        .unwrap_or(false);
                    if is_border {
                        self.bullets[i].hit = true;
                    }
                }
            }
        }
    }

    fn on_collisions(&mut self, _: &HashMap<u16, Vec<(u16, CollisionResult)>>, _: &mut World, _: f32) {}
}

fn apply_hit(
    ship: &mut Option<Ship>,
    score: &mut [u8; 2],
    winner: &mut Option<u8>,
    game_over_timer: &mut f32,
    death_timer: &mut Option<(f32, usize)>,
    scorer: usize,
) {
    if let Some(s) = ship {
        let (died, _shield) = s.take_hit();
        if died && scorer < 2 {
            score[scorer] += 1;
            if let Some(w) = check_winner(score) {
                *winner = Some(w);
                *game_over_timer = GAME_OVER_DELAY;
            } else if death_timer.is_none() {
                *death_timer = Some((DEATH_DELAY, scorer));
            }
        }
    }
}

fn apply_damage(
    ship: &mut Option<Ship>,
    amount: u8,
    score: &mut [u8; 2],
    winner: &mut Option<u8>,
    game_over_timer: &mut f32,
    death_timer: &mut Option<(f32, usize)>,
    scorer: usize,
) {
    if let Some(s) = ship {
        let died = s.take_damage(amount);
        if died && scorer < 2 {
            score[scorer] += 1;
            if let Some(w) = check_winner(score) {
                *winner = Some(w);
                *game_over_timer = GAME_OVER_DELAY;
            } else if death_timer.is_none() {
                *death_timer = Some((DEATH_DELAY, scorer));
            }
        }
    }
}
