use crate::engine::{color::Color, color_matrix::ColorMatrix, v2::V2};

// Ship sprite pixels (4x4, same as make_ship_sprite):
//  .XX.   row 0
//  .XX.   row 1
//  XXXX   row 2
//  XXXX   row 3
const SHIP_PX: &[(i16, i16)] = &[
    (1,0),(2,0),
    (1,1),(2,1),
    (0,2),(1,2),(2,2),(3,2),
    (0,3),(1,3),(2,3),(3,3),
];

fn is_ship(x: i16, y: i16) -> bool {
    SHIP_PX.iter().any(|&(sx, sy)| sx == x && sy == y)
}

// Build a 6x6 ColorMatrix (1px padding around 4x4) with the ship's outline.
fn make_shield_sprite() -> ColorMatrix {
    let white = Color::new(255, 255, 255, 200);
    let mut m = ColorMatrix::new(6, 6, Color::none());
    for row in 0i16..6 {
        for col in 0i16..6 {
            let ship_x = col - 1;
            let ship_y = row - 1;
            if is_ship(ship_x, ship_y) { continue; }
            let adjacent = (-1i16..=1).flat_map(|dy| (-1i16..=1).map(move |dx| (dx, dy)))
                .any(|(dx, dy)| is_ship(ship_x + dx, ship_y + dy));
            if adjacent {
                m.set(col as u8, row as u8, white);
            }
        }
    }
    m
}

pub fn draw_on_ship(ship_center: V2, rotation_deg: f32, result: &mut ColorMatrix) {
    let sprite = make_shield_sprite();
    result.write(&sprite, &ship_center, Some(rotation_deg), None, None, None);
}
