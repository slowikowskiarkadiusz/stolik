use crate::engine::{
    color::Color,
    color_matrix::ColorMatrix,
    components::{
        collider::{Collider, ColliderPart, ColliderType},
        transform::Transform,
        world::World,
    },
    engine::ActorId,
    v2::V2,
};
extern crate alloc;
use alloc::vec;

pub fn create_rectangle_actor(world: &mut World, center: V2, size: V2, color: Color, collider_type: Option<ColliderType>, name: Option<&str>) -> ActorId {
    world.add_new_actor(
        Some(Transform::new(center, size.clone())),
        if let Some(col_type) = collider_type {
            Some(Collider::new(vec![ColliderPart { offset: V2::zero(), extend: size.clone(), is_overlap: col_type == ColliderType::Blocking }], Some(0)))
        }
        else {None},
        None,
        None,
        Some(ColorMatrix::new(size.x as u8, size.y as u8, color)),
    )
}
