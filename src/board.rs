use crate::{
    move_generator::Move,
    piece::{Piece, PieceType},
};

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
    /// - WPawn
    /// - WKnight
    /// - WBishop
    /// - WRook
    /// - WQueen
    /// - WKing
    /// - BPawn
    /// - BKnight
    /// - ...
    pub(crate) bitboards: [Bitboard; 14],

    /// Array containing all pieces on the board
    pub(crate) pieces: [Option<Piece>; 64],

    white_turn: bool,
    // castling_rights: ...
    pub(crate) en_passant: Option<u8>,
    half_moves: u8,
    full_moves: u16,
}

impl Board {
    pub fn empty() -> Board {
        Board {
            bitboards: [Bitboard::empty(); 14],
            pieces: [None; 64],
            white_turn: true,
            en_passant: None,
            half_moves: 0,
            full_moves: 0,
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
        // TODO

        let en_passant_target = iter.next().expect("Malformed FEN string");
        if en_passant_target != "-" {}

        Ok(board)
    }

    pub fn algebraic_to_u8(notation: &str) -> u8 {
        let mut chars = notation.chars();

        let rank = chars
            .next()
            .expect("Algebraic notation needs 2 chars")
            .to_ascii_lowercase();
        let file = chars
            .next()
            .expect("Algebraic notation needs 2 chars")
            .to_digit(10)
            .unwrap() as u8;

        file - 1 + 8 * (rank as u8 - 'a' as u8) as u8
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
        // TODO: Castling and promotion
        if let Some(piece) = self.piece_at(move_description.start_square) {
            self.white_turn = !self.white_turn;

            self.remove_piece(move_description.target_square);
            self.remove_piece(move_description.start_square);
            self.add_piece(piece, move_description.target_square);
        }
    }

    pub fn undo_move(&mut self, move_description: &Move) {
        // TODO: Castling and promotion
        if let Some(piece) = self.piece_at(move_description.target_square) {
            self.remove_piece(move_description.target_square);
            self.add_piece(piece, move_description.start_square);

            if let Some(capture) = move_description.capture {
                self.add_piece(
                    Piece::new(capture, self.white_turn),
                    move_description.target_square,
                );
            }

            self.white_turn = !self.white_turn;
        }
    }

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

    pub fn king_checked(&self, white: bool) -> bool {
        todo!();
        false
    }
}
