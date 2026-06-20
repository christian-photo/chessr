use crate::board::BoardState;

/// Evaluates a given board position
pub fn evaluate(board: &BoardState) -> i32 {
    let w_piece_count = board.bitboards[12].get_u64().count_ones() as i32;
    let b_piece_count = board.bitboards[13].get_u64().count_ones() as i32;

    let eval = (w_piece_count - b_piece_count) * 100;

    if board.is_white_turn() { eval } else { -eval }
}
