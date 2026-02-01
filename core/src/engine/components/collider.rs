use std::{collections::HashMap, u8};

use crate::engine::{
    components::{transform::Transform, world::World},
    engine::ActorId,
    v2::V2,
};

pub type CollisionMask = u8;
pub type CollisionMaskId = u8;

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
        for first_actor in &world.all_actors {
            for i in (first_actor.clone() as usize + 1)..world.all_actors.len() {
                let second_actor = &world.all_actors[i];
                if first_actor == second_actor || dict[first_actor].contains(second_actor) {
                    continue;
                }

                if let Some(first_collider) = world.get_collider(first_actor)
                    && let Some(first_transform) = world.get_transform(first_actor)
                    && let Some(second_collider) = world.get_collider(second_actor)
                    && let Some(second_transform) = world.get_transform(second_actor)
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

    pub fn is_overlapping(first: (&Collider, &Transform), second: (&Collider, &Transform)) -> bool {
        fn ccw(a: &V2, b: &V2, c: &V2) -> bool {
            (c.y - a.y) * (b.x - a.x) > (b.y - a.y) * (c.x - a.x)
        }

        fn do_intersect(a: &V2, b: &V2, c: &V2, d: &V2) -> bool {
            ccw(&a, &c, &d) != ccw(&b, &c, &d) && ccw(&a, &b, &c) != ccw(&a, &b, &d)
        }

        fn is_part_overlapping(first_part: &ColliderPart, first_center: &V2, second_part: &ColliderPart, second_center: &V2) -> bool {
            let first_vertices = Collider::get_parts_vertices(first_part).map(|x| &x + first_center);
            let second_vertices = Collider::get_parts_vertices(second_part).map(|x| &x + second_center);

            let min_first_vertex = &first_vertices.iter().reduce(|a, b| if a.x < b.x { a } else { b }).unwrap();
            for s_v in second_vertices {
                let mut intersects: u8 = 0;

                for i in 0..first_vertices.len() {
                    let j = (i + 1) % &first_vertices.len();

                    if do_intersect(&V2::new(min_first_vertex.x - 1.0, s_v.y), &s_v, &first_vertices[i], &first_vertices[j]) {
                        intersects += 1;
                    }
                }

                if intersects > 0 && intersects % 2 == 1 {
                    return true;
                }
            }

            false
        }

        if !Collider::are_in_colliding_distance(first.0, &first.1.center, second.0, &second.1.center) {
            return false;
        }

        for first_part in &first.0.collider_parts {
            for second_part in &second.0.collider_parts {
                if is_part_overlapping(&first_part, &first.1.center, &second_part, &second.1.center) {
                    return true;
                }
            }
        }

        false
    }

    pub fn are_in_colliding_distance(first_collider: &Collider, first_position: &V2, second_collider: &Collider, second_position: &V2) -> bool {
        fn get_reach(collider: &Collider) -> f32 {
            collider
                .collider_parts
                .iter()
                .flat_map(|f| Collider::get_parts_vertices(f).map(|g| &g + &f.offset))
                .map(|f| f.mag())
                .reduce(|f, g| if f < g { f } else { g })
                .unwrap()
        }

        if first_collider.is_empty() || second_collider.is_empty() {
            return false;
        }

        first_position.distance(second_position) < get_reach(first_collider) + get_reach(second_collider)
    }
}
