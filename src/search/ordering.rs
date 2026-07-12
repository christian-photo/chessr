use crate::{
    board::{BoardState, PieceType},
    moves::{Move, generator::MoveFlag},
    search::{bonus_maps::*, transposition::TranspositionTable},
};

pub fn order_moves(moves: &mut [Move], board: &BoardState, tt: &TranspositionTable) {
    moves.sort_by_cached_key(|m| std::cmp::Reverse(rate_move(m, board, tt)));
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

    match m.get_piece() {
        PieceType::Pawn => {
            score +=
                PAWN_BONUS_MAP[m.target_square as usize] - PAWN_BONUS_MAP[m.start_square as usize]
        }
        PieceType::Knight => {
            score += KNIGHT_BONUS_MAP[m.target_square as usize]
                - KNIGHT_BONUS_MAP[m.start_square as usize]
        }
        PieceType::Bishop => {
            score += BISHOP_BONUS_MAP[m.target_square as usize]
                - BISHOP_BONUS_MAP[m.start_square as usize]
        }
        PieceType::Rook => {
            score +=
                ROOK_BONUS_MAP[m.target_square as usize] - ROOK_BONUS_MAP[m.start_square as usize]
        }
        PieceType::Queen => {
            score +=
                ROOK_BONUS_MAP[m.target_square as usize] - ROOK_BONUS_MAP[m.start_square as usize]
        }
        PieceType::King => {
            score +=
                KING_BONUS_MAP[m.target_square as usize] - KING_BONUS_MAP[m.start_square as usize]
        }
    }

    score
}
