extern crate alloc;
use alloc::string::String;

use crate::engine::{
    color::Color,
    color_matrix::ColorMatrix,
    components::{camera::Camera, transform::Transform, world::World},
    engine::ActorId,
    matrix::Matrix,
    v2::V2,
};

pub const MAX_LETTER_WIDTH: u8 = 3;
pub const LETTER_HEIGHT: u8 = 5;

pub struct TextActorOptions {
    pub animate_if_outbound: bool,
    pub reverse: bool,
    pub debug_highlight: bool,
}

pub fn make_text_actor_options(
    animate_if_outbound: Option<bool>,
    reverse: Option<bool>,
    debug_highlight: Option<bool>,
) -> TextActorOptions {
    TextActorOptions {
        animate_if_outbound: animate_if_outbound.unwrap_or_else(|| true),
        reverse: reverse.unwrap_or_else(|| false),
        debug_highlight: debug_highlight.unwrap_or_else(|| false),
    }
}

pub fn render_text(
    world: &mut World,
    text: String,
    top_left: V2,
    container_size: V2,
    options: Option<TextActorOptions>,
    rotation: Option<f32>,
    color: Color,
    camera: &Camera,
    result: &mut ColorMatrix,
) {
    let mut reverse = false;
    if let Some(opt) = options {
        reverse = opt.reverse;
    }
    let bottom_right = &top_left + &container_size;
    let center = (&top_left + &bottom_right) / 2.0;

    let generated = generate_word_matrix(&text, container_size.x as u8, &color, reverse).0;

    result.write(&generated, &center, rotation, None, None, Some(camera));

    // world.add_new_actor(Some(Transform::new(center, container_size.clone())), None, None, None)
}

pub fn create_text_actor_at_center(
    world: &mut World,
    text: String,
    center: V2,
    container_size: V2,
    options: Option<TextActorOptions>,
    rotation: Option<f32>,
    color: Color,
    camera: &Camera,
    result: &mut ColorMatrix,
) -> ActorId {
    let mut reverse = false;
    if let Some(opt) = options {
        reverse = opt.reverse;
    }
    let generated = generate_word_matrix(&text, container_size.x as u8, &color, reverse).0;

    result.write(&generated, &center, rotation, None, None, Some(camera));

    world.add_new_actor(Some(Transform::new(center, container_size)), None, None, None)
}

pub fn generate_word_matrix(text: &String, max_width: u8, color: &Color, reverse: bool) -> (ColorMatrix, Option<ColorMatrix>) {
    let mut word_render_length = 0;
    for letter in text.chars() {
        word_render_length += get_letter_width(&letter);
        word_render_length += 1;
    }
    word_render_length -= 1;
    let mut full_word_matrix = ColorMatrix::new(u8::max(max_width, word_render_length), LETTER_HEIGHT, Color::none());
    let color_none = Color::none();

    let mut current_position: u8 = 0;
    for letter in text.chars() {
        generate_letter(
            &mut full_word_matrix,
            &mut current_position,
            letter,
            if !reverse { color } else { &color_none },
        );

        current_position += 1;
    }

    if word_render_length <= max_width {
        (full_word_matrix, None)
    } else {
        (
            full_word_matrix.snippet(&V2::zero(), &V2::new(max_width as f32, LETTER_HEIGHT as f32)),
            Some(full_word_matrix),
        )
    }
}

