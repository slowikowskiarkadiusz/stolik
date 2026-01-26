use crate::engine::{
    actor::actor::{InnerActor, TActor}, color::Color, color_matrix::ColorMatrix, engine::{Engine, register_actor}, v2::V2
};

pub struct RectangleActor {
    actor: InnerActor,
}

impl RectangleActor {
    pub fn spawn(center: V2, size: &V2, color: Color) -> &Self {
        let mut actor = InnerActor::_new(center, size);
        let color_matrix = ColorMatrix::new(size.x as u8, size.y as u8, color);
        actor.render_color_matrix = Some(color_matrix);
        let obj = Self { actor };
        register_actor(Box::new(obj));
        &obj
    }
}

impl TActor for RectangleActor {
    fn get_actor(&self) -> &InnerActor {
        &self.actor
    }

    fn get_mut_actor(&mut self) -> &mut InnerActor {
        &mut self.actor
    }
}
