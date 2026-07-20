use libm::roundf;
use crate::engine::{
    actor::text::{LETTER_HEIGHT, MAX_LETTER_WIDTH, generate_word_matrix, render_text},
    color::Color,
    color_matrix::ColorMatrix,
    components::camera::Camera,
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

pub fn print_victory_text(out: &mut ColorMatrix, winner: u8, camera: &Camera, show_for_both_sides: bool) {
    let screen = camera.get_viewport().get_size();
    let screen_center_x = screen.x / 2.0;
    let scale = (screen.x / 64.0).min(1.0);

    let text = if winner == 1 { "P1 WON" } else { "P2 WON" };
    let color = if winner == 1 { P1_COLOR } else { P2_COLOR };
    let black = Color::new(0, 0, 0, 255);
    let (full_word_matrix, _) = generate_word_matrix(text, screen.x as u8, &color, false);
    let word_matrix = if scale < 1.0 { downscale_matrix(&full_word_matrix, scale) } else { full_word_matrix };

    let background_half_width = (word_matrix.width as f32 / 2.0 + 2.0).min(screen_center_x);
    let background_half_height = (4.5 * screen.y / 64.0).max(3.0);
    let background_left = (screen_center_x - background_half_width).max(0.0) as u8;
    let background_right = (screen_center_x + background_half_width).min(screen.x) as u8;

    if show_for_both_sides {
        let bottom_text_y = screen.y * 47.0 / 64.0;
        let bottom_box_top = (bottom_text_y - background_half_height).max(0.0) as u8;
        let bottom_box_bottom = (bottom_text_y + background_half_height).min(screen.y) as u8;
        for x in background_left..background_right { for y in bottom_box_top..bottom_box_bottom { out.set(x, y, black); } }
        out.write(&word_matrix, &V2::new(screen_center_x, bottom_text_y), None, None, None, None);

        let top_text_y = screen.y * 15.5 / 64.0;
        let top_box_top = (top_text_y - background_half_height).max(0.0) as u8;
        let top_box_bottom = (top_text_y + background_half_height).min(screen.y) as u8;
        for x in background_left..background_right { for y in top_box_top..top_box_bottom { out.set(x, y, black); } }
        out.write(&word_matrix, &V2::new(screen_center_x, top_text_y), Some(180.0), None, None, None);
    } else {
        let center_y = screen.y / 2.0;
        let box_top = (center_y - background_half_height).max(0.0) as u8;
        let box_bottom = (center_y + background_half_height).min(screen.y) as u8;
        for x in background_left..background_right { for y in box_top..box_bottom { out.set(x, y, black); } }
        out.write(&word_matrix, &V2::new(screen_center_x, center_y), None, None, None, None);
    }
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
