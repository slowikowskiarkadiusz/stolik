#[derive(Clone, PartialEq, Eq)]
pub enum Shape {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

pub fn get_shape(i: u8) -> Shape {
    match i {
        0 => Shape::I,
        1 => Shape::O,
        2 => Shape::T,
        3 => Shape::S,
        4 => Shape::Z,
        5 => Shape::J,
        6 => Shape::L,
        _ => panic!("no shape for {}", i),
    }
}
