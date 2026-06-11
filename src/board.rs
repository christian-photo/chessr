use crate::moves::{
    generator::{Move, MoveFlag},
    sliding::SlidingAttackLookup,
};
pub use crate::piece::{Piece, PieceType};

#[derive(Debug, Clone, Copy)]
pub struct CastlingRights {
    /// Meaning of individual bits:
    /// - 1st bit indicates white kingside
    /// - 2nd bit indicates white queenside
    /// - 3rd bit indicates if white has castled
    /// - 4rd bit indicates black kingside
    /// - 5th bit indicates black queenside
    /// - 6th bit indicates if black has castled
    rights: u8,
}

impl CastlingRights {
    #[inline(always)]
    pub fn king_side(&self, white: bool) -> bool {
        (self.rights & (0b1 << (2 * !white as u8))) != 0
    }

    #[inline(always)]
    pub fn queen_side(&self, white: bool) -> bool {
        (self.rights & (0b01 << (2 * !white as u8))) != 0
    }

    #[inline(always)]
    pub fn lose_king_side(&mut self, white: bool) {
        self.rights &= !(0b1 << (2 * !white as u8));
    }

    #[inline(always)]
    pub fn lose_queen_side(&mut self, white: bool) {
        self.rights &= !(0b01 << (2 * !white as u8));
    }

    #[inline(always)]
    pub fn lose(&mut self, king_side: bool, white: bool) {
        self.rights &= !(0b1 << (2 * !white as u8 + !king_side as u8))
    }

    #[inline(always)]
    pub fn gain_king_side(&mut self, white: bool) {
        self.rights |= 0b1 << (2 * !white as u8);
    }

    #[inline(always)]
    pub fn gain_queen_side(&mut self, white: bool) {
        self.rights |= 0b01 << (2 * !white as u8);
    }

    #[inline(always)]
    pub fn gain(&mut self, king_side: bool, white: bool) {
        self.rights |= 0b1 << (2 * !white as u8 + !king_side as u8)
    }

    pub fn none() -> Self {
        Self { rights: 0 }
    }
}

