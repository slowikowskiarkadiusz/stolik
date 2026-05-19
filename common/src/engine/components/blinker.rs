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

    pub fn reset(&mut self) {
        self.timer = 0.0;
        self.is_on = true;
    }
}
