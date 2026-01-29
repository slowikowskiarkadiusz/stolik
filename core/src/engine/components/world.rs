use std::collections::HashMap;

use crate::engine::{
    color_matrix::ColorMatrix,
    components::{collider::Collider, physics::Physics, transform::Transform},
    engine::ActorId,
};

pub struct World {
    all_actors: Vec<ActorId>,
    names: HashMap<ActorId, Option<String>>,
    transforms: HashMap<ActorId, Option<Transform>>,
    colliders: HashMap<ActorId, Option<Collider>>,
    physics: HashMap<ActorId, Option<Physics>>,
    renders: HashMap<ActorId, Option<ColorMatrix>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            all_actors: Vec::new(),
            names: HashMap::new(),
            transforms: HashMap::new(),
            colliders: HashMap::new(),
            physics: HashMap::new(),
            renders: HashMap::new(),
        }
    }

    pub fn get_all_actors(&self) -> &[ActorId] {
        &self.all_actors
    }

    pub fn get_name(&self, actor_id: &ActorId) -> Option<&String> {
        self.names.get(actor_id).unwrap().as_ref()
    }

    pub fn get_mut_name(&mut self, actor_id: &ActorId) -> Option<&mut String> {
        self.names.get_mut(actor_id).unwrap().as_mut()
    }

    pub fn get_transform(&self, actor_id: &ActorId) -> Option<&Transform> {
        self.transforms.get(actor_id).unwrap().as_ref()
    }

    pub fn get_mut_transform(&mut self, actor_id: &ActorId) -> Option<&mut Transform> {
        self.transforms.get_mut(actor_id).unwrap().as_mut()
    }

    pub fn get_collider(&self, actor_id: &ActorId) -> Option<&Collider> {
        self.colliders.get(actor_id).unwrap().as_ref()
    }

    pub fn get_mut_collider(&mut self, actor_id: &ActorId) -> Option<&mut Collider> {
        self.colliders.get_mut(actor_id).unwrap().as_mut()
    }

    pub fn get_physics(&self, actor_id: &ActorId) -> Option<&Physics> {
        self.physics.get(actor_id).unwrap().as_ref()
    }

    pub fn get_mut_physics(&mut self, actor_id: &ActorId) -> Option<&mut Physics> {
        self.physics.get_mut(actor_id).unwrap().as_mut()
    }

    pub fn get_render(&self, actor_id: &ActorId) -> Option<&ColorMatrix> {
        self.renders.get(actor_id).unwrap().as_ref()
    }

    pub fn get_mut_render(&mut self, actor_id: &ActorId) -> Option<&mut ColorMatrix> {
        self.renders.get_mut(actor_id).unwrap().as_mut()
    }

    pub fn add_new_actor(
        &mut self,
        name: Option<String>,
        transform: Option<Transform>,
        collider: Option<Collider>,
        physics: Option<Physics>,
        render: Option<ColorMatrix>,
    ) -> ActorId {
        let new_actor_id = self.get_free_id();

        self.all_actors.push(new_actor_id);
        self.names.insert(new_actor_id, name);
        self.transforms.insert(new_actor_id, transform);
        self.colliders.insert(new_actor_id, collider);
        self.physics.insert(new_actor_id, physics);
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

    fn get_free_id(&self) -> ActorId {
        let map = &self.transforms;
        let iter = map.iter();

        let iter_max = map.iter().map(|x| x.0).max();
        let mut actor_id: ActorId = 0;
        if let Some(max_id) = iter_max {
            actor_id = max_id.clone();

            if actor_id == u16::MAX {
                let mut found = false;
                for (k, _v) in iter {
                    if k - actor_id > 1 {
                        actor_id = k.clone();
                        found = true;
                        break;
                    }

                    actor_id = k.clone();
                }

                if !found {
                    actor_id += 1;
                }
            } else {
                actor_id += 1;
            }
        }

        actor_id
    }
}
