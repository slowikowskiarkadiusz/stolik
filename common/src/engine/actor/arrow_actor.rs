use libm::ceilf;

use crate::engine::{
    color::Color,
    color_matrix::ColorMatrix,
    components::{camera::Camera, world::World},
    v2::V2,
};
use crate::write_m;

pub fn render_arrow(_world: &mut World, center: V2, height: u8, color: Color, _delay_ms: u32, camera: &Camera, result: &mut ColorMatrix) {
    let shape = make_shape(height, color);
    write_m!(result, &shape, &center, camera);
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
