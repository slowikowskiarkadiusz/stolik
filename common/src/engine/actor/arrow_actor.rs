use libm::ceilf;

use crate::engine::{
    color::Color,
    color_matrix::ColorMatrix,
    components::{blinker::Blinker, transform::Transform, world::World},
    engine::ActorId,
    v2::V2,
};

pub fn create_arrow_actor(world: &mut World, center: V2, height: u8, color: Color, delay_ms: u32, name: Option<&str>) -> ActorId {
    let shape = make_shape(height, color);
    world.add_new_actor(
        Some(Transform::new(center, V2::new(shape.width as f32, shape.height as f32))),
        None,
        None,
        // Some(Blinker::new(delay_ms)),
        None,
        Some(shape),
    )
}

fn make_shape(height: u8, color: Color) -> ColorMatrix {
    let mut matrix = ColorMatrix::new(ceilf(height as f32 / 2.0) as u8, height, Color::none());

    for y in 0..matrix.height / 2 + 1 {
        matrix.set(0, y, color.clone());

        for x in 1..=y {
            matrix.set(x, y, color.clone());
            matrix.set(x, height - y - 1, color.clone());
        }

        matrix.set(0, height - y - 1, color.clone());
    }

    matrix
}
