use scene::Scene;

use crate::{
    engine::{
        components::{camera::Camera, collider::CollisionResult, world::World},
        engine::ActorId,
        hash_map::HashMap,
        input::input::Input,
    },
    scenes::tanks::world::TanksWorld,
};

const SIZE: u8 = 13;
// static uint border_size;
const CELL_SIZE: u8 = 4;

pub struct TanksScene {
    p1_actor_id: ActorId,
    p2_actor_id: ActorId,
    tanks_world: TanksWorld,
    // mode: TanksSceneMode,
    is_player_dead: u8,
}

impl Scene for TanksScene {
    fn new() -> Self {
        Self {
            p1_actor_id: 0,
            p2_actor_id: 0,
            tanks_world: TanksWorld {
                bullet_p1: None,
                bullet_p2: None,
            },
            is_player_dead: 0,
        }
    }

    fn init(&mut self, world: &mut World) {
        // engine::instantiate<border_actor>(v2::zero(), engine::screen_size - v2::one(), engine::screen_size, border_size);
        // board_size = (size - border_size * 2) / cell_size;

        // auto obstacle = engine::instantiate<obstacle_actor>(engine::screen_size / 2, 4, border_size, board_size + 1);
        // obstacle->set_render_importance(1);

        // auto on_tank_killed_func = [this](tank_actor *killed_tank)
        // { on_tank_killed(killed_tank); };
        // ;

        // tank1 = engine::instantiate<tank_actor>(true, obstacle);
        // tank1->rotate(180);
        // tank1->set_anchor_offset(v2(0, -2));
        // tank1->set_center(cell_to_pos(v2(0, 0)));
        // tank1->on_tank_killed = on_tank_killed_func;

        // tank2 = engine::instantiate<tank_actor>(false, obstacle);
        // tank2->set_anchor_offset(v2(0, -2));
        // tank2->set_center(cell_to_pos(v2(board_size - 1, board_size - 1)));
        // tank2->on_tank_killed = on_tank_killed_func;

        // std::srand(static_cast<unsigned>(std::time(nullptr)));
    }

    fn tick(&mut self, input: &Box<dyn Input>, world: &mut World, delta_time: f32) {
        todo!()
    }

    fn render(&mut self, camera: &Camera, world: &mut World, delta_time: f32) -> color_matrix::ColorMatrix {
        todo!()
    }

    fn on_overlaps(&mut self, overlaps: &HashMap<ActorId, Vec<ActorId>>, world: &mut World, delta_time: f32) {
        todo!()
    }

    fn on_collisions(&mut self, collisions: &HashMap<u16, Vec<(u16, CollisionResult)>>, world: &mut World, delta_time: f32) {
        todo!()
    }
}
