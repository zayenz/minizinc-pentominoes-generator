use crate::Board;
use itertools::Itertools;

/// Flip each row in the board
fn flip1(board: &Board) -> Board {
    board
        .iter()
        .map(|row| row.iter().rev().cloned().collect_vec())
        .collect_vec()
}

/// Rotate the board 90 degrees
fn rot90(board: &Board) -> Board {
    let size = board.len();
    let mut result = vec![vec![0; size]; size];
    for w in 0..size {
        for h in 0..size {
            result[w][size - h - 1] = board[h][w];
        }
    }
    result
}

/// Generate all 8 symmetries of a board
pub fn generate_symmetric_boards(board: &Board) -> Vec<Board> {
    vec![
        board.clone(),
        rot90(board),
        rot90(&rot90(board)),
        rot90(&(rot90(&rot90(board)))),
        flip1(board),
        flip1(&rot90(board)),
        flip1(&rot90(&rot90(board))),
        flip1(&rot90(&(rot90(&rot90(board))))),
    ]
}
