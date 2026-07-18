extern crate alloc;
use alloc::vec::Vec;

use libm::{cosf, roundf, sinf};
use crate::engine::{color::Color, color_matrix::ColorMatrix, components::world::World, v2::V2};
use crate::scenes::astro_duel::power_ups::power_up::PowerUpType;
use crate::scenes::astro_duel::ship::Ship;
use crate::scenes::astro_duel::bullet::Bullet;

pub const REFLECTOR_DURATION: f32 = 10.0;
pub const ARC_DIST: f32 = 3.0;
pub const ARC_HALF_WIDTH: i16 = 3;

pub fn apply(ship_p1: &Option<Ship>, ship_p2: &Option<Ship>, bullets: &mut Vec<Bullet>, world: &mut World) {
    for ship in [ship_p1, ship_p2] {
        let Some(ship) = ship else { continue };
        if ship.active_power_up != Some(PowerUpType::Reflector) {
            continue;
        }
        let Some(center) = world.get_transform(&ship.actor_id).map(|t| t.center) else {
            continue;
        };
        let (la, lb) = line_endpoints(center, ship.rotation as f32, -1.0);
        for bullet in bullets.iter_mut() {
            if bullet.hit {
                continue;
            }
            let Some(bpos) = world.get_transform(&bullet.actor_id).map(|t| t.center) else {
                continue;
            };
            if point_to_seg_dist(bpos, la, lb) <= 1.5 {
                if let Some(phys) = world.get_mut_physics(&bullet.actor_id) {
                    let vel = *phys.get_velocity();
                    phys.set_velocity(vel * -1.0);
                }
            }
        }
    }
}

fn point_to_seg_dist(p: V2, a: V2, b: V2) -> f32 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let len_sq = dx * dx + dy * dy;
    if len_sq == 0.0 {
        return (p - a).mag();
    }
    let t = (((p.x - a.x) * dx + (p.y - a.y) * dy) / len_sq).max(0.0).min(1.0);
    let proj = V2::new(a.x + t * dx, a.y + t * dy);
    (p - proj).mag()
}

pub fn tick(timer: &mut f32, delta_time: f32) -> bool {
    *timer -= delta_time;
    *timer <= 0.0
}

/// Returns (line_start, line_end) endpoints of the reflector line in world space.
/// `fwd_offset` shifts the line along the forward axis (negative = toward ship).
pub fn line_endpoints(ship_center: V2, rotation_deg: f32, fwd_offset: f32) -> (V2, V2) {
    let rad = rotation_deg.to_radians();
    let fwd = V2::new(sinf(rad), -cosf(rad));
    let perp = V2::new(fwd.y, -fwd.x);
    let dist = ARC_DIST + fwd_offset;
    let center = V2::new(ship_center.x + fwd.x * dist, ship_center.y + fwd.y * dist);
    let half = ARC_HALF_WIDTH as f32;
    (
        V2::new(center.x - perp.x * half, center.y - perp.y * half),
        V2::new(center.x + perp.x * half, center.y + perp.y * half),
    )
}

/// Gradient line perpendicular to the bow: max alpha at center 2px, fading to 150 at edges.
pub fn draw_on_ship(ship_center: V2, rotation_deg: f32, result: &mut ColorMatrix) {
    let rad = rotation_deg.to_radians();
    let fwd_x = sinf(rad);
    let fwd_y = -cosf(rad);
    let perp_x = fwd_y;
    let perp_y = -fwd_x;

    let cx = ship_center.x + fwd_x * ARC_DIST;
    let cy = ship_center.y + fwd_y * ARC_DIST;

    let half = ARC_HALF_WIDTH as f32;
    for i in -ARC_HALF_WIDTH..=ARC_HALF_WIDTH {
        let t = i.abs() as f32 / half; // 0 at center, 1 at edge
        let alpha = (255.0 - (255.0 - 150.0) * t) as u8;
        let px = roundf(cx + perp_x * i as f32) as i16;
        let py = roundf(cy + perp_y * i as f32) as i16;
        if px >= 0 && py >= 0 && px < 64 && py < 64 {
            result.set(px as u8, py as u8, Color::new(0, 100, 255, alpha));
        }
    }
}
