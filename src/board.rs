pub use crate::piece::{Piece, PieceType};
use crate::{
    bit_ops::pop_lsb,
    engine::ZobristHash,
    moves::{
        generator::{Move, MoveFlag},
        sliding::SlidingAttackLookup,
    },
    pregen::{BLACK_PAWN_ATTACK, KING_MOVE_MAP, KNIGHT_ATTACK, WHITE_PAWN_ATTACK},
};

#[derive(Debug, Clone, Copy)]
pub struct CastlingRights {
    /// Meaning of individual bits:
    /// - 1st bit indicates white kingside
    /// - 2nd bit indicates white queenside
    /// - 3rd bit indicates black kingside
    /// - 4th bit indicates black queenside
    rights: u8,
}

impl CastlingRights {
    #[inline(always)]
    pub fn king_side(&self, white: bool) -> bool {
        (self.rights & (0b1 << (2 * !white as u8))) != 0
    }

    #[inline(always)]
    pub fn queen_side(&self, white: bool) -> bool {
        (self.rights & (0b01 << (2 * !white as u8 + 1))) != 0
    }

    #[inline(always)]
    pub fn lose_king_side(&mut self, white: bool) {
        self.rights &= !(0b1 << (2 * !white as u8));
    }

    #[inline(always)]
    pub fn lose_queen_side(&mut self, white: bool) {
        self.rights &= !(0b1 << (2 * !white as u8 + 1));
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
        self.rights |= 0b1 << (2 * !white as u8 + 1);
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
pub struct BoardState {
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

    pub hash: ZobristHash,

    /// If en passant is possible, this is Some with the square behind the pawn as its value
    pub en_passant: Option<u8>,

    pub half_moves: u8,
    pub full_moves: u16,
}

impl BoardState {
    pub fn empty() -> BoardState {
        BoardState {
            bitboards: [Bitboard::empty(); 14],
            pieces: [None; 64],
            white_turn: true,
            castling_rights: CastlingRights::none(),
            en_passant: None,
            hash: ZobristHash::new(),
            half_moves: 0,
            full_moves: 0,
        }
    }

    pub fn startpos() -> BoardState {
        Self::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }

    pub fn from_fen(fen: &str) -> Result<BoardState, String> {
        let mut board = BoardState::empty();

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

        if !board.white_turn {
            board.hash.change_side();
        }

        let castling_rights = iter.next().expect("Malformed FEN string");
        for c in castling_rights.chars() {
            if c == '-' {
                break;
            }

            let white = c.is_ascii_uppercase();
            let king_side = c.to_ascii_lowercase() == 'k';
            board.castling_rights.gain(king_side, white);
            board.hash.castling(king_side, white);
        }

        let en_passant_target = iter.next().expect("Malformed FEN string");
        if en_passant_target != "-" {
            let square = BoardState::algebraic_to_u8(en_passant_target);
            if square < 64 {
                board.en_passant = Some(square);
                board.hash.en_passant(square);
            }
        }

        let half_move_string = iter.next().expect("Malformed FEN string");
        if half_move_string != "-" {
            board.half_moves = half_move_string.parse::<u8>().unwrap_or(0);
        }

        let full_move_string = iter.next().expect("Malformed FEN string");
        if full_move_string != "-" {
            board.full_moves = full_move_string.parse::<u16>().unwrap_or(0);
        }

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

        self.hash.update(pos, &piece);
    }

    pub fn remove_piece(&mut self, pos: u8) {
        if let Some(piece) = self.pieces[pos as usize] {
            self.pieces[pos as usize] = None;
            let index = piece.to_bitboard_index();
            self.bitboards[index].remove_piece(pos);

            self.bitboards[12 + !piece.is_white() as usize].remove_piece(pos);

            self.hash.update(pos, &piece);
        } else {
            debug_assert!(
                self.pieces[pos as usize].is_none(),
                "square {pos} is not occupied"
            );
        }
    }

    /// Moves a piece to a specified square. Does not check if the move is legal!
    pub fn make_move(&mut self, move_description: &Move) {
        self.half_moves += 1;

        if !self.is_white_turn() {
            self.full_moves += 1;
        }

        self.hash.change_side();

        match move_description.get_flag() {
            MoveFlag::CastleKingside => {
                self.castling_rights.lose_king_side(self.white_turn);
                self.castling_rights.lose_queen_side(self.white_turn);

                self.remove_piece(move_description.start_square);
                self.remove_piece(move_description.start_square + 3);
                self.add_piece(
                    Piece::new(PieceType::King, self.white_turn),
                    move_description.target_square,
                );
                self.add_piece(
                    Piece::new(PieceType::Rook, self.white_turn),
                    move_description.start_square + 1,
                );

                if let Some(esq) = self.en_passant {
                    self.hash.en_passant(esq);
                    self.en_passant = None;
                }

                self.white_turn = !self.white_turn;
                return;
            }
            MoveFlag::CastleQueenside => {
                self.castling_rights.lose_king_side(self.white_turn);
                self.castling_rights.lose_queen_side(self.white_turn);

                self.remove_piece(move_description.start_square);
                self.remove_piece(move_description.start_square - 4);
                self.add_piece(
                    Piece::new(PieceType::King, self.white_turn),
                    move_description.target_square,
                );
                self.add_piece(
                    Piece::new(PieceType::Rook, self.white_turn),
                    move_description.start_square - 1,
                );

                if let Some(esq) = self.en_passant {
                    self.hash.en_passant(esq);
                    self.en_passant = None;
                }

                self.white_turn = !self.white_turn;
                return;
            }
            MoveFlag::EnPassant => {
                self.remove_piece(if self.white_turn {
                    move_description.target_square - 8
                } else {
                    move_description.target_square + 8
                });
                self.remove_piece(move_description.start_square);
                self.add_piece(
                    Piece::new(PieceType::Pawn, self.white_turn),
                    move_description.target_square,
                );

                if let Some(esq) = self.en_passant {
                    self.hash.en_passant(esq);
                    self.en_passant = None;
                }

                self.white_turn = !self.white_turn;
                return;
            }
            MoveFlag::Promotion => {
                self.remove_piece(move_description.target_square);
                self.remove_piece(move_description.start_square);

                self.add_piece(
                    Piece::new(
                        move_description
                            .promotion
                            .expect("MoveFlag::Promotion was set"),
                        self.white_turn,
                    ),
                    move_description.target_square,
                );
            }
            MoveFlag::DoublePawnPush => {
                self.remove_piece(move_description.target_square);
                self.remove_piece(move_description.start_square);

                self.add_piece(
                    Piece::new(move_description.get_piece(), self.white_turn),
                    move_description.target_square,
                );

                if let Some(esq) = self.en_passant {
                    self.hash.en_passant(esq);
                }

                self.en_passant =
                    Some((move_description.start_square + move_description.target_square) / 2);
                self.hash.en_passant(self.en_passant.unwrap());

                self.white_turn = !self.white_turn;
                return;
            }
            MoveFlag::None => {
                self.remove_piece(move_description.target_square);
                self.remove_piece(move_description.start_square);

                self.add_piece(
                    Piece::new(move_description.get_piece(), self.white_turn),
                    move_description.target_square,
                );
            }
        }

        match move_description.get_piece() {
            PieceType::King => {
                self.castling_rights.lose_king_side(self.white_turn);
                self.castling_rights.lose_queen_side(self.white_turn);

                self.hash.castling(true, self.white_turn);
                self.hash.castling(false, self.white_turn);
            }
            PieceType::Rook => match move_description.start_square {
                0 => {
                    if self.castling_rights.queen_side(true) {
                        self.castling_rights.lose_queen_side(true);
                        self.hash.castling(false, true);
                    }
                }
                7 => {
                    if self.castling_rights.king_side(true) {
                        self.castling_rights.lose_king_side(true);
                        self.hash.castling(true, true);
                    }
                }
                56 => {
                    if self.castling_rights.queen_side(false) {
                        self.castling_rights.lose_queen_side(false);
                        self.hash.castling(false, false);
                    }
                }
                63 => {
                    if self.castling_rights.king_side(false) {
                        self.castling_rights.lose_king_side(false);
                        self.hash.castling(true, false);
                    }
                }
                _ => (),
            },
            _ => (),
        }

        // Lose castling rights when a rook is captured
        match move_description.target_square {
            0 => {
                if self.castling_rights.queen_side(true) {
                    self.castling_rights.lose_queen_side(true);
                    self.hash.castling(false, true);
                }
            }
            7 => {
                if self.castling_rights.king_side(true) {
                    self.castling_rights.lose_king_side(true);
                    self.hash.castling(true, true);
                }
            }
            56 => {
                if self.castling_rights.queen_side(false) {
                    self.castling_rights.lose_queen_side(false);
                    self.hash.castling(false, false);
                }
            }
            63 => {
                if self.castling_rights.king_side(false) {
                    self.castling_rights.lose_king_side(false);
                    self.hash.castling(true, false);
                }
            }
            _ => (),
        }

        if let Some(esq) = self.en_passant {
            self.hash.en_passant(esq);
            self.en_passant = None;
        }
        self.white_turn = !self.white_turn;
    }

    pub fn restore(&mut self, board: BoardState) {
        self.bitboards = board.bitboards;
        self.castling_rights = board.castling_rights;
        self.en_passant = board.en_passant;
        self.pieces = board.pieces;
        self.white_turn = board.white_turn;
    }

    /// Returns true if this position occured three times -> draw
    pub fn check_threefold_repetition(&self) -> bool {
        false
    }

    /// Returns true if the half move limit has been exceeded -> draw
    pub fn check_halfmoves(&self) -> bool {
        self.half_moves >= 100
    }

    pub fn is_white_turn(&self) -> bool {
        self.white_turn
    }

    pub fn piece_at(&self, pos: u8) -> Option<Piece> {
        self.pieces[pos as usize]
    }

    /// Returns true if the opponents attacks any one of the squares in the bitboard
    pub fn is_attacked(
        squares: u64,
        occupancy: u64,
        bitboards: &[Bitboard],
        attacker_white: bool,
        lookup: &SlidingAttackLookup,
    ) -> bool {
        let offset = if attacker_white { 0usize } else { 6 };

        // Queens
        let mut queens = bitboards[4 + offset].get_u64();
        while queens != 0 {
            let pos = pop_lsb(&mut queens);

            if lookup.get_queen_attacks(pos, occupancy) & squares != 0 {
                return true;
            }
        }

        // Rooks
        let mut rooks = bitboards[3 + offset].get_u64();
        while rooks != 0 {
            let pos = pop_lsb(&mut rooks);

            if lookup.get_rook_attacks(pos, occupancy) & squares != 0 {
                return true;
            }
        }

        // Bishops
        let mut bishops = bitboards[2 + offset].get_u64();
        while bishops != 0 {
            let pos = pop_lsb(&mut bishops);

            if lookup.get_bishop_attacks(pos, occupancy) & squares != 0 {
                return true;
            }
        }

        if attacker_white {
            // It is easier to handle pawns completely seperate, because they have different attack maps for black and white
            let mut pawns = bitboards[offset].get_u64();
            while pawns != 0 {
                let pos = pop_lsb(&mut pawns);

                if WHITE_PAWN_ATTACK[pos as usize] & squares != 0 {
                    return true;
                }
            }
        } else {
            let mut pawns = bitboards[offset].get_u64();
            while pawns != 0 {
                let pos = pop_lsb(&mut pawns);

                if BLACK_PAWN_ATTACK[pos as usize] & squares != 0 {
                    return true;
                }
            }
        }

        // Knights
        let mut knights = bitboards[1 + offset].get_u64();
        while knights != 0 {
            let pos = pop_lsb(&mut knights);

            if KNIGHT_ATTACK[pos as usize] & squares != 0 {
                return true;
            }
        }

        // King
        let mut king = bitboards[5 + offset].get_u64();
        let pos = pop_lsb(&mut king);

        if KING_MOVE_MAP[pos as usize] & squares != 0 {
            return true;
        }

        false
    }
}