impl Default for CastlingRights {
    fn default() -> Self {
        Self { rights: 0b1111 }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Bitboard {
    board: u64,
}

impl Bitboard {
    pub fn empty() -> Bitboard {
        Bitboard { board: 0 }
    }

    /// Does not perform bounds checking (0..63) on `pos`
    pub fn add_piece(self: &mut Bitboard, pos: u8) {
        self.board = self.board | 1u64 << pos;
    }

    /// Sets the corresponding bit to 0, if the bit already was 0, nothing changes
    pub fn remove_piece(self: &mut Bitboard, pos: u8) {
        // Performs a bitwise AND operation on the board and flipped position bits
        // We flip the position bits because we want to leave everything in the board unchanged,
        // except for the desired position
        // Example:
        // 0b1101 & !0b0100
        // 0b1101 & 0b1011
        // 0b1011
        self.board = self.board & !(1u64 << pos);
    }

    pub fn has_piece_at(self: &Bitboard, pos: u8) -> bool {
        let bitboard_pos = 1u64 << pos;
        self.board & bitboard_pos > 0
    }

    pub fn get_u64(&self) -> u64 {
        self.board
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Board {
    /// Piece and color specific bitboards, which should be used for move generation. The last two are white and black pieces
    /// - WPawn (index 0)
    /// - WKnight
    /// - WBishop
    /// - WRook
    /// - WQueen
    /// - WKing
    /// - BPawn
    /// - BKnight
    /// - ...
    /// - White pieces (index 12)
    /// - Black pieces (index 13)
    pub bitboards: [Bitboard; 14],

    /// Array containing all pieces on the board
    pub pieces: [Option<Piece>; 64],

    white_turn: bool,

    /// Holds the castling rights for both white and black, works with a single byte internally
    pub castling_rights: CastlingRights,

    /// If en passant is possible, this is Some with the square behind the pawn as its value
    pub en_passant: Option<u8>,
    // More than likely irrelevant for our purposes
    // half_moves: u8,
    // full_moves: u16,
}

impl Board {
    pub fn empty() -> Board {
        Board {
            bitboards: [Bitboard::empty(); 14],
            pieces: [None; 64],
            white_turn: true,
            castling_rights: CastlingRights::none(),
            en_passant: None,
        }
    }

    pub fn from_fen(fen: &str) -> Result<Board, String> {
        let mut board = Board::empty();

        let mut iter = fen.split_ascii_whitespace();
        let placement = iter.next().expect("Malformed FEN string");
        let ranks = placement.split('/');

        let mut current_pos = 64u8;

        for rank in ranks {
            for char in rank.chars().rev() {
                if char.is_ascii_alphabetic() {
                    current_pos -= 1;
                    let is_white = char.is_ascii_uppercase();

                    if let Some(piece_type) =
                        PieceType::from_abbreviation(char.to_ascii_lowercase())
                    {
                        let piece = Piece::new(piece_type, is_white);

                        board.add_piece(piece, current_pos);
                    } else {
                        return Err(format!(
                            "Encountered an unknown piece description: {0}",
                            char
                        ));
                    }
                } else if let Some(num) = char.to_digit(10) {
                    current_pos -= num as u8;
                }
            }
        }

        board.white_turn = iter.next().expect("Malformed FEN string") == "w";

        let castling_rights = iter.next().expect("Malformed FEN string");
        for c in castling_rights.chars() {
            if c == '-' {
                break;
            }

            let white = c.is_ascii_uppercase();
            let king_side = c.to_ascii_lowercase() == 'k';
            board.castling_rights.gain(king_side, white);
        }

        let en_passant_target = iter.next().expect("Malformed FEN string");
        if en_passant_target != "-" {
            let square = Board::algebraic_to_u8(en_passant_target);
            if square < 64 {
                board.en_passant = Some(square);
            }
        }

        // Half and full moves are intentionally ignored for now

        Ok(board)
    }

    pub fn algebraic_to_u8(notation: &str) -> u8 {
        let mut chars = notation.chars();

        let file = chars
            .next()
            .expect("Algebraic notation needs 2 chars")
            .to_ascii_lowercase() as u8
            - 'a' as u8;
        let rank = chars
            .next()
            .expect("Algebraic notation needs 2 chars")
            .to_digit(10)
            .unwrap() as u8
            - 1;

        file + rank * 8
    }

    pub fn to_ascii(&self) -> String {
        let mut repr: String = "".to_string();

        for row in 0..16 {
            if row % 2 == 0 {
                repr += "+---+---+---+---+---+---+---+---+";
            } else {
                let row_pos = (7 - row / 2) * 8;
                for file in 0..8 {
                    repr += "| ";
                    match self.piece_at(row_pos + file) {
                        Some(piece) => repr = repr + &piece.notation().to_string() + " ",
                        None => repr += "  ",
                    }
                }
                repr += "|";
            }
            repr += "\n";
        }
        repr += "+---+---+---+---+---+---+---+---+";

        return repr;
    }

    pub fn add_piece(&mut self, piece: Piece, pos: u8) {
        debug_assert!(
            self.pieces[pos as usize].is_none(),
            "square {pos} is already occupied"
        );
        // Update piece type array
        self.pieces[pos as usize] = Some(piece);

        let index = piece.to_bitboard_index();

        // Update piece and color specific bitboard
        self.bitboards[index].add_piece(pos);

        // Update color specific bitboard
        self.bitboards[12 + !piece.is_white() as usize].add_piece(pos);
    }

    pub fn remove_piece(&mut self, pos: u8) {
        if let Some(piece) = self.pieces[pos as usize] {
            self.pieces[pos as usize] = None;
            let index = piece.to_bitboard_index();
            self.bitboards[index].remove_piece(pos);

            self.bitboards[12 + !piece.is_white() as usize].remove_piece(pos);
        } else {
            debug_assert!(
                self.pieces[pos as usize].is_none(),
                "square {pos} is not occupied"
            );
        }
    }

    /// Moves a piece to a specified square. Does not check if the move is legal!
    /// It does check beforehand if there is a piece present
    pub fn make_move(&mut self, move_description: &Move) {
        if let Some(piece) = self.piece_at(move_description.start_square) {
            if move_description.get_flag() == MoveFlag::CastleKingside
                || move_description.get_flag() == MoveFlag::CastleQueenside
            {
                if move_description.get_flag() == MoveFlag::CastleKingside {
                    self.castling_rights.lose(true, self.white_turn);

                    self.remove_piece(move_description.start_square);
                    self.remove_piece(move_description.start_square + 3);
                    self.add_piece(piece, move_description.target_square);
                    self.add_piece(
                        Piece::new(PieceType::Rook, self.white_turn),
                        move_description.start_square + 1,
                    );
                } else {
                    self.castling_rights.lose(false, self.white_turn);

                    self.remove_piece(move_description.start_square);
                    self.remove_piece(move_description.start_square - 4);
                    self.add_piece(piece, move_description.target_square);
                    self.add_piece(
                        Piece::new(PieceType::Rook, self.white_turn),
                        move_description.start_square - 1,
                    );
                }

                self.white_turn = !self.white_turn;
                return;
            }
            if self
                .en_passant
                .is_some_and(|sq| sq == move_description.target_square)
            {
                self.remove_piece(
                    move_description.target_square - 8 * (self.white_turn as u8 * 2 - 1),
                );
            } else {
                self.remove_piece(move_description.target_square);
            }

            self.remove_piece(move_description.start_square);

            if let Some(new_piece) = move_description.promotion {
                self.add_piece(
                    Piece::new(new_piece, self.white_turn),
                    move_description.target_square,
                );
            } else {
                self.add_piece(piece, move_description.target_square);
            }

            // White kingside rook moved or captured
            if move_description.start_square == 0 || move_description.target_square == 0 {
                self.castling_rights.lose_king_side(true);
            } else if move_description.start_square == 56 || move_description.target_square == 56 {
                self.castling_rights.lose_king_side(false);
            } else if move_description.start_square == 7 || move_description.target_square == 7 {
                self.castling_rights.lose_queen_side(true);
            } else if move_description.start_square == 63 || move_description.target_square == 63 {
                self.castling_rights.lose_queen_side(false);
            }

            self.white_turn = !self.white_turn;
        }
    }

    pub fn restore(&mut self, board: Board) {
        self.bitboards = board.bitboards;
        self.castling_rights = board.castling_rights;
        self.en_passant = board.en_passant;
        self.pieces = board.pieces;
        self.white_turn = board.white_turn;
    }

    // pub fn undo_move(&mut self, move_description: &Move) {
    //     if let Some(piece) = self.piece_at(move_description.target_square) {
    //         if let Some(castle) = move_description.flag
    //             && (castle == MoveFlag::CastleKingside || castle == MoveFlag::CastleQueenside)
    //         {
    //             // Restore rights
    //             self.castling_rights.uncastle(!self.white_turn);

    //             if castle == MoveFlag::CastleKingside {
    //                 self.castling_rights.gain(true, !self.white_turn);

    //                 self.remove_piece(move_description.start_square + 1); // Rook
    //                 self.remove_piece(move_description.target_square); // King
    //                 self.add_piece(piece, move_description.start_square);
    //                 self.add_piece(
    //                     Piece::new(PieceType::Rook, !self.white_turn),
    //                     move_description.start_square + 3,
    //                 );
    //             } else {
    //                 self.castling_rights.gain(false, !self.white_turn);

    //                 self.remove_piece(move_description.start_square - 1); // Rook
    //                 self.remove_piece(move_description.target_square); // King
    //                 self.add_piece(piece, move_description.start_square);
    //                 self.add_piece(
    //                     Piece::new(PieceType::Rook, !self.white_turn),
    //                     move_description.start_square - 4,
    //                 );
    //             }

    //             self.white_turn = !self.white_turn;
    //             return;
    //         }
    //         self.remove_piece(move_description.target_square);

    //         if move_description.promotion.is_some() {
    //             self.add_piece(
    //                 Piece::new(PieceType::Pawn, !self.white_turn),
    //                 move_description.start_square,
    //             );
    //         } else {
    //             self.add_piece(piece, move_description.start_square);
    //         }

    //         if let Some(capture) = move_description.capture {
    //             self.add_piece(
    //                 Piece::new(capture, self.white_turn),
    //                 move_description.target_square
    //                     - (move_description.flag.map_or(0, |_| 1u8)
    //                         * 8
    //                         * (self.white_turn as u8 * 2 - 1)),
    //             );
    //         }

    //         self.white_turn = !self.white_turn;
    //     }
    // }

    pub fn is_white_turn(&self) -> bool {
        self.white_turn
    }

    pub fn is_game_over(&self) -> bool {
        // self.legal_moves().len() == 0
        false
    }

    pub fn piece_at(&self, pos: u8) -> Option<Piece> {
        self.pieces[pos as usize]
    }
}
