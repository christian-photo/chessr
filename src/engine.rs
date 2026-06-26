use vampirc_uci::{UciSearchControl, UciTimeControl};

use crate::{
    board::BoardState,
    generated::book::OpeningBook,
    moves::{
        Move, MoveList,
        sliding::{SlidingAttackLookup, precompute_attacks},
    },
    search::{
        self, ordering::order_moves, search::PositionSearcher, transposition::TranspositionTable,
    },
    uci,
};

pub struct ChessrEngine {
    pub board: Option<BoardState>,

    lookup: SlidingAttackLookup,
    book: Option<OpeningBook>,
    transposition_table: TranspositionTable,

    settings: ChessrSettings,

    out_of_opening_book: bool,
}

impl ChessrEngine {
    pub fn new() -> Self {
        Self {
            board: None,
            lookup: precompute_attacks(),
            book: OpeningBook::read_from_file("./openings.bin").ok(),
            transposition_table: TranspositionTable::new(),
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
    pub fn set_option(&mut self, name: String, value: Option<String>) {
        match name.as_str() {
            "Threads" => {
                // if self.settings.can_set_thread_count
                //     && let Some(value) = value
                // {
                //     if let Ok(thread_count) = usize::from_str_radix(value.as_str(), 10) {
                //         rayon::ThreadPoolBuilder::new()
                //             .num_threads(thread_count)
                //             .build_global()
                //             .unwrap();
                //         self.settings.thread_count = thread_count;
                //         self.settings.can_set_thread_count = false;
                //     }
                // }
            }
            "IterativeDeepening" => {
                if let Some(value) = value {
                    if value == "true" {
                        self.settings.iterative_deepening = true;
                    } else if value == "false" {
                        self.settings.iterative_deepening = false;
                    }
                }
            }
            "OwnBook" => {
                if let Some(value) = value {
                    if value == "true" {
                        self.settings.use_opening_book = true;
                    } else if value == "false" {
                        self.settings.use_opening_book = false;
                    }
                }
            }
            _ => {}
        }
    }

    pub fn send_options(&self) {
        println!("option name OwnBook type check default true");
        println!("option name IterativeDeepening type check default true");
        if self.settings.can_set_thread_count {
            println!(
                "option name Threads type spin default {} min 1 max 1024",
                self.settings.thread_count
            );
        }
    }

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
                depth = search_control.depth.unwrap_or(4);
            }

            let mut searcher = PositionSearcher::new(board.clone());

            let best_move = searcher.search(
                depth,
                self.settings.iterative_deepening,
                &mut self.transposition_table,
                &self.lookup,
            );

            if let Some(m) = best_move {
                board.make_move(&m);
                uci::best_move(&m.to_uci_move());
            }
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
    pub thread_count: usize,
    pub iterative_deepening: bool,

    pub can_set_thread_count: bool,
}

impl Default for ChessrSettings {
    fn default() -> Self {
        Self {
            use_opening_book: true,
            can_set_thread_count: true,
            iterative_deepening: true,
            thread_count: std::thread::available_parallelism().map_or(1, |p| p.get()),
        }
    }
}
