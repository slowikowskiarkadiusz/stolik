// TODO HashMap

extern crate alloc;
use alloc::{string::String, vec::Vec};

use crate::engine::{
    color_matrix::ColorMatrix,
    components::{
        blinker::Blinker,
        collider::{Collider, CollisionMask, CollisionMaskId},
        physics::Physics,
        transform::Transform,
    },
    engine::ActorId,
    hash_map::HashMap,
};

pub struct World {
    pub all_actors: Vec<ActorId>,
    names: HashMap<ActorId, Option<String>>,
    transforms: HashMap<ActorId, Option<Transform>>,
    colliders: HashMap<ActorId, Option<Collider>>,
    physics: HashMap<ActorId, Option<Physics>>,
    blinkers: HashMap<ActorId, Option<Blinker>>,
    renders: HashMap<ActorId, Option<ColorMatrix>>,

    collision_matrix: [CollisionMask; CollisionMaskId::MAX as usize],
}

impl World {
    pub fn new() -> Self {
        Self {
            all_actors: Vec::new(),
            names: HashMap::new(),
            transforms: HashMap::new(),
            colliders: HashMap::new(),
            physics: HashMap::new(),
            blinkers: HashMap::new(),
            renders: HashMap::new(),

            collision_matrix: [CollisionMask::MAX; CollisionMaskId::MAX as usize],
        }
    }

    pub fn get_collision_matrix(&self, index: CollisionMaskId) -> CollisionMask {
        self.collision_matrix[index as usize]
    }

    pub fn set_collisions_on(&mut self, first: CollisionMaskId, second: CollisionMaskId, on: bool) {
        let new_value = if on { 1 } else { 0 };
        self.collision_matrix[first as usize] |= new_value << second;
        self.collision_matrix[second as usize] |= new_value << first;
    }

    pub fn get_name(&self, actor_id: &ActorId) -> Option<&String> {
        if let Some(r) = self.names.get(actor_id) { r.as_ref() } else { None }
    }

    pub fn get_mut_name(&mut self, actor_id: &ActorId) -> Option<&mut String> {
        if let Some(r) = self.names.get_mut(actor_id) {
            r.as_mut()
        } else {
            None
        }
    }

    pub fn get_transform(&self, actor_id: &ActorId) -> Option<&Transform> {
        if let Some(r) = self.transforms.get(actor_id) {
            r.as_ref()
        } else {
            None
        }
    }

    pub fn get_mut_transform(&mut self, actor_id: &ActorId) -> Option<&mut Transform> {
        if let Some(r) = self.transforms.get_mut(actor_id) {
            r.as_mut()
        } else {
            None
        }
    }

    pub fn get_collider(&self, actor_id: &ActorId) -> Option<&Collider> {
        if let Some(r) = self.colliders.get(actor_id) {
            r.as_ref()
        } else {
            None
        }
    }

    pub fn get_mut_collider(&mut self, actor_id: &ActorId) -> Option<&mut Collider> {
        if let Some(r) = self.colliders.get_mut(actor_id) {
            r.as_mut()
        } else {
            None
        }
    }

    pub fn get_physics(&self, actor_id: &ActorId) -> Option<&Physics> {
        if let Some(r) = self.physics.get(actor_id) {
            r.as_ref()
        } else {
            None
        }
    }

    pub fn get_mut_physics(&mut self, actor_id: &ActorId) -> Option<&mut Physics> {
        if let Some(r) = self.physics.get_mut(actor_id) {
            r.as_mut()
        } else {
            None
        }
    }

    pub fn get_blinker(&self, actor_id: &ActorId) -> Option<&Blinker> {
        if let Some(r) = self.blinkers.get(actor_id) {
            r.as_ref()
        } else {
            None
        }
    }

    pub fn get_mut_blinker(&mut self, actor_id: &ActorId) -> Option<&mut Blinker> {
        if let Some(r) = self.blinkers.get_mut(actor_id) {
            r.as_mut()
        } else {
            None
        }
    }

    pub fn get_render(&self, actor_id: &ActorId) -> Option<&ColorMatrix> {
        if let Some(r) = self.renders.get(actor_id) {
            r.as_ref()
        } else {
            None
        }
    }

    pub fn get_mut_render(&mut self, actor_id: &ActorId) -> Option<&mut ColorMatrix> {
        if let Some(r) = self.renders.get_mut(actor_id) {
            r.as_mut()
        } else {
            None
        }
    }

    pub fn add_new_actor(
        &mut self,
        name: Option<&str>,
        transform: Option<Transform>,
        collider: Option<Collider>,
        physics: Option<Physics>,
        blinker: Option<Blinker>,
        render: Option<ColorMatrix>,
    ) -> ActorId {
        let mut new_actor_id = 0;
        for i in 0..=u16::MAX {
            new_actor_id = i;
            if !self.all_actors.contains(&new_actor_id) {
                break;
            }
        }

        self.all_actors.push(new_actor_id);
        self.all_actors.sort();
        self.names.insert(
            new_actor_id,
            if let Some(name_string) = name {
                Some(String::from(name_string))
            } else {
                None
            },
        );
        self.transforms.insert(new_actor_id, transform);
        self.colliders.insert(new_actor_id, collider);
        self.physics.insert(new_actor_id, physics);
        self.blinkers.insert(new_actor_id, blinker);
        self.renders.insert(new_actor_id, render);
        new_actor_id
    }

    pub fn remove_actor(&mut self, actor_id: &ActorId) {
        self.names.remove(actor_id);
        self.transforms.remove(actor_id);
        self.colliders.remove(actor_id);
        self.physics.remove(actor_id);
        self.renders.remove(actor_id);
    }

    pub fn clear_all(&mut self) {
        self.all_actors.clear();
        self.names.clear();
        self.transforms.clear();
        self.colliders.clear();
        self.physics.clear();
        self.renders.clear();
    }
}
