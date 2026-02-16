use crate::engine::components::world::World;

pub struct Blinker {
    pub timer: f32,
    pub delay_ms: u32,
    pub is_on: bool,
}

impl Blinker {
    pub fn new(delay_ms: u32) -> Self {
        Self {
            is_on: true,
            delay_ms,
            timer: 0.0,
        }
    }

    pub fn tick(world: &mut World, delta_time: f32) {
        // TODO czy ja musze klonowac te liste?
        for actor_id in &world.all_actors.clone() {
            if let Some(blinker) = world.get_mut_blinker(&actor_id) {
                blinker.timer += delta_time;
                if blinker.timer >= (blinker.delay_ms as f32 / 1000.0) {
                    blinker.is_on = !blinker.is_on;
                    blinker.timer = 0.0;
                }
            }
        }
    }

    pub fn reset(&mut self) {
        self.timer = 0.0;
        self.is_on = true;
    }
}
