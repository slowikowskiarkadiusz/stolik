extern crate alloc;
use alloc::string::String;

use crate::engine::{
    actor::text::generate_word_matrix,
    color::Color,
    color_matrix::ColorMatrix,
    v2::V2,
};

pub fn print_victory_text(out: &mut ColorMatrix, winner: u8) {
    let text = String::from(if winner == 1 { "P1 WON" } else { "P2 WON" });
    let white = Color::new(255, 255, 255, 255);
    let black = Color::new(0, 0, 0, 255);
    let (word_matrix, _) = generate_word_matrix(&text, 21, &white, false);

    // bottom half (P1's side) — normal orientation, center at (32, 47)
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
