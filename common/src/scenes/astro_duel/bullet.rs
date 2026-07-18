extern crate alloc;
use alloc::vec::Vec;

use crate::engine::{
    color_matrix::ColorMatrix,
    components::{
        camera::Camera,
        collider::{Collider, ColliderPart},
        physics::Physics,
        transform::Transform,
        world::World,
    },
    engine::{ActorId, SCREEN_SIZEF32},
    hash_map::HashMap,
    v2::V2,
};

use super::astro_obstacle::AstroObstacleMap;
use super::ship::Ship;
use super::scoring::apply_hit;
use crate::scenes::utils::{P1_COLOR, P2_COLOR};

const BULLET_LIFETIME: f32 = 2.5;
pub const BULLET_SIZE: f32 = 2.0;

pub struct Bullet {
    pub actor_id: ActorId,
    pub owner_is_p1: bool,
    pub hit: bool,
    lifetime: f32,
}

impl Bullet {
    pub fn new(world: &mut World, pos: V2, velocity: V2, owner_is_p1: bool) -> Self {
        let mut physics = Physics::new();
        physics.with_can_move(true).with_mass(0.1).with_drag(0.0);

        let id = world.add_new_actor(
            Some(Transform::new(pos, V2::new(BULLET_SIZE, BULLET_SIZE))),
            Some(Collider::new(
                alloc::vec![ColliderPart::rect(V2::zero(), V2::new(BULLET_SIZE, BULLET_SIZE), true)],
                Some(0),
                false
            )),
            Some(physics),
            None,
        );
        world.get_mut_physics(&id).unwrap().set_velocity(velocity);

        Self {
            actor_id: id,
            owner_is_p1,
            hit: false,
            lifetime: BULLET_LIFETIME,
        }
    }

    /// Returns true if bullet should be destroyed.
    pub fn tick(&mut self, world: &mut World, _obstacle: &AstroObstacleMap, delta_time: f32) -> bool {
        self.lifetime -= delta_time;
        wrap_center(world, self.actor_id);
        self.hit || self.lifetime <= 0.0
    }

    pub fn destroy(&self, world: &mut World) {
        world.murder(&self.actor_id);
    }

    pub fn apply_all(
        bullets: &mut Vec<Bullet>,
        overlaps: &HashMap<u16, Vec<u16>>,
        world: &mut World,
        p1_id: Option<ActorId>,
        p2_id: Option<ActorId>,
        obstacle: &mut Option<AstroObstacleMap>,
        ship_p1: &mut Option<Ship>,
        ship_p2: &mut Option<Ship>,
        score: &mut [u8; 2],
        winner: &mut Option<u8>,
        game_over_timer: &mut f32,
        death_timer: &mut Option<(f32, usize)>,
    ) {
        let bullet_actor_ids: Vec<ActorId> = bullets.iter().map(|b| b.actor_id).collect();

        for i in 0..bullets.len() {
            if bullets[i].hit { continue; }
            let bullet_id = bullets[i].actor_id;
            let owner_is_p1 = bullets[i].owner_is_p1;
            let Some(list) = overlaps.get(&bullet_id) else { continue };
            let overlapped: Vec<ActorId> = list.iter().copied().collect();

            for overlapped_id in overlapped {
                if let Some(j) = bullet_actor_ids.iter().position(|&id| id == overlapped_id) {
                    if j != i && !bullets[j].hit {
                        bullets[i].hit = true;
                        bullets[j].hit = true;
                    }
                    continue;
                }

                let hit_p1_ship = p1_id == Some(overlapped_id);
                let hit_p2_ship = p2_id == Some(overlapped_id);

                if hit_p1_ship {
                    bullets[i].hit = true;
                    let scorer = if owner_is_p1 {
                        if score[0] > 0 { score[0] -= 1; }
                        None
                    } else {
                        Some(1usize)
                    };
                    apply_hit(ship_p1, score, winner, game_over_timer, death_timer, scorer.unwrap_or(usize::MAX));
                } else if hit_p2_ship {
                    bullets[i].hit = true;
                    let scorer = if !owner_is_p1 {
                        if score[1] > 0 { score[1] -= 1; }
                        None
                    } else {
                        Some(0usize)
                    };
                    apply_hit(ship_p2, score, winner, game_over_timer, death_timer, scorer.unwrap_or(usize::MAX));
                } else {
                    let is_destroyable = obstacle.as_ref()
                        .map(|o| o.is_destroyable_obstacle(&overlapped_id))
                        .unwrap_or(false);
                    if is_destroyable {
                        if let Some(o) = obstacle.as_mut() { o.remove(&overlapped_id); }
                        world.murder(&overlapped_id);
                        bullets[i].hit = true;
                    }
                    let is_border = obstacle.as_ref()
                        .map(|o| o.is_border_obstacle(&overlapped_id))
                        .unwrap_or(false);
                    if is_border {
                        bullets[i].hit = true;
                    }
                }
            }
        }
    }

    pub fn render(&self, world: &World, _camera: &Camera, result: &mut ColorMatrix) {
        if let Some(t) = world.get_transform(&self.actor_id) {
            let color = if self.owner_is_p1 { P1_COLOR } else { P2_COLOR };
            result.write(
                &ColorMatrix::new(BULLET_SIZE as u8, BULLET_SIZE as u8, color),
                &t.center,
                None,
                None,
                None,
                None,
            );
        }
    }
}

pub fn wrap_center(world: &mut World, actor_id: ActorId) {
    if let Some(t) = world.get_mut_transform(&actor_id) {
        let s = SCREEN_SIZEF32;
        t.center.x = ((t.center.x % s) + s) % s;
        t.center.y = ((t.center.y % s) + s) % s;
    }
}
