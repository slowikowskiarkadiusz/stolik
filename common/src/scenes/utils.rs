use crate::engine::{actor::text::create_text_actor_at_center, color::Color, color_matrix::ColorMatrix, components::{camera::Camera, world::World}, engine::SCREEN_SIZE, v2::V2};

extern crate alloc;
use alloc::format;

pub fn print_victory_text(world: &mut World, winner: u8, camera: &Camera, result: &mut ColorMatrix) {
    let text = format!("P{} WON", winner);
    for x in 0..2 {
        let center = &(V2::one() * SCREEN_SIZE as f32 / 2.0) + &(V2::down() * if x == 0 { 1.0 } else { -1.0 } * SCREEN_SIZE as f32 / 4.0);
        let actor_id = create_text_actor_at_center(
            world,
            text.clone(),
            center,
            V2::one() * SCREEN_SIZE as f32,
            None,
            Some(if x == 0 { 180.0 } else { 0.0 }),
            Color::white(),
            camera,
            result,
        );

        if x == 0 {
            if let Some(transform) = world.get_mut_transform(&actor_id) {
                transform.rotation = 180.0;
            }
        }
    }
}
