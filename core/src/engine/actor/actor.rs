use crate::engine::{color_matrix::ColorMatrix, engine::{ActorId, EngineView}, v2::V2};

pub struct InnerActor {
    pub name: String,
    pub id: ActorId,
    pub has_temp_id: bool,
    pub center: V2,
    pub size: V2,
    pub anchor_offset: V2,
    pub original_size: V2,
    pub rotation: f32,
    pub render_importance: u8,
    pub render_color_matrix: Option<ColorMatrix>,
}

impl InnerActor {
    pub fn _new(center: V2, size: &V2, name: String) -> Self {
        Self {
            name,
            id: 0,
            has_temp_id: true,
            center,
            original_size: size.clone(),
            size: size.clone(),
            anchor_offset: V2::zero(),
            rotation: 0.0,
            render_importance: 0,
            render_color_matrix: None,
        }
    }
}

pub trait TActor: Send + Sync {
    fn get_actor(&self) -> &InnerActor;

    fn get_mut_actor(&mut self) -> &mut InnerActor;

    fn get_id(&self) -> (u16, bool) {
        (self.get_actor().id, self.get_actor().has_temp_id)
    }

    fn set_id(&mut self, new_value: u16, is_temp_id: bool) {
        self.get_mut_actor().id = new_value;
        self.get_mut_actor().has_temp_id = is_temp_id;
    }

    fn get_center(&self) -> &V2 {
        &self.get_actor().center
    }

    fn set_center(&mut self, new_value: V2) {
        self.get_mut_actor().center = new_value;
    }

    fn move_by(&mut self, delta: V2) {
        self.set_center(self.get_center() + &delta);
    }

    fn get_size(&self) -> &V2 {
        &self.get_actor().size
    }

    fn set_size(&mut self, new_value: V2) {
        self.get_mut_actor().size = new_value;
    }

    fn get_anchor_offset(&self) -> &V2 {
        &self.get_actor().anchor_offset
    }

    fn set_anchor_offset(&mut self, new_value: V2) {
        self.get_mut_actor().anchor_offset = new_value;
    }

    fn get_original_size(&self) -> &V2 {
        &self.get_actor().original_size
    }

    fn set_original_size(&mut self, new_value: V2) {
        self.get_mut_actor().original_size = new_value;
    }

    fn get_rotation(&self) -> &f32 {
        &self.get_actor().rotation
    }

    fn set_rotation(&mut self, new_value: f32) {
        self.get_mut_actor().rotation = new_value;
    }

    fn get_render_importance(&self) -> &u8 {
        &self.get_actor().render_importance
    }

    fn set_render_importance(&mut self, new_value: u8) {
        self.get_mut_actor().render_importance = new_value;
    }

    fn get_render_color_matrix(&self) -> &Option<ColorMatrix> {
        &self.get_actor().render_color_matrix
    }

    fn update(&mut self, engine: &EngineView);
}
