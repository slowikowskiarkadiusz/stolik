use crate::engine::input::key::KEYS_LENGTH;

pub trait AiInput {
    fn on_ai_data(&mut self, inputs: &[f64], outputs_to_keys: fn(&[f64]) -> [bool; KEYS_LENGTH as usize]);
}
