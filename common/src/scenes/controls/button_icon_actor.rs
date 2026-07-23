use crate::engine::{color::Color, color_matrix::ColorMatrix, input::key::Key};

const LEFT_KEYS: [Key; 2] = [Key::Left, Key::Left];
const RIGHT_KEYS: [Key; 2] = [Key::Right, Key::Right];
const UP_KEYS: [Key; 2] = [Key::Up, Key::Up];
const DOWN_KEYS: [Key; 2] = [Key::Down, Key::Down];

// pub fn create_button_icon_actor(world: &mut World, center: V2, size: u8, key: Key, _name: Option<&str>) -> ActorId {
//     world.add_new_actor(
//         Some(Transform::new(center, V2::one() * size as f32)),
//         None,
//         None,
//         None,
//         // Some(make_matrix(size, key)),
//     )
// }

pub fn make_button_matrix(size: u8, key: Key) -> ColorMatrix {
    let color = map_key_to_color(key);
    let mut color_matrix = ColorMatrix::new(size, size, Color::none());

    for x in 0..size {
        for y in 0..size {
            if (x == 0 || x == size - 1) && (y != 0 && y != size - 1) {
                color_matrix.set(x, y, color.clone());
            } else if (y == 0 || y == size - 1) && (x != 0 && x != size - 1) {
                color_matrix.set(x, y, color.clone());
            } else if x > 0 && x < size - 1 && y > 0 && y < size - 1 {
                color_matrix.set(x, y, color.clone());
            }
        }
    }

    let is_left_key = LEFT_KEYS.contains(&key);
    let is_right_key = RIGHT_KEYS.contains(&key);

    if is_left_key || is_right_key {
        color_matrix.set(2, 1, Color::black());
        color_matrix.set(if is_left_key { 1 } else { 3 }, 2, Color::black());
        color_matrix.set(2, 3, Color::black());
    } else {
        let is_up_key = UP_KEYS.contains(&key);
        let is_down_key = DOWN_KEYS.contains(&key);

        if is_up_key || is_down_key {
            color_matrix.set(1, 2, Color::black());
            color_matrix.set(2, if is_down_key { 1 } else { 3 }, Color::black());
            color_matrix.set(3, 2, Color::black());
        }
    }

    color_matrix
}

fn map_key_to_color(key: Key) -> Color {
    match key {
        Key::Down | Key::Up | Key::Left | Key::Right | Key::AnyDirection => Color::red(),
        Key::Blue => Color::green(),
        Key::Green => Color::blue(),
        Key::Start => Color::white(),
        _ => Color::yellow(),
    }
}
