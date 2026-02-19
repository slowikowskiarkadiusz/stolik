use crate::{
    engine::{engine::ActorId, hash_map::HashMap},
    scenes::tetris::board::Board,
};

pub struct TetrisWorld {
    boards: HashMap<ActorId, Option<Board>>,
}

impl TetrisWorld {
    pub fn new() -> Self {
        Self { boards: HashMap::new() }
    }

    pub fn get_board(&self, actor_id: &ActorId) -> Option<&Board> {
        if let Some(r) = self.boards.get(actor_id) {
            r.as_ref()
        } else {
            None
        }
    }

    pub fn get_mut_board(&mut self, actor_id: &ActorId) -> Option<&mut Board> {
        if let Some(r) = self.boards.get_mut(actor_id) {
            r.as_mut()
        } else {
            None
        }
    }

    pub fn add_new_actor(&mut self, actor_id: ActorId, board: Option<Board>) {
        self.boards.insert(actor_id, board);
    }
}
