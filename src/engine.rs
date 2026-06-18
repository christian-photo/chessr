use vampirc_uci::{UciSearchControl, UciTimeControl};

use crate::{
    board::{BoardState, Piece},
    moves::{
        Move, MoveList,
        magic::Random,
        sliding::{SlidingAttackLookup, precompute_attacks},
    },
};

pub struct ChessrEngine {
    pub board: Option<BoardState>,

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
    pub fn set_board(&mut self, board: BoardState) {
        self.board = Some(board);
    }

    pub fn make_moves(&mut self, moves: &[Move]) {
        if let Some(board) = &mut self.board {
            for move_desc in moves {
                board.make_move(move_desc);
            }
        }
    }

    pub fn go(
        &mut self,
        time_control: Option<UciTimeControl>,
        search_control: Option<UciSearchControl>,
    ) {
        if let Some(time) = time_control {}

        if let Some(search_control) = search_control {}
    }

    pub fn stop(&mut self) {}

    pub fn perft(&self, depth: u8, debug: bool) -> u64 {
        fn count_legal_moves(depth: u8, board: &BoardState, lookup: &SlidingAttackLookup) -> u64 {
            if depth == 0 {
                return 1;
            }
            let mut counter = 0u64;

            let mut moves = MoveList::new();
            Move::generate_moves(&mut moves, board, lookup);
            for m in moves.iter() {
                if m.legal(board, lookup) {
                    let mut new_board = board.clone();
                    new_board.make_move(m);

                    counter += count_legal_moves(depth - 1, &new_board, lookup);
                }
            }

            return counter;
        }

        if let Some(board) = &self.board {
            let mut counter = 0u64;

            let mut moves = MoveList::new();
            Move::generate_moves(&mut moves, &board, &self.lookup);
            for m in moves.iter() {
                if m.legal(&board, &self.lookup) {
                    let mut new_board = board.clone();
                    new_board.make_move(m);

                    let old_counter = counter;

                    counter += count_legal_moves(depth - 1, &new_board, &self.lookup);

                    if debug {
                        println!("{}: {}", m.to_uci_move(), counter - old_counter);
                    }
                }
            }

            return counter;
        } else {
            0
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ZobristHash {
    hash: u64,

    table: [[u64; 12]; 64],

    /// Should be XORed in to the hash if it is blacks turn
    side_to_move: u64,

    /// KQkq
    castling: [u64; 4],

    /// The file of the en passant square is the index
    en_passant: [u64; 8],
}

impl ZobristHash {
    pub fn new() -> Self {
        let mut table = [[0u64; 12]; 64];
        let mut castling = [0u64; 4];
        let mut en_passant = [0u64; 8];
        let mut rand = Random::new(170620260930);

        for sq in 0..64 {
            for p in 0..12 {
                table[sq as usize][p as usize] = rand.random();
            }
        }

        for i in 0..4 {
            castling[i as usize] = rand.random();
        }

        for file in 0..8 {
            en_passant[file as usize] = rand.random();
        }

        Self {
            hash: 0,
            table,
            castling,
            en_passant,
            side_to_move: rand.random(),
        }
    }

    pub fn hash_board(&mut self, board: &BoardState) {
        board.pieces.iter().enumerate().for_each(|(sq, pc)| {
            if let Some(piece) = pc {
                self.update(sq as u8, piece);
            }
        });

        if board.castling_rights.king_side(true) {
            self.hash ^= self.castling[0];
        }
        if board.castling_rights.queen_side(true) {
            self.hash ^= self.castling[1];
        }
        if board.castling_rights.king_side(false) {
            self.hash ^= self.castling[2];
        }
        if board.castling_rights.queen_side(false) {
            self.hash ^= self.castling[3];
        }

        if let Some(sq) = board.en_passant {
            self.hash ^= self.en_passant[sq as usize % 8];
        }

        if !board.is_white_turn() {
            self.hash ^= self.side_to_move;
        }
    }

    #[inline(always)]
    pub fn update(&mut self, square: u8, piece: &Piece) {
        self.hash ^= self.table[square as usize][piece.to_bitboard_index()];
    }

    #[inline(always)]
    pub fn en_passant(&mut self, square: u8) {
        self.hash ^= self.en_passant[square as usize % 8];
    }

    #[inline(always)]
    pub fn change_side(&mut self) {
        self.hash ^= self.side_to_move;
    }

    #[inline(always)]
    pub fn castling(&mut self, kingside: bool, white: bool) {
        self.hash ^= self.castling[!kingside as usize | ((!white as usize) << 1)];
    }
}
