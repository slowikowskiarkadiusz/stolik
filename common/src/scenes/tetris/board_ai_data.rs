use crate::{
    engine::{matrix::Matrix, v2::V2},
    scenes::tetris::{
        block::Block,
        board::{BOARD_HEIGHT, BOARD_WIDTH, Board, TetrisAiData},
        shape::Shape,
    },
};

pub fn get_future_board(current_agent: &Block, is_cell_taken: &Matrix<bool>, center_x: u8, rotation: u16) -> Option<Matrix<bool>> {
    let shape = current_agent.shape.clone();

    let mut block = Block::new(V2::new(center_x as f32, 0.0), shape, false);
    if rotation > 0 {
        block.rotate_block(rotation as i32);
    }

    // sprawdź czy klocek w ogóle mieści się w tej kolumnie (x-bounds)
    let initial_spots = block.get_taken_spots();
    if initial_spots.iter().any(|f| f.x < 0.0 || f.x >= BOARD_WIDTH as f32) {
        return None;
    }

    // opuść klocek tak jak calc_drop w spawn() — start=1
    let drop = Board::calc_drop(&is_cell_taken, 1, initial_spots);
    block.center.y += drop as f32;

    // nanieś na kopię planszy
    let mut future = is_cell_taken.clone();
    for spot in block.get_taken_spots() {
        let x = spot.x as i16;
        let y = spot.y as i16;
        if x >= 0 && y >= 0 && (x as u8) < BOARD_WIDTH && (y as u8) < BOARD_HEIGHT {
            future.set(x as u8, y as u8, true);
        }
    }

    Some(future)
}

pub fn get_all_possible_future_boards_ai_data(
    is_cell_taken: &Matrix<bool>,
    current_agent: &Block,
) -> [Option<TetrisAiData>; BOARD_WIDTH as usize * 4] {
    let mut result: [Option<TetrisAiData>; BOARD_WIDTH as usize * 4] = [None; BOARD_WIDTH as usize * 4];

    for rotation in [0u16, 90, 180, 270] {
        for x in 0..BOARD_WIDTH {
            if let Some(future_board) = get_future_board(current_agent, is_cell_taken, x, rotation) {
                let column_heights = get_column_heights(&future_board);

                result[x as usize * 4 + (rotation / 90) as usize] = Some(TetrisAiData {
                    column_heights,
                    bumpiness: get_bumpiness(&column_heights),
                    holes: get_holes_count(&future_board),
                    aggregate_height: (&column_heights).iter().sum(),
                    max_height: (&column_heights).iter().max().unwrap().clone(),
                    lines_cleared: get_lines_cleared(&future_board),
                    current_piece: {
                        let mut result: [u8; 7] = [0; 7];
                        result[current_agent.shape.clone() as usize] = 1;
                        result
                    },
                    current_rotation: {
                        let mut result: [u8; 4] = [0; 4];
                        result[current_agent.rotation as usize / 90] = 1;
                        result
                    },
                    piece_x: x,
                    // next_piece: {
                    //     let mut result: [u8; 7] = [0; 7];
                    //     result[next_shape.clone() as u8] = 1;
                    //     result
                    // },
                })
            } else {
                result[x as usize] = None;
            }
        }
    }

    result
}

fn get_lines_cleared(is_cell_taken: &Matrix<bool>) -> u8 {
    let mut line_count = 0;

    for y in 0..is_cell_taken.height {
        let mut is_whole_line_taken = false;

        for x in 0..is_cell_taken.width {
            is_whole_line_taken = *is_cell_taken.get(x, y);
            if !is_whole_line_taken {
                break;
            }
        }

        if is_whole_line_taken {
            line_count += 1;
        }
    }

    line_count
}

fn get_column_heights(is_cell_taken: &Matrix<bool>) -> [u8; BOARD_WIDTH as usize] {
    let mut result = [0; BOARD_WIDTH as usize];

    for x in 0..is_cell_taken.width {
        for y in 0..is_cell_taken.height {
            if *is_cell_taken.get(x, y) {
                result[x as usize] += 1;
            }
        }
    }

    result
}

fn get_bumpiness(heights: &[u8; BOARD_WIDTH as usize]) -> [u8; BOARD_WIDTH as usize - 1] {
    let mut bumpiness = [0; BOARD_WIDTH as usize - 1];
    for i in 0..9 {
        bumpiness[i] = (heights[i] as i8 - heights[i + 1] as i8).abs() as u8;
    }
    bumpiness
}

fn get_holes_count(is_cell_taken: &Matrix<bool>) -> u8 {
    let mut is_cell_taken = is_cell_taken.clone();
    let mut hole_count = 0;

    for x in 0..is_cell_taken.width {
        for y in 0..is_cell_taken.height {
            if !is_cell_taken.get(x, y) {
                hole_count += 1;
                visit(x, y, &mut is_cell_taken);
            }
        }
    }

    fn visit(x: u8, y: u8, is_cell_taken: &mut Matrix<bool>) {
        if x >= 0 && y >= 0 && x < is_cell_taken.width && y < is_cell_taken.height && !*is_cell_taken.get(x, y) {
            is_cell_taken.set(x, y, true);
            if let Some(newx) = x.checked_sub(1) {
                visit(newx, y, is_cell_taken);
            }
            if let Some(newx) = x.checked_add(1) {
                visit(newx, y, is_cell_taken);
            }
            if let Some(newy) = y.checked_sub(1) {
                visit(x, newy, is_cell_taken);
            }
            if let Some(newy) = y.checked_add(1) {
                visit(x, newy, is_cell_taken);
            }
        }
    }

    hole_count
}
