use crate::{board::BoardState, search::eval};

pub fn search(depth: u8, alpha: i32, beta: i32, board: &BoardState) -> i32 {
    if depth == 0 {
        return eval::evaluate(board);
    }

    0
}
