use crate::{
    board::BoardState,
    moves::{Move, generator::MoveFlag},
    search::transposition::TranspositionTable,
};

pub fn order_moves(moves: &mut [Move], board: &BoardState, tt: &TranspositionTable) {
    moves.sort_by(|m1, m2| rate_move(m2, board, tt).cmp(&rate_move(m1, board, tt)));
}

fn rate_move(m: &Move, board: &BoardState, tt: &TranspositionTable) -> i32 {
    let mut score = 0;

    if let Some(entry) = tt.get(board.hash.get_u64())
        && entry.key == board.hash.get_u64()
        && entry.best_move == *m
    {
        score = 10000;
    }

    if let Some(capture) = m.capture {
        score += 20;

        if capture.get_value() > m.get_piece().get_value() {
            score += capture.get_value() - m.get_piece().get_value();
        }
    } else if m.promotion.is_some() {
        score += 100;
    } else if m.get_flag() == MoveFlag::CastleKingside || m.get_flag() == MoveFlag::CastleQueenside
    {
        score += 20;
    }

    score
}
