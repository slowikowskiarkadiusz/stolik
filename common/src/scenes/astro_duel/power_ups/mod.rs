pub mod power_up;
pub mod shield;
pub mod reflector;
pub mod mine;
pub mod ray_gun;

pub use power_up::{PowerUp, PowerUpKind, power_up_kind};
pub use mine::PlacedMine;
pub use ray_gun::RayGunBlast;

/// Passive power-ups only — active ones (Mine, RayGun) live in Ship::stored_power_up.
#[derive(Debug)]
pub enum ShipPowerUp {
    Shield,
    Reflector { timer: f32 },
}
