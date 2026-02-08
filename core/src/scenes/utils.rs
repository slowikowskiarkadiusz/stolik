use crate::engine::{actor::text::create_text_actor_at_center, color::Color, components::world::World, engine::SCREEN_SIZE, v2::V2};

pub fn print_victory_text(world: &mut World, winner: u8) {
    let text = format!("P{} WON", winner);
    for x in 0..2 {
        let center = &(V2::one() * SCREEN_SIZE as f32 / 2.0) + &(V2::down() * if x == 0 { 1.0 } else { -1.0 } * SCREEN_SIZE as f32 / 4.0);
        let actor_id = create_text_actor_at_center(world, text.clone(), center, Color::white(), None, Some("victory text"));

        if x == 0 {
            if let Some(transform) = world.get_mut_transform(&actor_id) {
                transform.rotation = 180.0;
            }
        }
    }
}
