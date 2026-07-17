use crate::engine::{
    actor::text::{LETTER_HEIGHT, MAX_LETTER_WIDTH, generate_word_matrix, render_text},
    color::Color,
    color_matrix::ColorMatrix,
    engine::SCREEN_SIZEF32,
    v2::V2,
};

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

pub fn print_victory_text(out: &mut ColorMatrix, winner: u8) {
    let text = if winner == 1 { "P1 WON" } else { "P2 WON" };
    let white = if winner == 1 { P1_COLOR } else { P2_COLOR };
    let black = Color::new(0, 0, 0, 255);
    let (word_matrix, _) = generate_word_matrix(text, 21, &white, false);

    for x in 19u8..45u8 {
        for y in 43u8..52u8 {
            out.set(x, y, black);
        }
    }
    out.write(&word_matrix, &V2::new(32.0, 47.0), None, None, None, None);

    for x in 19u8..45u8 {
        for y in 12u8..21u8 {
            out.set(x, y, black);
        }
    }
    out.write(&word_matrix, &V2::new(31.5, 15.5), Some(180.0), None, None, None);
}

pub fn print_score(score_p1: u8, score_p2: u8, result: &mut ColorMatrix) {
    let mut buf = [0u8; 1];

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
            1.0 - (LETTER_HEIGHT as f32) / 2.0,
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
    (from as f32 * (1.0 - step) + to as f32 * step).round() as u8
}
