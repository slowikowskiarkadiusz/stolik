use crate::engine::{
    actor::text::{LETTER_HEIGHT, MAX_LETTER_WIDTH, create_text_actor_at_center, generate_word_matrix, render_text},
    color::Color,
    color_matrix::ColorMatrix,
    components::camera::Camera,
    engine::SCREEN_SIZEF32,
    v2::V2,
};
use libm::roundf;

pub const fn cmyk_to_rgb(c: u8, m: u8, y: u8, k: u8) -> (u8, u8, u8) {
    let r = (255.0 * (1.0 - c as f32 / 100.0) * (1.0 - k as f32 / 100.0)) as u8;
    let g = (255.0 * (1.0 - m as f32 / 100.0) * (1.0 - k as f32 / 100.0)) as u8;
    let b = (255.0 * (1.0 - y as f32 / 100.0) * (1.0 - k as f32 / 100.0)) as u8;
    (r, g, b)
}

// Orange Yellow: C0 M33 Y100 K0
pub const P1_COLOR: Color = {
    let (r, g, b) = cmyk_to_rgb(0, 33, 100, 0);
    Color::new(r, g, b, 255)
};

// Blue: C95 M54 Y0 K0
pub const P2_COLOR: Color = {
    let (r, g, b) = cmyk_to_rgb(95, 54, 0, 0);
    Color::new(r, g, b, 255)
};

fn digit_str(n: u8, buf: &mut [u8; 1]) -> &str {
    buf[0] = b'0' + (n % 10);
    core::str::from_utf8(buf.as_slice()).unwrap_or("?")
}

fn downscale_matrix(source: &ColorMatrix, scale: f32) -> ColorMatrix {
    let new_width = ((source.width as f32 * scale) as u8).max(1);
    let new_height = ((source.height as f32 * scale) as u8).max(1);
    let mut result = ColorMatrix::new(new_width, new_height, Color::none());
    for y in 0..new_height {
        for x in 0..new_width {
            let src_x = ((x as f32 / scale) as u8).min(source.width - 1);
            let src_y = ((y as f32 / scale) as u8).min(source.height - 1);
            result.set(x, y, *source.get(src_x, src_y));
        }
    }
    result
}

pub fn print_victory_text(out: &mut ColorMatrix, winner: u8, show_for_both_sides: bool) {
    fn internal(text: &str, center: V2, rotation: Option<f32>, color: Color, result: &mut ColorMatrix) {
        let container_size = V2::new(result.width as f32, 6.0);
        let generated = generate_word_matrix(&text, None, &color, false).0;

        result.write(&generated, &center, rotation, None, None, None);
    }

    let text = if winner == 1 { "P1 WON" } else { "P2 WON" };
    let color = if winner == 1 { P1_COLOR } else { P2_COLOR };
    let black = Color::new(0, 0, 0, 150);

    out.write_at_origin(&ColorMatrix::new(out.width, out.height, black), &V2::zero(), Some(true));

    internal(text, out.get_size() / 2.0 + V2::down() * 6.0, None, color, out);
    internal(text, out.get_size() / 2.0 + V2::up() * 6.0, Some(180.0), color, out);
}

pub fn print_score(score_p1: u8, score_p2: u8, result: &mut ColorMatrix) {
    let mut buf = [0; 1];

    render_text(
        digit_str(score_p1, &mut buf),
        V2::new(
            (SCREEN_SIZEF32 / 2.0) - (MAX_LETTER_WIDTH as f32) / 2.0,
            SCREEN_SIZEF32 - 2.0 - (LETTER_HEIGHT as f32) / 2.0 - 1.0,
        ),
        V2::new(MAX_LETTER_WIDTH as f32, LETTER_HEIGHT as f32),
        None,
        None,
        P1_COLOR,
        None,
        result,
    );

    render_text(
        digit_str(score_p2, &mut buf),
        V2::new(
            (SCREEN_SIZEF32 / 2.0) - (MAX_LETTER_WIDTH as f32) / 2.0,
            1.0 - (LETTER_HEIGHT as f32) / 2.0 + 1.0,
        ),
        V2::new(MAX_LETTER_WIDTH as f32, LETTER_HEIGHT as f32),
        None,
        Some(180.0),
        P2_COLOR,
        None,
        result,
    );
}

pub fn lerp_f32(from: f32, to: f32, step: f32) -> f32 {
    from * (1.0 - step) + to * step
}

pub fn lerp_u8(from: u8, to: u8, step: f32) -> u8 {
    roundf(from as f32 * (1.0 - step) + to as f32 * step) as u8
}
