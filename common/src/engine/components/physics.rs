use crate::engine::{components::world::World, engine::ActorId, v2::V2};

#[allow(dead_code)]
pub struct Physics {
    force: V2,
    velocity: V2,
    angular_velocity: f32,
    is_fixed: bool,
    mass: f32,
    drag: f32,
    angular_drag: f32,
    rotation: f32,
    inertia: f32,
    bounciness: f32,
    max_distance_from_center: f32,
}

impl Physics {
    pub fn new() -> Self {
        Self {
            force: V2::zero(),
            velocity: V2::zero(),
            angular_velocity: 0.0,
            is_fixed: false,
            mass: 0.0,
            drag: 0.0,
            angular_drag: 0.0,
            rotation: 0.0,
            inertia: 0.0,
            bounciness: 0.0,
            max_distance_from_center: 0.0,
        }
    }

    pub fn update(world: &mut World, delta_time: f32) {
        let actors = world.actors();
        for (ai, first_actor) in actors.iter().enumerate() {
            for second_actor in &actors[ai + 1..] {}
        }
    }

    fn apply_forces(actors: &[ActorId], world: &mut World, delta_time: f32) {
        for actor_id in actors {
            if let Some(physics) = world.get_mut_physics(actor_id)
                && let Some(transform) = world.get_mut_transform(actor_id)
            {
                physics.velocity += physics.force / physics.mass * delta_time;
                transform.center += physics.velocity * delta_time;
                physics.force = V2::zero();
            }
        }
    }

    pub fn with_is_fixed(&mut self, is_fixed: bool) -> &Self {
        self.is_fixed = is_fixed;
        self
    }

    pub fn with_mass(&mut self, mass: f32) -> &Self {
        self.mass = mass;
        self
    }

    pub fn with_drag(&mut self, drag: f32) -> &Self {
        self.drag = drag;
        self
    }

    pub fn with_angular_drag(&mut self, angular_drag: f32) -> &Self {
        self.angular_drag = angular_drag;
        self
    }

    pub fn with_rotation(&mut self, rotation: f32) -> &Self {
        self.rotation = rotation;
        self
    }

    pub fn with_inertia(&mut self, inertia: f32) -> &Self {
        self.inertia = inertia;
        self
    }

    pub fn with_bounciness(&mut self, bounciness: f32) -> &Self {
        self.bounciness = bounciness;
        self
    }

    pub fn with_max_distance_from_center(&mut self, max_distance_from_center: f32) -> &Self {
        self.max_distance_from_center = max_distance_from_center;
        self
    }
}
