use crate::{
    board::BoardState,
    moves::{Move, MoveList, sliding::SlidingAttackLookup},
    search::{eval, ordering::order_moves},
};

pub fn depth_search(
    depth: u8,
    ply: u8,
    alpha: i32,
    beta: i32,
    board: &BoardState,
    lookup: &SlidingAttackLookup,
) -> i32 {
    if board.check_halfmoves() || board.check_threefold_repetition() {
        return 0; // Draw
    }
    if depth == 0 {
        return eval::evaluate(board);
    }

    let mut alpha = alpha;

    let mut list = MoveList::new();
    Move::generate_moves(&mut list, board, lookup);
    let moves = list.iter();
    order_moves(moves);

    let king_index = if board.is_white_turn() { 5 } else { 11 };

    let checked = BoardState::is_attacked(
        board.bitboards[king_index].get_u64(),
        board.bitboards[12].get_u64() | board.bitboards[13].get_u64(),
        &board.bitboards,
        !board.is_white_turn(),
        &lookup,
    );

    let mut legal_move_found = false;

    for m in moves {
        if m.legal(board, lookup, checked) {
            legal_move_found = true;
            let mut new_board = board.clone();
            new_board.make_move(m);
            let score = -depth_search(depth - 1, ply + 1, -beta, -alpha, &new_board, lookup);

            if score >= beta {
                return score;
            }
            if score > alpha {
                alpha = score;
            }
        }
    }

    if !legal_move_found {
        if checked {
            return -1_000_000 + ply as i32; // checkmate, prefer faster mates
        } else {
            return 0; // stalemate
        }
    }

    return alpha;
}

pub struct SearchResult {
    pub best_move: Move,
    pub best_score: i32,
}