fn generate_letter(into: &mut ColorMatrix, at_x: &mut u8, letter: char, color: &Color) {
    let mut foreground = Matrix::<bool>::new(get_letter_width(&letter), LETTER_HEIGHT, false);

    match letter {
        'a' | 'A' => {
            foreground.set(1, 0, true);
            foreground.set(0, 1, true);
            foreground.set(2, 1, true);
            foreground.set(0, 2, true);
            foreground.set(1, 2, true);
            foreground.set(2, 2, true);
            foreground.set(0, 3, true);
            foreground.set(2, 3, true);
            foreground.set(0, 4, true);
            foreground.set(2, 4, true);
        }
        'b' | 'B' => {
            foreground.set(0, 0, true);
            foreground.set(1, 0, true);
            foreground.set(0, 1, true);
            foreground.set(2, 1, true);
            foreground.set(0, 2, true);
            foreground.set(1, 2, true);
            foreground.set(0, 3, true);
            foreground.set(2, 3, true);
            foreground.set(0, 4, true);
            foreground.set(1, 4, true);
        }
        'c' | 'C' => {
            foreground.set(1, 0, true);
            foreground.set(2, 0, true);
            foreground.set(0, 1, true);
            foreground.set(0, 2, true);
            foreground.set(0, 3, true);
            foreground.set(1, 4, true);
            foreground.set(2, 4, true);
        }
        'd' | 'D' => {
            foreground.set(0, 0, true);
            foreground.set(1, 0, true);
            foreground.set(0, 1, true);
            foreground.set(2, 1, true);
            foreground.set(0, 2, true);
            foreground.set(2, 2, true);
            foreground.set(0, 3, true);
            foreground.set(2, 3, true);
            foreground.set(0, 4, true);
            foreground.set(1, 4, true);
        }
        'e' | 'E' => {
            foreground.set(0, 0, true);
            foreground.set(1, 0, true);
            foreground.set(2, 0, true);
            foreground.set(0, 1, true);
            foreground.set(0, 2, true);
            foreground.set(1, 2, true);
            foreground.set(0, 3, true);
            foreground.set(0, 4, true);
            foreground.set(1, 4, true);
            foreground.set(2, 4, true);
        }
        'f' | 'F' => {
            foreground.set(0, 0, true);
            foreground.set(1, 0, true);
            foreground.set(2, 0, true);
            foreground.set(0, 1, true);
            foreground.set(0, 2, true);
            foreground.set(1, 2, true);
            foreground.set(0, 3, true);
            foreground.set(0, 4, true);
        }
        'g' | 'G' => {
            foreground.set(1, 0, true);
            foreground.set(2, 0, true);
            foreground.set(0, 1, true);
            foreground.set(0, 2, true);
            foreground.set(2, 2, true);
            foreground.set(0, 3, true);
            foreground.set(2, 3, true);
            foreground.set(1, 4, true);
            foreground.set(2, 4, true);
        }
        'h' | 'H' => {
            foreground.set(0, 0, true);
            foreground.set(2, 0, true);
            foreground.set(0, 1, true);
            foreground.set(2, 1, true);
            foreground.set(0, 2, true);
            foreground.set(1, 2, true);
            foreground.set(2, 2, true);
            foreground.set(0, 3, true);
            foreground.set(2, 3, true);
            foreground.set(0, 4, true);
            foreground.set(2, 4, true);
        }
        'i' | 'I' => {
            foreground.set(0, 0, true);
            foreground.set(1, 0, true);
            foreground.set(2, 0, true);
            foreground.set(1, 1, true);
            foreground.set(1, 2, true);
            foreground.set(1, 3, true);
            foreground.set(0, 4, true);
            foreground.set(1, 4, true);
            foreground.set(2, 4, true);
        }
        'j' | 'J' => {
            foreground.set(0, 0, true);
            foreground.set(1, 0, true);
            foreground.set(2, 0, true);
            foreground.set(2, 1, true);
            foreground.set(2, 2, true);
            foreground.set(0, 3, true);
            foreground.set(2, 3, true);
            foreground.set(1, 4, true);
        }
        'k' | 'K' => {
            foreground.set(0, 0, true);
            foreground.set(2, 0, true);
            foreground.set(0, 1, true);
            foreground.set(2, 1, true);
            foreground.set(0, 2, true);
            foreground.set(1, 2, true);
            foreground.set(0, 3, true);
            foreground.set(2, 3, true);
            foreground.set(0, 4, true);
            foreground.set(2, 4, true);
        }
        'l' | 'L' => {
            foreground.set(0, 0, true);
            foreground.set(0, 1, true);
            foreground.set(0, 2, true);
            foreground.set(0, 3, true);
            foreground.set(0, 4, true);
            foreground.set(1, 4, true);
            foreground.set(2, 4, true);
        }
        'm' | 'M' => {
            foreground.set(0, 0, true);
            foreground.set(2, 0, true);
            foreground.set(0, 1, true);
            foreground.set(1, 1, true);
            foreground.set(2, 1, true);
            foreground.set(0, 2, true);
            foreground.set(2, 2, true);
            foreground.set(0, 3, true);
            foreground.set(2, 3, true);
            foreground.set(0, 4, true);
            foreground.set(2, 4, true);
        }
        'n' | 'N' => {
            // set!(0, 0);
            foreground.set(0, 1, true);
            foreground.set(0, 2, true);
            foreground.set(0, 3, true);
            foreground.set(0, 4, true);
            foreground.set(1, 0, true);
            foreground.set(2, 0, true);
            // set!(1, 1);
            foreground.set(2, 1, true);
            // set!(1, 2);
            foreground.set(2, 2, true);
            foreground.set(2, 3, true);
            foreground.set(2, 4, true);
        }
        'o' | 'O' => {
            foreground.set(1, 0, true);
            foreground.set(0, 1, true);
            foreground.set(2, 1, true);
            foreground.set(0, 2, true);
            foreground.set(2, 2, true);
            foreground.set(0, 3, true);
            foreground.set(2, 3, true);
            foreground.set(1, 4, true);
        }
        'p' | 'P' => {
            foreground.set(0, 0, true);
            foreground.set(1, 0, true);
            foreground.set(0, 1, true);
            foreground.set(2, 1, true);
            foreground.set(0, 2, true);
            foreground.set(1, 2, true);
            foreground.set(0, 3, true);
            foreground.set(0, 4, true);
        }
        'q' | 'Q' => {
            foreground.set(1, 0, true);
            foreground.set(0, 1, true);
            foreground.set(2, 1, true);
            foreground.set(0, 2, true);
            foreground.set(2, 2, true);
            foreground.set(1, 3, true);
            foreground.set(2, 4, true);
        }
        'r' | 'R' => {
            foreground.set(0, 0, true);
            foreground.set(1, 0, true);
            foreground.set(0, 1, true);
            foreground.set(2, 1, true);
            foreground.set(0, 2, true);
            foreground.set(1, 2, true);
            foreground.set(0, 3, true);
            foreground.set(2, 3, true);
            foreground.set(0, 4, true);
            foreground.set(2, 4, true);
        }
        's' | 'S' => {
            foreground.set(1, 0, true);
            foreground.set(2, 0, true);
            foreground.set(0, 1, true);
            foreground.set(1, 2, true);
            foreground.set(2, 3, true);
            foreground.set(0, 4, true);
            foreground.set(1, 4, true);
        }
        't' | 'T' => {
            foreground.set(0, 0, true);
            foreground.set(1, 0, true);
            foreground.set(2, 0, true);
            foreground.set(1, 1, true);
            foreground.set(1, 2, true);
            foreground.set(1, 3, true);
            foreground.set(1, 4, true);
        }
        'u' | 'U' => {
            foreground.set(0, 0, true);
            foreground.set(0, 1, true);
            foreground.set(0, 2, true);
            foreground.set(0, 3, true);
            foreground.set(0, 4, true);
            foreground.set(1, 4, true);
            foreground.set(2, 2, true);
            foreground.set(2, 0, true);
            foreground.set(2, 1, true);
            foreground.set(2, 3, true);
            foreground.set(2, 4, true);
        }
        'v' | 'V' => {
            foreground.set(0, 0, true);
            foreground.set(0, 1, true);
            foreground.set(0, 2, true);
            foreground.set(0, 3, true);
            // set!(1, 3);
            foreground.set(1, 4, true);
            foreground.set(2, 0, true);
            foreground.set(2, 1, true);
            foreground.set(2, 2, true);
            foreground.set(2, 3, true);
        }
        'w' | 'W' => {
            foreground.set(0, 0, true);
            foreground.set(2, 0, true);
            foreground.set(0, 1, true);
            foreground.set(2, 1, true);
            foreground.set(0, 2, true);
            foreground.set(2, 2, true);
            foreground.set(0, 3, true);
            foreground.set(1, 3, true);
            foreground.set(2, 3, true);
            foreground.set(0, 4, true);
            foreground.set(2, 4, true);
        }
        'x' | 'X' => {
            foreground.set(0, 0, true);
            foreground.set(2, 0, true);
            foreground.set(1, 1, true);
            foreground.set(1, 2, true);
            foreground.set(1, 3, true);
            foreground.set(0, 4, true);
            foreground.set(2, 4, true);
        }
        'y' | 'Y' => {
            foreground.set(0, 0, true);
            foreground.set(2, 0, true);
            foreground.set(0, 1, true);
            foreground.set(2, 1, true);
            foreground.set(1, 2, true);
            foreground.set(1, 3, true);
            foreground.set(1, 4, true);
        }
        'z' | 'Z' => {
            foreground.set(0, 0, true);
            foreground.set(1, 0, true);
            foreground.set(2, 0, true);
            foreground.set(2, 1, true);
            foreground.set(1, 2, true);
            foreground.set(0, 3, true);
            foreground.set(0, 4, true);
            foreground.set(1, 4, true);
            foreground.set(2, 4, true);
        }
        '0' => {
            foreground.set(0, 0, true);
            foreground.set(1, 0, true);
            foreground.set(2, 0, true);
            foreground.set(0, 1, true);
            foreground.set(2, 1, true);
            foreground.set(0, 2, true);
            foreground.set(2, 2, true);
            foreground.set(0, 3, true);
            foreground.set(2, 3, true);
            foreground.set(0, 4, true);
            foreground.set(1, 4, true);
            foreground.set(2, 4, true);
        }
        '1' => {
            foreground.set(1, 0, true);
            foreground.set(0, 1, true);
            foreground.set(1, 1, true);
            foreground.set(1, 2, true);
            foreground.set(1, 3, true);
            foreground.set(0, 4, true);
            foreground.set(1, 4, true);
            foreground.set(2, 4, true);
        }
        '2' => {
            foreground.set(0, 0, true);
            foreground.set(1, 0, true);
            foreground.set(2, 0, true);
            foreground.set(2, 1, true);
            foreground.set(2, 2, true);
            foreground.set(1, 2, true);
            foreground.set(0, 3, true);
            foreground.set(0, 4, true);
            foreground.set(1, 4, true);
            foreground.set(2, 4, true);
        }
        '3' => {
            foreground.set(0, 0, true);
            foreground.set(1, 0, true);
            foreground.set(2, 0, true);
            foreground.set(2, 1, true);
            foreground.set(2, 2, true);
            foreground.set(1, 2, true);
            foreground.set(2, 3, true);
            foreground.set(0, 4, true);
            foreground.set(1, 4, true);
            foreground.set(2, 4, true);
        }
        '4' => {
            foreground.set(0, 0, true);
            foreground.set(2, 0, true);
            foreground.set(0, 1, true);
            foreground.set(2, 1, true);
            foreground.set(0, 2, true);
            foreground.set(1, 2, true);
            foreground.set(2, 2, true);
            foreground.set(2, 3, true);
            foreground.set(2, 4, true);
        }
        '5' => {
            foreground.set(0, 0, true);
            foreground.set(1, 0, true);
            foreground.set(2, 0, true);
            foreground.set(0, 1, true);
            foreground.set(0, 2, true);
            foreground.set(1, 2, true);
            foreground.set(2, 2, true);
            foreground.set(2, 3, true);
            foreground.set(0, 4, true);
            foreground.set(1, 4, true);
        }
        '6' => {
            foreground.set(1, 0, true);
            foreground.set(2, 0, true);
            foreground.set(0, 1, true);
            foreground.set(0, 2, true);
            foreground.set(1, 2, true);
            foreground.set(2, 2, true);
            foreground.set(0, 3, true);
            foreground.set(2, 3, true);
            foreground.set(0, 4, true);
            foreground.set(1, 4, true);
            foreground.set(2, 4, true);
        }
        '7' => {
            foreground.set(0, 0, true);
            foreground.set(1, 0, true);
            foreground.set(2, 0, true);
            foreground.set(2, 1, true);
            foreground.set(1, 2, true);
            foreground.set(1, 3, true);
            foreground.set(1, 4, true);
        }
        '8' => {
            foreground.set(0, 0, true);
            foreground.set(1, 0, true);
            foreground.set(2, 0, true);
            foreground.set(0, 1, true);
            foreground.set(2, 1, true);
            foreground.set(0, 2, true);
            foreground.set(1, 2, true);
            foreground.set(2, 2, true);
            foreground.set(0, 3, true);
            foreground.set(2, 3, true);
            foreground.set(0, 4, true);
            foreground.set(1, 4, true);
            foreground.set(2, 4, true);
        }
        '9' => {
            foreground.set(0, 0, true);
            foreground.set(1, 0, true);
            foreground.set(2, 0, true);
            foreground.set(0, 1, true);
            foreground.set(2, 1, true);
            foreground.set(1, 2, true);
            foreground.set(2, 2, true);
            foreground.set(2, 3, true);
            foreground.set(0, 4, true);
            foreground.set(1, 4, true);
        }
        '.' => {
            foreground.set(1, 5, true);
        }
        '-' => {
            foreground.set(0, 2, true);
            foreground.set(1, 2, true);
        }
        '+' => {
            foreground.set(0, 2, true);
            foreground.set(1, 2, true);
            foreground.set(1, 1, true);
            foreground.set(1, 3, true);
            foreground.set(2, 2, true);
        }
        ' ' => {}
        _ => {}
    }

    for x in 0..foreground.width {
        for y in 0..foreground.height {
            if foreground.get(x, y) == &true {
                into.set(*at_x + x, y, color.clone())
            }
        }
    }

    *at_x = *at_x + get_letter_width(&letter);
}

fn get_letter_width(letter: &char) -> u8 {
    match letter {
        'a'..='z' | 'A'..='Z' | '0'..='9' => 3,
        '.' => 1,
        '-' => 2,
        '+' => 3,
        ' ' => 1,
        _ => 0,
    }
}
