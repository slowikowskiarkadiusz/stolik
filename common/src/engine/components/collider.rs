extern crate alloc;
use alloc::vec::Vec;

use crate::engine::{
    components::{transform::Transform, world::World},
    engine::ActorId,
    hash_map::HashMap,
    v2::V2,
};

pub type CollisionMask = u8;
pub type CollisionMaskId = u8;

#[derive(PartialEq, Eq)]
pub enum ColliderType {
    Blocking,
    Overlapping,
}

pub struct CollisionResult {
    pub normal: V2,
    pub penetration: f32,
    /// World-space contact point: deepest penetrating vertex (or edge midpoint) of shape B into A.
    pub contact_point: V2,
    pub is_overlap: bool,
}

pub struct ColliderPart {
    pub offset: V2,
    pub extend: V2,
    pub is_overlap: bool,
}

pub struct Collider {
    pub collider_parts: Vec<ColliderPart>,
    pub mask_id: CollisionMaskId,
}

impl Collider {
    pub fn new(collider_parts: Vec<ColliderPart>, mask_id: Option<CollisionMaskId>) -> Self {
        Self {
            collider_parts,
            mask_id: mask_id.unwrap_or_else(|| 0),
        }
    }

    pub fn detect_overlaps(world: &World) -> HashMap<u16, Vec<u16>> {
        let mut dict = HashMap::<ActorId, Vec<ActorId>>::new();
        let actors = world.get_actors();
        for (ai, first_actor) in actors.iter().enumerate() {
            for second_actor in &actors[ai + 1..] {
                if first_actor != second_actor
                    && let Some(first_collider) = world.get_collider(first_actor)
                    && let Some(first_transform) = world.get_transform(first_actor)
                    && let Some(second_collider) = world.get_collider(second_actor)
                    && let Some(second_transform) = world.get_transform(second_actor)
                    && (!dict.contains_key(first_actor) || !dict[first_actor].contains(second_actor))
                {
                    if (world.get_collision_matrix(first_collider.mask_id) & 1 << second_collider.mask_id) == 1
                        && Collider::is_overlapping((first_collider, first_transform), (second_collider, second_transform))
                    {
                        if !dict.contains_key(first_actor) {
                            dict.insert(first_actor.clone(), Vec::new());
                        }

                        if !dict.contains_key(second_actor) {
                            dict.insert(second_actor.clone(), Vec::new());
                        }

                        dict.get_mut(first_actor).unwrap().push(second_actor.clone());
                        dict.get_mut(second_actor).unwrap().push(first_actor.clone());
                    }
                }
            }
        }

        dict
    }

    pub fn detect_collisions(world: &World) -> HashMap<u16, Vec<(u16, CollisionResult)>> {
        let mut dict = HashMap::<ActorId, Vec<(ActorId, CollisionResult)>>::new();
        let actors = world.get_actors();
        for (ai, first_actor) in actors.iter().enumerate() {
            for second_actor in &actors[ai + 1..] {
                if first_actor != second_actor
                    && let Some(first_collider) = world.get_collider(first_actor)
                    && let Some(first_transform) = world.get_transform(first_actor)
                    && let Some(second_collider) = world.get_collider(second_actor)
                    && let Some(second_transform) = world.get_transform(second_actor)
                {
                    if (world.get_collision_matrix(first_collider.mask_id) & 1 << second_collider.mask_id) == 1 {
                        if let Some(result) =
                            Collider::get_collision_result((first_collider, first_transform), (second_collider, second_transform))
                        {
                            if !dict.contains_key(first_actor) {
                                dict.insert(first_actor.clone(), Vec::new());
                            }
                            if !dict.contains_key(second_actor) {
                                dict.insert(second_actor.clone(), Vec::new());
                            }

                            let flipped = CollisionResult {
                                normal: V2::new(-result.normal.x, -result.normal.y),
                                penetration: result.penetration,
                                contact_point: result.contact_point,
                                is_overlap: result.is_overlap,
                            };

                            dict.get_mut(first_actor).unwrap().push((second_actor.clone(), result));
                            dict.get_mut(second_actor).unwrap().push((first_actor.clone(), flipped));
                        }
                    }
                }
            }
        }

        dict
    }

    pub fn is_empty(&self) -> bool {
        self.collider_parts.is_empty()
    }

    fn get_parts_vertices(part: &ColliderPart) -> [V2; 4] {
        [
            V2::new(part.extend.x / 2.0, part.extend.y / 2.0),
            V2::new(part.extend.x / 2.0, -part.extend.y / 2.0),
            V2::new(-part.extend.x / 2.0, -part.extend.y / 2.0),
            V2::new(-part.extend.x / 2.0, part.extend.y / 2.0),
        ]
    }

