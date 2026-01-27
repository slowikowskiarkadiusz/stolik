#[derive(Copy, Clone)]
pub enum Key {
    P1Down,
    P1Up,
    P1Left,
    P1Right,
    P1AnyDirection,
    P1Blue,
    P1Green,
    P1Any,
    P2Down,
    P2Up,
    P2Left,
    P2Right,
    P2AnyDirection,
    P2Blue,
    P2Green,
    P2Any,
    Start,
}

pub static KEYS_LENGTH: u8 = Key::Start as u8;

pub enum KeyState {
    Down,
    Up,
    Press,
}
