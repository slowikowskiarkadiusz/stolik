use crate::engine::{
    actor::actor::{InnerActor, TActor},
    color::Color,
    color_matrix::ColorMatrix,
    engine::{ActorId, EngineView},
    v2::V2,
};

pub struct RectangleActor {
    actor: InnerActor,
}

impl RectangleActor {
    pub fn new(
        center: V2,
        size: &V2,
        color: Color,
        name: Option<String>,
    ) -> Self {
        let mut actor = InnerActor::_new(
            center,
            size,
            if let Some(name_unwrapped) = name {
                name_unwrapped
            } else {
                String::from("rectangle")
            },
        );
        let color_matrix = ColorMatrix::new(size.x as u8, size.y as u8, color);
        actor.render_color_matrix = Some(color_matrix);
        Self { actor }
    }
}

impl TActor for RectangleActor {
    fn get_actor(&self) -> &InnerActor {
        &self.actor
    }

    fn get_mut_actor(&mut self) -> &mut InnerActor {
        &mut self.actor
    }

    fn update(&mut self, _engine: &EngineView) {}
}
