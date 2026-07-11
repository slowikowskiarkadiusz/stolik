use crate::engine::engine::ActorId;

pub struct TanksWorld {
    pub bullet_p1: Option<ActorId>,
    pub bullet_p2: Option<ActorId>,
}
