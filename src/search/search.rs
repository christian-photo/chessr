use std::sync::{Arc, atomic::AtomicBool};

use crate::{
    board::BoardState,
    moves::{Move, MoveList, sliding::SlidingAttackLookup},
    search::{
        eval,
        ordering::order_moves,
        transposition::{TranspositionNodeType, TranspositionTable},
    },
};

#[derive(Clone)]
pub struct PositionSearcher {
    pub best_move: Option<Move>,
    pub best_score: Option<i32>,
    best_move_this_iter: Option<Move>,
    best_score_this_iter: Option<i32>,
    board: BoardState,
    pub current_depth: u8,
    pub search_cancelled: Arc<AtomicBool>,
}

pub const MATE_SCORE: i32 = -1_000_000;
pub const MATE_THRESHOLD: i32 = MATE_SCORE + 100;

impl PositionSearcher {
    pub fn new(board: BoardState) -> Self {
        Self {
            best_move: None,
            best_score: None,
            best_move_this_iter: None,
            best_score_this_iter: None,
            board,
            current_depth: 0,
            search_cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn search(
        &mut self,
        depth: u8,
        iterative_deepening: bool,
        tt: &mut TranspositionTable,
        lookup: &SlidingAttackLookup,
    ) -> Option<Move> {
        let b = self.board.clone();
        if iterative_deepening {
            for search_depth in 1..=depth {
                self.depth_search(search_depth, 0, i32::MIN + 1, i32::MAX - 1, &b, lookup, tt);

                if self
                    .search_cancelled
                    .load(std::sync::atomic::Ordering::Relaxed)
                {
                    break;
                } else {
                    self.current_depth = search_depth;
                    self.best_move = self.best_move_this_iter;
                    self.best_score = self.best_score_this_iter;

                    println!(
                        "info depth {} cp {} pv {}",
                        self.current_depth,
                        self.best_score.unwrap_or(0),
                        self.best_move
                            .map(|m| m.to_uci_move())
                            .unwrap_or("".to_string())
                    );

                    if let Some(best_score) = self.best_score
                        && is_mate_eval(best_score)
                    {
                        break;
                    }
                }
            }
        } else {
            self.depth_search(depth, 0, i32::MIN + 1, i32::MAX - 1, &b, lookup, tt);

            self.best_move = self.best_move_this_iter;
            self.best_score = self.best_score_this_iter;
        }

        return self.best_move;
    }

    fn depth_search(
        &mut self,
        depth: u8,
        ply: u8,
        alpha: i32,
        beta: i32,
        board: &BoardState,
        lookup: &SlidingAttackLookup,
        tt: &mut TranspositionTable,
    ) -> i32 {
        if board.check_halfmoves() || board.check_threefold_repetition() {
            return -50; // Slightly discourage drawíng
        }

        if let Some(transposition) = tt.lookup(board.hash.get_u64(), depth, ply, alpha, beta) {
            if ply == 0 {
                self.best_move_this_iter = Some(transposition.best_move);
                self.best_score_this_iter = Some(transposition.eval);
            }
            return transposition.eval;
        }

        if depth == 0 {
            return self.quiescence_search(alpha, beta, board, lookup, tt);
        }

        let mut alpha = alpha;

        let mut list = MoveList::new();
        Move::generate_moves(&mut list, board, lookup);
        let moves = list.iter();
        order_moves(moves, &board, tt);

        let king_index = if board.is_white_turn() { 5 } else { 11 };

        let checked = BoardState::is_attacked(
            board.bitboards[king_index].get_u64(),
            board.bitboards[12].get_u64() | board.bitboards[13].get_u64(),
            &board.bitboards,
            !board.is_white_turn(),
            &lookup,
        );

        let mut legal_move = None;

        let mut node_type = TranspositionNodeType::UpperBound;
        let mut best_move = None;

        for m in moves {
            if m.legal(board, lookup, checked) {
                legal_move = Some(m.clone());
                let mut new_board = board.clone();
                new_board.make_move(m);
                let score =
                    -self.depth_search(depth - 1, ply + 1, -beta, -alpha, &new_board, lookup, tt);

                if self
                    .search_cancelled
                    .load(std::sync::atomic::Ordering::Relaxed)
                {
                    return alpha;
                }

                if score >= beta {
                    tt.store(
                        board.hash.get_u64(),
                        score,
                        depth,
                        ply,
                        TranspositionNodeType::LowerBound,
                        m.clone(),
                    );
                    return score;
                }
                if score > alpha {
                    node_type = TranspositionNodeType::Exact;
                    best_move = Some(m.clone());
                    alpha = score;

                    if ply == 0 {
                        self.best_move_this_iter = Some(m.clone());
                        self.best_score_this_iter = Some(score);
                    }
                }
            }
        }

        if legal_move.is_none() {
            if checked {
                return MATE_SCORE + ply as i32; // checkmate, prefer faster mates
            } else {
                return 0; // stalemate
            }
        }

        tt.store(
            board.hash.get_u64(),
            alpha,
            depth,
            ply,
            node_type,
            best_move.unwrap_or_else(|| legal_move.unwrap()),
        );

        return alpha;
    }

    pub fn quiescence_search(
        &mut self,
        alpha: i32,
        beta: i32,
        board: &BoardState,
        lookup: &SlidingAttackLookup,
        tt: &mut TranspositionTable,
    ) -> i32 {
        let king_index = if board.is_white_turn() { 5 } else { 11 };
        let checked = BoardState::is_attacked(
            board.bitboards[king_index].get_u64(),
            board.bitboards[12].get_u64() | board.bitboards[13].get_u64(),
            &board.bitboards,
            !board.is_white_turn(),
            &lookup,
        );

        let mut alpha = alpha;
        if !checked {
            let eval = eval::evaluate(board);
            if eval >= beta {
                return beta;
            }
            if eval > alpha {
                alpha = eval;
            }
        }

        let mut list = MoveList::new();
        if checked {
            // Generate all moves, not just captures
            Move::generate_moves(&mut list, board, lookup);
        } else {
            Move::generate_captures_only(&mut list, board, lookup);
        }
        let moves = list.iter();
        order_moves(moves, &board, tt);

        for m in moves {
            if m.legal(board, lookup, checked) {
                let mut new_board = board.clone();
                new_board.make_move(m);
                let score = -self.quiescence_search(-beta, -alpha, &new_board, lookup, tt);

                if self
                    .search_cancelled
                    .load(std::sync::atomic::Ordering::Relaxed)
                {
                    return alpha;
                }

                if score >= beta {
                    return score;
                }
                if score > alpha {
                    alpha = score;
                }
            }
        }

        return alpha;
    }
}

pub fn is_mate_eval(eval: i32) -> bool {
    eval <= MATE_THRESHOLD || eval >= -MATE_THRESHOLD
}
