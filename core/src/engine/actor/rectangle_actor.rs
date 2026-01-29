use crate::engine::{
    color::Color,
    color_matrix::ColorMatrix,
    components::{transform::Transform, world::World},
    engine::ActorId,
    v2::V2,
};

pub fn create_rectangle_actor(world: &mut World, center: V2, size: V2, color: Color, name: Option<String>) -> ActorId {
    world.add_new_actor(
        name.or_else(|| Some(String::from("rectangle"))),
        Some(Transform::new(center, size.clone())),
        None,
        None,
        Some(ColorMatrix::new(size.x as u8, size.y as u8, color)),
    )
}
