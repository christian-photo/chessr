use crate::{
    board::Board,
    moves::{
        Move,
        sliding::{SlidingAttackLookup, precompute_attacks},
    },
};

pub struct ChessrEngine {
    board: Option<Board>,

    lookup: SlidingAttackLookup,
}

impl ChessrEngine {
    pub fn new() -> Self {
        Self {
            board: None,
            lookup: precompute_attacks(),
        }
    }

    pub fn author() -> String {
        "Christian Palm".into()
    }

    pub fn name() -> String {
        "Chessr".into()
    }
}

impl ChessrEngine {
    pub fn set_board(&mut self, board: Board) {
        self.board = Some(board);
    }

    pub fn make_moves(&mut self, moves: &[Move]) {
        if let Some(mut board) = self.board {
            for move_desc in moves {
                board.make_move(move_desc);
            }
        }
    }

    pub fn go(&mut self) {}
    pub fn stop(&mut self) {}
}
