use core::f32::consts::PI;
use spin::Mutex;

extern crate alloc;
use crate::engine::{
    components::{
        collider::{Collider, CollisionResult},
        world::World,
    },
    engine::ActorId,
    hash_map::HashMap,
    v2::V2,
};
use alloc::vec::Vec;

static GRAVITY: Mutex<V2> = Mutex::new(V2::new(0.0, 0.981));
static ITERATIONS: u8 = 10;

#[allow(dead_code)]
pub struct Physics {
    force: V2,
    velocity: V2,
    angular_velocity: f32,
    can_rotate: bool,
    can_move: bool,
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
            can_rotate: false,
            can_move: false,
            mass: 1.0,
            drag: 0.0,
            angular_drag: 0.0,
            rotation: 0.0,
            inertia: 0.0,
            bounciness: 0.0,
            max_distance_from_center: 0.0,
        }
    }

    pub fn set_gravity(value: V2) {
        let mut g = GRAVITY.lock();
        g.x = value.x;
        g.y = value.y;
    }

    pub fn update(world: &mut World, delta_time: f32, collisions: &mut HashMap<u16, Vec<(u16, CollisionResult)>>) {
        let actors: Vec<ActorId> = world.get_actors().iter().copied().collect();
        Physics::apply_forces(&actors, world, delta_time);
        for _ in 0..ITERATIONS {
            Collider::detect_collisions(world, collisions);
            for res in collisions.iter() {
                for col in res.1 {
                    Physics::apply_impuls(&res.0, &col, world, delta_time);
                }
            }
        }
    }

    fn apply_forces(actors: &Vec<ActorId>, world: &mut World, delta_time: f32) {
        for actor_id in actors {
            if world.get_physics(&actor_id).is_none() || !world.get_physics(&actor_id).unwrap().can_move {
                continue;
            }
            let (new_velocity, new_center, new_ang_vel, new_rotation, computed_inertia) = {
                let transform = world.get_transform(&actor_id).unwrap();
                let physics = world.get_physics(&actor_id).unwrap();

                let new_vel = &physics.velocity + &(physics.force / physics.mass * delta_time);
                let new_vel = &new_vel * (1.0 - physics.drag * delta_time);
                let g = GRAVITY.lock();
                let new_vel = &new_vel + &g;
                let new_center = &transform.center + &(new_vel * delta_time);

                let new_ang_vel = if physics.can_rotate {
                    physics.angular_velocity * (1.0 - physics.angular_drag * delta_time)
                } else {
                    0.0
                };
                let new_rotation = if physics.can_rotate {
                    physics.rotation + new_ang_vel * delta_time
                } else {
                    0.0
                };

                let computed_inertia = if physics.inertia > 0.0 {
                    physics.inertia
                } else {
                    let w = transform.size.x;
                    let h = transform.size.y;
                    physics.mass * (w * w + h * h) / 12.0
                };

                (new_vel, new_center, new_ang_vel, new_rotation, computed_inertia)
            };
            let physics = world.get_mut_physics(&actor_id).unwrap();
            physics.velocity = new_velocity;
            physics.force = V2::zero();
            physics.angular_velocity = new_ang_vel;
            physics.rotation = new_rotation;
            physics.inertia = computed_inertia;

            let transform = world.get_mut_transform(&actor_id).unwrap();
            transform.center = new_center;
            transform.rotation = new_rotation;
        }
    }

    fn apply_impuls(actor: &ActorId, collision: &(ActorId, CollisionResult), world: &mut World, _delta_time: f32) {
        if world.get_physics(actor).is_none() || (!world.get_physics(actor).unwrap().can_move) {
            return;
        }

        let normal = collision.1.normal;
        let penetration = collision.1.penetration;
        let contact = collision.1.contact_point;

        let (a_mass, a_velocity, a_inertia, a_ang_vel, a_center) = {
            let body = world.get_physics(actor).unwrap();
            let t = world.get_transform(actor).unwrap();
            (body.mass, body.velocity, body.inertia, body.angular_velocity, t.center)
        };
        let (b_mass, b_velocity, b_inertia, b_ang_vel, b_center, b_is_fixed) = {
            let t = world.get_transform(&collision.0).unwrap();
            if let Some(body) = world.get_physics(&collision.0) {
                (
                    body.mass,
                    body.velocity,
                    body.inertia,
                    body.angular_velocity,
                    t.center,
                    body.can_move,
                )
            } else {
                (1.0, V2::zero(), 0.0, 0.0, t.center, false) // traktuj jak !can_move
            }
        };

        let ra = V2::new(contact.x - a_center.x, contact.y - a_center.y);
        let rb = V2::new(contact.x - b_center.x, contact.y - b_center.y);

        let omega_a = a_ang_vel * (PI / 180.0);
        let omega_b = b_ang_vel * (PI / 180.0);
        let va_c = V2::new(a_velocity.x - omega_a * ra.y, a_velocity.y + omega_a * ra.x);
        let vb_c = V2::new(b_velocity.x - omega_b * rb.y, b_velocity.y + omega_b * rb.x);

        let v_rel = &vb_c - &va_c;
        let vel_along_normal = v_rel.dot(&normal);

        if vel_along_normal > 0.0 {
            return;
        }

        let e = 0.5;

        let a_can_rotate = world.get_physics(actor).unwrap().can_rotate;
        let inv_ia = if a_can_rotate && a_inertia > 0.0 { 1.0 / a_inertia } else { 0.0 };
        let inv_ib = if !b_is_fixed && b_inertia > 0.0 { 1.0 / b_inertia } else { 0.0 };
        let inv_mb = if b_is_fixed { 0.0 } else { 1.0 / b_mass };

        let ra_x_n = ra.cross(&normal);
        let rb_x_n = rb.cross(&normal);
        let j = -(1.0 + e) * vel_along_normal / (1.0 / a_mass + inv_mb + ra_x_n * ra_x_n * inv_ia + rb_x_n * rb_x_n * inv_ib);

        {
            let phys_a = world.get_mut_physics(actor).unwrap();
            phys_a.velocity -= normal * (j / a_mass);
            if a_can_rotate {
                let delta_omega_rad = -(j * ra_x_n) * inv_ia;
                phys_a.angular_velocity += delta_omega_rad * (180.0 / PI);
            }
        }

        const SLOP: f32 = 0.1;
        const PERCENT: f32 = 0.8;
        let correction = (penetration - SLOP).max(0.0) * PERCENT;
        if let Some(t) = world.get_mut_transform(actor) {
            t.center -= normal * correction;
        }
    }

    pub fn add_force(&mut self, force: V2) {
        self.force += force;
    }

    /// instantly changes the speed, ignoring delta_time. good for jumping
    pub fn add_impulse(&mut self, impulse: V2) {
        self.velocity += impulse;
    }

    pub fn get_velocity(&self) -> &V2 {
        &self.velocity
    }

    pub fn set_velocity(&mut self, velocity: V2) {
        self.velocity = velocity;
    }

    pub fn with_can_rotate(&mut self, can_rotate: bool) -> &mut Self {
        self.can_rotate = can_rotate;
        self
    }

    pub fn with_can_move(&mut self, can_move: bool) -> &mut Self {
        self.can_move = can_move;
        self
    }

    pub fn is_movable(&self) -> bool {
        self.can_move
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
