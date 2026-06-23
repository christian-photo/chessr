use vampirc_uci::{UciSearchControl, UciTimeControl};

use crate::{
    board::BoardState,
    generated::book::OpeningBook,
    moves::{
        Move, MoveList,
        sliding::{SlidingAttackLookup, precompute_attacks},
    },
    search::{self, ordering::order_moves},
    uci,
};

pub struct ChessrEngine {
    pub board: Option<BoardState>,

    lookup: SlidingAttackLookup,
    book: Option<OpeningBook>,
    settings: ChessrSettings,

    out_of_opening_book: bool,
}

impl ChessrEngine {
    pub fn new() -> Self {
        Self {
            board: None,
            lookup: precompute_attacks(),
            book: OpeningBook::read_from_file("./openings.bin").ok(),
            settings: ChessrSettings::default(),
            out_of_opening_book: false,
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
        fn run(
            depth: u8,
            board: &BoardState,
            moves: &mut [Move],
            lookup: &SlidingAttackLookup,
        ) -> Move {
            let mut best_move = Move::default();
            let mut best_score = i32::MIN;

            let king_checked = BoardState::is_attacked(
                board.bitboards[5 + 6 * !board.is_white_turn() as usize].get_u64(),
                board.bitboards[12].get_u64() | board.bitboards[13].get_u64(),
                &board.bitboards,
                !board.is_white_turn(),
                &lookup,
            );

            order_moves(moves);

            for m in moves.iter() {
                if m.legal(&board, &lookup, king_checked) {
                    let mut new_board = board.clone();
                    new_board.make_move(m);
                    let score = -search::search::depth_search(
                        depth,
                        0,
                        i32::MIN + 1,
                        i32::MAX - 1,
                        &new_board,
                        &lookup,
                    );
                    if score > best_score {
                        best_score = score;
                        best_move = *m;
                    }
                }
            }

            return best_move;
        }

        let mut search_moves: Vec<Move>;
        let mut depth: u8 = 4;

        if let Some(board) = &mut self.board {
            if let Some(book) = &self.book
                && !self.out_of_opening_book
                && self.settings.use_opening_book
            {
                let mut entries = book.find_entries(board.hash.get_u64());

                if entries.len() > 0 {
                    entries.sort_by_key(|e| e.weight);

                    let best_entry = entries.last().unwrap();
                    match Move::from_uci_move(&best_entry.to_uci(), board) {
                        Ok(best_move) => {
                            board.make_move(&best_move);
                            uci::best_move(&best_move.to_uci_move());
                            uci::acknowledge_uci();
                            return;
                        }
                        Err(e) => {
                            eprintln!(
                                "Failed to parse book move for hash {:08x}: {}",
                                board.hash.get_u64(),
                                e
                            )
                        }
                    }
                } else {
                    self.out_of_opening_book = true;
                }
            }

            if let Some(time) = time_control {}

            if let Some(search_control) = search_control {
                if !search_control.search_moves.is_empty() {
                    search_moves = search_control
                        .search_moves
                        .iter()
                        .map(|m| Move::from_uci_move(m, &board).unwrap())
                        .collect::<Vec<Move>>();
                } else {
                    let mut moves = MoveList::new();
                    Move::generate_moves(&mut moves, &board, &self.lookup);
                    search_moves = moves.iter().iter().map(|m| m.clone()).collect();
                }

                depth = search_control.depth.unwrap_or(4);
            } else {
                let mut moves = MoveList::new();
                Move::generate_moves(&mut moves, &board, &self.lookup);
                search_moves = moves.iter().iter().map(|m| m.clone()).collect();
            }

            let best_move = run(depth, board, search_moves.as_mut_slice(), &self.lookup);

            board.make_move(&best_move);
            uci::best_move(&best_move.to_uci_move());
        }
    }

    pub fn stop(&mut self) {}

    pub fn reset(&mut self) {
        self.out_of_opening_book = false;
    }

    pub fn perft(&self, depth: u8, debug: bool) -> u64 {
        fn count_legal_moves(depth: u8, board: &BoardState, lookup: &SlidingAttackLookup) -> u64 {
            if depth == 0 {
                return 1;
            }
            let mut counter = 0u64;

            let mut moves = MoveList::new();
            Move::generate_moves(&mut moves, board, lookup);

            let checked = BoardState::is_attacked(
                board.bitboards[5 + 6 * !board.is_white_turn() as usize].get_u64(),
                board.bitboards[12].get_u64() | board.bitboards[13].get_u64(),
                &board.bitboards,
                !board.is_white_turn(),
                lookup,
            );

            for m in moves.iter() {
                if m.legal(board, lookup, checked) {
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

            let checked = BoardState::is_attacked(
                board.bitboards[5 + 6 * !board.is_white_turn() as usize].get_u64(),
                board.bitboards[12].get_u64() | board.bitboards[13].get_u64(),
                &board.bitboards,
                !board.is_white_turn(),
                &self.lookup,
            );

            for m in moves.iter() {
                if m.legal(&board, &self.lookup, checked) {
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

pub struct ChessrSettings {
    pub use_opening_book: bool,
}

impl Default for ChessrSettings {
    fn default() -> Self {
        Self {
            use_opening_book: true,
        }
    }
}
