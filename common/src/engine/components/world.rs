extern crate alloc;

use crate::engine::{
    color_matrix::ColorMatrix,
    components::{
        blinker::Blinker, camera::Camera, collider::{Collider, CollisionMask, CollisionMaskId}, physics::Physics, transform::Transform
    },
    engine::ActorId,
};

pub const MAX_ACTORS: usize = 32;

pub struct World {
    camera: Camera,
    pub all_actors: [ActorId; MAX_ACTORS],
    pub actor_count: usize,
    actor_alive: [bool; MAX_ACTORS],
    transforms: [Option<Transform>; MAX_ACTORS],
    colliders: [Option<Collider>; MAX_ACTORS],
    physics: [Option<Physics>; MAX_ACTORS],
    pub(crate) blinkers: [Option<Blinker>; MAX_ACTORS],
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
            blinkers: core::array::from_fn(|_| None),
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

    pub fn get_mut_collider(&mut self, actor_id: &ActorId) -> Option<&mut Collider> {
        self.colliders[*actor_id as usize].as_mut()
    }

    pub fn get_blinker(&self, actor_id: &ActorId) -> Option<&Blinker> {
        self.blinkers[*actor_id as usize].as_ref()
    }

    pub fn get_mut_blinker(&mut self, actor_id: &ActorId) -> Option<&mut Blinker> {
        self.blinkers[*actor_id as usize].as_mut()
    }

    pub fn add_new_actor(
        &mut self,
        transform: Option<Transform>,
        collider: Option<Collider>,
        physics: Option<Physics>,
        blinker: Option<Blinker>,
    ) -> ActorId {
        let slot = (0..MAX_ACTORS).find(|&i| !self.actor_alive[i]).expect("MAX_ACTORS exceeded") as ActorId;

        let idx = slot as usize;
        self.actor_alive[idx] = true;
        self.all_actors[self.actor_count] = slot;
        self.actor_count += 1;

        self.transforms[idx] = transform;
        self.colliders[idx] = collider;
        self.physics[idx] = physics;
        self.blinkers[idx] = blinker;

        slot
    }

    pub fn remove_actor(&mut self, actor_id: &ActorId) {
        let idx = *actor_id as usize;
        self.actor_alive[idx] = false;
        self.transforms[idx] = None;
        self.colliders[idx] = None;
        self.physics[idx] = None;
        self.blinkers[idx] = None;

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
            self.blinkers[i] = None;
        }
    }

    /// Ticks all blinker components — called by the engine each frame.
    pub fn tick_blinkers(&mut self, delta_time: f32) {
        for i in 0..self.actor_count {
            let actor_id = self.all_actors[i];
            if let Some(blinker) = self.blinkers[actor_id as usize].as_mut() {
                blinker.timer += delta_time;
                if blinker.timer >= (blinker.delay_ms as f32 / 1000.0) {
                    blinker.is_on = !blinker.is_on;
                    blinker.timer = 0.0;
                }
            }
        }
    }
}
