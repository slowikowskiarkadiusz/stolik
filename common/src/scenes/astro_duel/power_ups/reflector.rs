use libm::{cosf, roundf, sinf};
use crate::engine::{color::Color, color_matrix::ColorMatrix, v2::V2};

pub const REFLECTOR_DURATION: f32 = 10.0;
// Distance from ship center to arc center (just past the nose tip at ~2px from center)
const ARC_DIST: f32 = 3.0;
// Half-width of the arc in pixels
const ARC_HALF_WIDTH: i16 = 3;

pub fn tick(timer: &mut f32, delta_time: f32) -> bool {
    *timer -= delta_time;
    *timer <= 0.0
}

// Draws a short blue arc perpendicular to the ship's forward direction, just past the bow.
pub fn draw_on_ship(ship_center: V2, rotation_deg: f32, result: &mut ColorMatrix) {
    let blue = Color::new(0, 100, 255, 230);
    let rad = rotation_deg.to_radians();
    let fwd_x = sinf(rad);
    let fwd_y = -cosf(rad);
    // Perpendicular (left of forward)
    let perp_x = fwd_y;
    let perp_y = -fwd_x;

    // Arc center: ARC_DIST pixels ahead of ship center
    let cx = ship_center.x + fwd_x * ARC_DIST;
    let cy = ship_center.y + fwd_y * ARC_DIST;

    // Draw a line perpendicular to the bow (arc approximation)
    for i in -ARC_HALF_WIDTH..=ARC_HALF_WIDTH {
        let px = roundf(cx + perp_x * i as f32) as i16;
        let py = roundf(cy + perp_y * i as f32) as i16;
        if px >= 0 && py >= 0 && px < 64 && py < 64 {
            result.set(px as u8, py as u8, blue);
        }
        // Slight curve: draw one step forward for the middle pixels
        if i.abs() <= 1 {
            let px2 = roundf(cx + fwd_x + perp_x * i as f32) as i16;
            let py2 = roundf(cy + fwd_y + perp_y * i as f32) as i16;
            if px2 >= 0 && py2 >= 0 && px2 < 64 && py2 < 64 {
                result.set(px2 as u8, py2 as u8, blue);
            }
        }
    }
}
