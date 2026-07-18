extern crate alloc;
use alloc::vec::Vec;

use crate::engine::{
    components::{
        camera::{Camera, Viewport},
        collider::{Collider, ColliderPartDebug, CollisionMask, CollisionMaskId},
        physics::Physics,
        transform::Transform,
    },
    engine::ActorId, v2::V2,
};

pub const MAX_ACTORS: usize = 90;

pub struct World {
    camera: Camera,
    pub all_actors: [ActorId; MAX_ACTORS],
    pub actor_count: usize,
    actor_alive: [bool; MAX_ACTORS],
    transforms: [Option<Transform>; MAX_ACTORS],
    colliders: [Option<Collider>; MAX_ACTORS],
    physics: [Option<Physics>; MAX_ACTORS],
    collision_matrix: [CollisionMask; CollisionMaskId::MAX as usize],
}

impl World {
    pub fn new() -> Self {
        Self {
            camera: Camera::new(),
            all_actors: [0; MAX_ACTORS],
            actor_count: 0,
            actor_alive: [false; MAX_ACTORS],
            transforms: core::array::from_fn(|_| None),
            colliders: core::array::from_fn(|_| None),
            physics: core::array::from_fn(|_| None),
            collision_matrix: [CollisionMask::MAX; CollisionMaskId::MAX as usize],
        }
    }

    pub fn get_camera(&self) -> Camera {
        self.camera.clone()
    }

    pub fn get_mut_camera(&mut self) -> &mut Camera {
        &mut self.camera
    }

    pub fn get_actors(&self) -> &[ActorId] {
        &self.all_actors[..self.actor_count]
    }

    pub fn get_collision_matrix(&self, index: CollisionMaskId) -> CollisionMask {
        self.collision_matrix[index as usize]
    }

    pub fn set_collisions_on(&mut self, first: CollisionMaskId, second: CollisionMaskId, on: bool) {
        let new_value = if on { 1 } else { 0 };
        self.collision_matrix[first as usize] |= new_value << second;
        self.collision_matrix[second as usize] |= new_value << first;
    }

    pub fn get_transform(&self, actor_id: &ActorId) -> Option<&Transform> {
        self.transforms[*actor_id as usize].as_ref()
    }

    pub fn get_mut_transform(&mut self, actor_id: &ActorId) -> Option<&mut Transform> {
        self.transforms[*actor_id as usize].as_mut()
    }

    pub fn get_physics(&self, actor_id: &ActorId) -> Option<&Physics> {
        self.physics[*actor_id as usize].as_ref()
    }

    pub fn get_mut_physics(&mut self, actor_id: &ActorId) -> Option<&mut Physics> {
        self.physics[*actor_id as usize].as_mut()
    }

    pub fn get_collider(&self, actor_id: &ActorId) -> Option<&Collider> {
        self.colliders[*actor_id as usize].as_ref()
    }

    pub fn _debug_get_collider_parts(&self, viewport: Viewport) -> Vec<ColliderPartDebug> {
        self.colliders
            .iter()
            .zip(self.transforms.iter())
            .filter_map(|(c, t)| match (c, t) {
                (Some(collider), Some(transform)) => Some((collider, transform)),
                _ => None,
            })
            .flat_map(|(collider, transform)| {
                collider.collider_parts.iter().map(move |part| {
                    let offset = if transform.rotation != 0.0 {
                        part.offset.rotate(transform.rotation)
                    } else {
                        part.offset
                    };

                    ColliderPartDebug {
                        center: &V2::new(transform.center.x + offset.x, transform.center.y + offset.y) - &viewport.from,
                        extend: part.extend,
                        shape: part.shape,
                        is_overlap: part.is_overlap,
                    }
                })
            })
            .collect()
    }

    pub fn get_mut_collider(&mut self, actor_id: &ActorId) -> Option<&mut Collider> {
        self.colliders[*actor_id as usize].as_mut()
    }

    pub fn add_new_actor(
        &mut self,
        transform: Option<Transform>,
        collider: Option<Collider>,
        physics: Option<Physics>,
    ) -> ActorId {
        let slot = (0..MAX_ACTORS).find(|&i| !self.actor_alive[i]).expect("MAX_ACTORS exceeded") as ActorId;

        let idx = slot as usize;
        self.actor_alive[idx] = true;
        self.all_actors[self.actor_count] = slot;
        self.actor_count += 1;

        self.transforms[idx] = transform;
        self.colliders[idx] = collider;
        self.physics[idx] = physics;

        slot
    }

    pub fn murder(&mut self, actor_id: &ActorId) {
        let idx = *actor_id as usize;
        self.actor_alive[idx] = false;
        self.transforms[idx] = None;
        self.colliders[idx] = None;
        self.physics[idx] = None;

        if let Some(pos) = self.all_actors[..self.actor_count].iter().position(|&id| id == *actor_id) {
            self.actor_count -= 1;
            self.all_actors[pos] = self.all_actors[self.actor_count];
        }
    }

    pub fn clear_all(&mut self) {
        self.actor_count = 0;
        self.actor_alive = [false; MAX_ACTORS];
        for i in 0..MAX_ACTORS {
            self.transforms[i] = None;
            self.colliders[i] = None;
            self.physics[i] = None;
        }
    }
}
