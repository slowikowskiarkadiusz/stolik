use crate::engine::{
    actor::text::render_text,
    color::Color,
    color_matrix::ColorMatrix,
    components::{camera::Camera, world::World},
    engine::SCREEN_SIZE,
    v2::V2,
};

extern crate alloc;
use alloc::format;

pub fn print_victory_text(world: &mut World, winner: u8, camera: &Camera, result: &mut ColorMatrix) {
    let text = format!("P{} WON", winner);
    for x in 0..2 {
        let center = &(V2::one() * SCREEN_SIZE as f32 / 2.0) + &(V2::down() * if x == 0 { 1.0 } else { -1.0 } * SCREEN_SIZE as f32 / 4.0);
        render_text(
            world,
            text.clone(),
            center,
            V2::one() * SCREEN_SIZE as f32,
            None,
            None,
            Color::white(),
            camera,
            result,
        );

        if x == 0 {
            result.write(&result.clone(), &(result.get_size() / 2.0), Some(180.0), None, None, Some(camera));
        }
    }
}
