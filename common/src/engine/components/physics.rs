use crate::engine::{
    components::{
        collider::{Collider, CollisionResult},
        world::World,
    },
    engine::ActorId,
    v2::V2,
};

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
            mass: 1.0,
            drag: 0.0,
            angular_drag: 0.0,
            rotation: 0.0,
            inertia: 0.0,
            bounciness: 0.0,
            max_distance_from_center: 0.0,
        }
    }

    pub fn update(world: &mut World, delta_time: f32) {
        let actors: Vec<ActorId> = world.actors().iter().copied().collect();
        Physics::apply_forces(&actors, world, delta_time);
        for res in Collider::detect_collisions(world) {
            for col in res.1 {
                Physics::apply_impuls(&res.0, &col, world, delta_time);
            }
        }
    }

    fn apply_forces(actors: &Vec<ActorId>, world: &mut World, delta_time: f32) {
        for actor_id in actors {
            if world.get_physics(&actor_id).is_none() || world.get_physics(&actor_id).unwrap().is_fixed {
                continue;
            }
            let (new_velocity, new_center) = {
                let transform = world.get_transform(&actor_id).unwrap();
                let physics = world.get_physics(&actor_id).unwrap();
                let new_vel = &physics.velocity + &(physics.force / physics.mass * delta_time);
                let new_vel = &new_vel * (1.0 - physics.drag * delta_time);
                let new_center = &transform.center + &(new_vel * delta_time);
                (new_vel, new_center)
            };
            let physics = world.get_mut_physics(&actor_id).unwrap();
            physics.velocity = new_velocity;
            physics.force = V2::zero();
            world.get_mut_transform(&actor_id).unwrap().center = new_center;
        }
    }

    fn apply_impuls(actor: &ActorId, collision: &(ActorId, CollisionResult), world: &mut World, delta_time: f32) {
        if world.get_physics(actor).unwrap().is_fixed {
            return;
        }

        let (a_mass, a_velocity) = {
            let body = world.get_physics(actor).unwrap();
            (body.mass, body.velocity)
        };
        let (b_mass, b_velocity) = {
            let body = world.get_physics(&collision.0).unwrap();
            (body.mass, body.velocity)
        };
        let normal = collision.1.normal;
        let penetration = collision.1.penetration;
        let v_rel = &b_velocity - &a_velocity;
        let vel_along_normal = v_rel.dot(&normal);

        if vel_along_normal > 0.0 {
            return;
        }

        let e = 0.5; // restitution
        let j = -(1.0 + e) * vel_along_normal / (1.0 / a_mass + 1.0 / b_mass);

        world.get_mut_physics(actor).unwrap().velocity -= normal * j / a_mass;
    }

    pub fn add_force(&mut self, force: V2) {
        self.force += force;
    }

    pub fn with_is_fixed(&mut self, is_fixed: bool) -> &mut Self {
        self.is_fixed = is_fixed;
        self
    }

    pub fn with_mass(&mut self, mass: f32) -> &mut Self {
        self.mass = mass;
        self
    }

    pub fn with_drag(&mut self, drag: f32) -> &mut Self {
        self.drag = drag;
        self
    }

    pub fn with_angular_drag(&mut self, angular_drag: f32) -> &mut Self {
        self.angular_drag = angular_drag;
        self
    }

    pub fn with_rotation(&mut self, rotation: f32) -> &mut Self {
        self.rotation = rotation;
        self
    }

    pub fn with_inertia(&mut self, inertia: f32) -> &mut Self {
        self.inertia = inertia;
        self
    }

    pub fn with_bounciness(&mut self, bounciness: f32) -> &mut Self {
        self.bounciness = bounciness;
        self
    }

    pub fn with_max_distance_from_center(&mut self, max_distance_from_center: f32) -> &mut Self {
        self.max_distance_from_center = max_distance_from_center;
        self
    }
}
