use crate::{engine::{engine::ActorId, hash_map::HashMap}, scenes::tetris::board::Board};

pub struct World {
    boards: HashMap<ActorId, Option<Board>>,
}
