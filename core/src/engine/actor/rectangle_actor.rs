use crate::engine::{
    actor::actor::{InnerActor, TActor},
    color::Color,
    color_matrix::ColorMatrix,
    engine::{Engine},
    v2::V2,
};

pub struct RectangleActor {
    actor: InnerActor,
}

impl RectangleActor {
    pub fn spawn(engine: &mut Engine, center: V2, size: &V2, color: Color) -> u16 {
        let mut actor = InnerActor::_new(center, size);
        let color_matrix = ColorMatrix::new(size.x as u8, size.y as u8, color);
        actor.render_color_matrix = Some(color_matrix);
        engine.register_actor(Box::new(Self { actor }))
    }
}

impl TActor for RectangleActor {
    fn get_actor(&self) -> &InnerActor {
        &self.actor
    }

    fn get_mut_actor(&mut self) -> &mut InnerActor {
        &mut self.actor
    }

    fn update(&mut self, _delta_time: f32) {}
}