    fn project_shape(verts: &[V2], axis: &V2) -> (f32, f32) {
        let mut min = f32::MAX;
        let mut max = f32::MIN;
        for v in verts {
            let d = v.dot(axis);
            if d < min {
                min = d;
            }
            if d > max {
                max = d;
            }
        }
        (min, max)
    }

    fn sat_test(
        first_part: &ColliderPart, first_center: &V2, first_rotation: f32,
        second_part: &ColliderPart, second_center: &V2, second_rotation: f32,
    ) -> Option<CollisionResult> {
        // Rotate local vertices (including their offset) by the transform rotation, then
        // translate to world space. Offset=0 is the common case so the `if` is a fast path.
        let first_verts = Collider::get_parts_vertices(first_part).map(|v| {
            let local = &v + &first_part.offset;
            let rotated = if first_rotation != 0.0 { local.rotate(first_rotation) } else { local };
            &rotated + first_center
        });
        let second_verts = Collider::get_parts_vertices(second_part).map(|v| {
            let local = &v + &second_part.offset;
            let rotated = if second_rotation != 0.0 { local.rotate(second_rotation) } else { local };
            &rotated + second_center
        });

        let mut min_penetration = f32::MAX;
        let mut best_normal = V2::new(0.0, 0.0);

        for verts in [first_verts.as_slice(), second_verts.as_slice()] {
            for i in 0..verts.len() {
                let j = (i + 1) % verts.len();
                let edge = &verts[j] - &verts[i];
                let axis = V2::new(-edge.y, edge.x).norm();

                let proj_first = Collider::project_shape(&first_verts, &axis);
                let proj_second = Collider::project_shape(&second_verts, &axis);

                let overlap = f32::min(proj_first.1, proj_second.1) - f32::max(proj_first.0, proj_second.0);

                if overlap <= 0.0 {
                    return None;
                }

                if overlap < min_penetration {
                    min_penetration = overlap;
                    best_normal = axis;
                }
            }
        }

        let first_offset_rot = if first_rotation != 0.0 { first_part.offset.rotate(first_rotation) } else { first_part.offset };
        let second_offset_rot = if second_rotation != 0.0 { second_part.offset.rotate(second_rotation) } else { second_part.offset };
        let first_center_eff = V2::new(first_center.x + first_offset_rot.x, first_center.y + first_offset_rot.y);
        let second_center_eff = V2::new(second_center.x + second_offset_rot.x, second_center.y + second_offset_rot.y);
        let dir = &second_center_eff - &first_center_eff;
        if dir.dot(&best_normal) < 0.0 {
            best_normal = V2::new(-best_normal.x, -best_normal.y);
        }

        let min_proj = second_verts.iter().map(|v| v.dot(&best_normal)).fold(f32::MAX, f32::min);
        let mut cx = 0.0;
        let mut cy = 0.0;
        let mut cnt = 0;
        for v in &second_verts {
            if v.dot(&best_normal) <= min_proj + 0.01 {
                cx += v.x;
                cy += v.y;
                cnt += 1;
            }
        }
        let contact_point = V2::new(cx / cnt as f32, cy / cnt as f32);

        Some(CollisionResult {
            normal: best_normal,
            penetration: min_penetration,
            contact_point,
            is_overlap: first_part.is_overlap || second_part.is_overlap,
        })
    }

    pub fn get_collision_result(first: (&Collider, &Transform), second: (&Collider, &Transform)) -> Option<CollisionResult> {
        if !Collider::are_in_colliding_distance(first.0, &first.1.center, second.0, &second.1.center) {
            return None;
        }

        let mut result: Option<CollisionResult> = None;

        for first_part in &first.0.collider_parts {
            for second_part in &second.0.collider_parts {
                if let Some(r) = Collider::sat_test(first_part, &first.1.center, first.1.rotation, second_part, &second.1.center, second.1.rotation) {
                    match &result {
                        None => result = Some(r),
                        Some(prev) if r.penetration < prev.penetration => result = Some(r),
                        _ => {}
                    }
                }
            }
        }

        result
    }

    pub fn is_overlapping(first: (&Collider, &Transform), second: (&Collider, &Transform)) -> bool {
        Collider::get_collision_result(first, second).is_some()
    }

    pub fn are_in_colliding_distance(
        first_collider: &Collider,
        first_position: &V2,
        second_collider: &Collider,
        second_position: &V2,
    ) -> bool {
        fn get_reach(collider: &Collider) -> f32 {
            collider
                .collider_parts
                .iter()
                .flat_map(|f| Collider::get_parts_vertices(f).map(|g| &g + &f.offset))
                .map(|f| f.mag())
                .reduce(|f, g| if f > g { f } else { g })
                .unwrap()
        }

        if first_collider.is_empty() || second_collider.is_empty() {
            return false;
        }

        first_position.distance(second_position) < get_reach(first_collider) + get_reach(second_collider)
    }
}
