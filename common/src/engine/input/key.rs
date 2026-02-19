#[derive(Copy, Clone, PartialEq)]
pub enum Key {
    P1Down = 0,
    P1Up = 1,
    P1Left = 2,
    P1Right = 3,
    P1AnyDirection = 4,
    P1Blue = 5,
    P1Green = 6,
    P1Any = 7,
    P2Down = 8,
    P2Up = 9,
    P2Left = 10,
    P2Right = 11,
    P2AnyDirection = 12,
    P2Blue = 13,
    P2Green = 14,
    P2Any = 15,
    Start = 16,
}

pub static KEYS_LENGTH: u8 = Key::Start as u8 + 1;

pub enum KeyState {
    Down,
    Up,
    Press,
}

pub fn u8_to_key(k: u8) -> Key {
    match k {
        0 => Key::P1Down,
        1 => Key::P1Up,
        2 => Key::P1Left,
        3 => Key::P1Right,
        4 => Key::P1AnyDirection,
        5 => Key::P1Blue,
        6 => Key::P1Green,
        7 => Key::P1Any,
        8 => Key::P2Down,
        9 => Key::P2Up,
        10 => Key::P2Left,
        11 => Key::P2Right,
        12 => Key::P2AnyDirection,
        13 => Key::P2Blue,
        14 => Key::P2Green,
        15 => Key::P2Any,
        16 => Key::Start,
        _ => panic!("can't transform {} to key", k),
    }
}
