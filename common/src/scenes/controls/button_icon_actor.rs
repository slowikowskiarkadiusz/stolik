use crate::engine::{
    color::Color,
    color_matrix::ColorMatrix,
    components::{transform::Transform, world::World},
    engine::ActorId,
    input::key::Key,
    v2::V2,
};

const LEFT_KEYS: [Key; 2] = [Key::P1Left, Key::P2Left];
const RIGHT_KEYS: [Key; 2] = [Key::P1Right, Key::P2Right];
const UP_KEYS: [Key; 2] = [Key::P1Up, Key::P2Up];
const DOWN_KEYS: [Key; 2] = [Key::P1Down, Key::P2Down];

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
            color_matrix.set(if is_down_key { 1 } else { 3 }, 2, Color::black());
            color_matrix.set(3, 2, Color::black());
        }
    }

    color_matrix
}

fn map_key_to_color(key: Key) -> Color {
    match key {
        Key::P1Down
        | Key::P1Up
        | Key::P1Left
        | Key::P1Right
        | Key::P1AnyDirection
        | Key::P2Down
        | Key::P2Up
        | Key::P2Left
        | Key::P2Right
        | Key::P2AnyDirection => Color::red(),
        Key::P1Blue | Key::P2Blue => Color::green(),
        Key::P1Green | Key::P2Green => Color::blue(),
        Key::Start => Color::white(),
        _ => Color::yellow(),
    }
}
