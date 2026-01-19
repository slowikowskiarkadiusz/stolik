#[derive(PartialEq, Clone)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn red(a: Option<u8>) -> Color {
        Color::new(1, 0, 0, a.unwrap_or(255))
    }

    pub fn green(a: Option<u8>) -> Color {
        Color::new(0, 1, 0, a.unwrap_or(255))
    }

    pub fn blue(a: Option<u8>) -> Color {
        Color::new(0, 0, 1, a.unwrap_or(255))
    }

    pub fn yellow(a: Option<u8>) -> Color {
        Color::new(1, 1, 0, a.unwrap_or(255))
    }

    pub fn cyan(a: Option<u8>) -> Color {
        Color::new(0, 1, 1, a.unwrap_or(255))
    }

    pub fn magenta(a: Option<u8>) -> Color {
        Color::new(1, 0, 1, a.unwrap_or(255))
    }

    pub fn white(a: Option<u8>) -> Color {
        Color::new(1, 1, 1, a.unwrap_or(255))
    }

    pub fn black(a: Option<u8>) -> Color {
        Color::new(0, 0, 0, a.unwrap_or(255))
    }

    pub fn none() -> Color {
        Color::black(Some(0))
    }

    pub fn is_none(&self) -> bool {
        self.r == 0 && self.g == 0 && self.b == 0 && self.a == 0
    }

    pub fn blend_colors(self, input_colors: &[Color]) -> Color {
        let mut colors: Vec<Color> = vec![];
        for a in input_colors {
            if a.is_none() {
                colors.push(a.clone().clone());
            }
        }

        if colors.is_empty() {
            return Color::none();
        }

        if colors.len() == 1 {
            return colors.first().unwrap().clone();
        }

        Color::additive_blending(&colors)
    }

    fn additive_blending(colors: &[Color]) -> Color {
        let mut r: u16 = 0;
        let mut g: u16 = 0;
        let mut b: u16 = 0;
        let mut a: u16 = 0;
        for c in colors {
            r += ((c.r as f32) * (c.a as f32) / 255.0).round() as u16;
            g += ((c.g as f32) * (c.a as f32) / 255.0).round() as u16;
            b += ((c.b as f32) * (c.a as f32) / 255.0).round() as u16;
            a += c.a as u16;
        }

        Color::new(
            r.clamp(0, 255) as u8,
            g.clamp(0, 255) as u8,
            b.clamp(0, 255) as u8,
            a.clamp(0, 255) as u8,
        )
    }
}

impl Default for Color {
    fn default() -> Self {
        Color::none()
    }
}
