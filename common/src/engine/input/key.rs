#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum Key {
    Down = 0,
    Up = 1,
    Left = 2,
    Right = 3,
    AnyDirection = 4,
    Blue = 5,
    Green = 6,
    Any = 7,
    Start = 8,
}

pub static KEYS_LENGTH: u8 = Key::Start as u8 + 1;

pub enum KeyState {
    Down,
    Up,
    Press,
}

pub fn u8_to_key(k: u8) -> Key {
    match k {
        0 => Key::Down,
        1 => Key::Up,
        2 => Key::Left,
        3 => Key::Right,
        4 => Key::AnyDirection,
        5 => Key::Blue,
        6 => Key::Green,
        7 => Key::Any,
        8 => Key::Start,
        _ => panic!("can't transform {} to key", k),
    }
}
