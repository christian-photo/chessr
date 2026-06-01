use crate::{
    board::*,
    pregen::{BLACK_PAWN_ATTACK, KNIGHT_ATTACK, WHITE_PAWN_ATTACK},
};

pub struct MoveList {
    moves: [Move; 218],
    len: usize,
}

impl MoveList {
    pub fn new() -> Self {
        MoveList {
            moves: [Move::default(); 218],
            len: 0,
        }
    }

    pub fn push(&mut self, m: Move) {
        self.moves[self.len] = m;
        self.len += 1;
    }

    pub fn push_if_legal(&mut self, m: Move, board: &Board) {
        if m.is_legal(board) {
            self.push(m);
        }
    }

    pub fn iter(&self) -> &[Move] {
        &self.moves[..self.len]
    }

    pub fn reset(&mut self) {
        self.len = 0;
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Move {
    pub start_square: u8,
    pub target_square: u8,

    pub is_castle: bool,
    pub en_passant: bool,

    pub capture: Option<PieceType>,
    pub promotion: Option<PieceType>,
}

// Constructors
impl Move {
    pub fn capture(
        from: u8,
        to: u8,
        en_passant: bool,
        captured_piece: PieceType,
        promotion: Option<PieceType>,
    ) -> Move {
        Move {
            capture: Some(captured_piece),
            is_castle: false,
            en_passant,
            promotion,
            start_square: from,
            target_square: to,
        }
    }

    pub fn simple(from: u8, to: u8) -> Move {
        Move {
            capture: None,
            is_castle: false,
            en_passant: false,
            promotion: None,
            start_square: from,
            target_square: to,
        }
    }
}

// Move methods
impl Move {
    /// Not optimized for performance, but doesn't need to be performant
    /// Because it is (compared to the move generation itself) rarely used
    pub fn to_algebraic(&self, board: &Board, piece: &Piece) -> String {
        fn pos_to_algebraic(pos: u8) -> String {
            let rank = pos / 8;

            format!("{}{}", file_char(pos), rank + 1)
        }

        fn file_char(pos: u8) -> char {
            let file = pos % 8;

            (('a' as u8) + file) as char
        }

        fn disambiguation(start: u8, target: u8, board: &Board, piece: &Piece) -> String {
            if piece.piece_type() == PieceType::Pawn || piece.piece_type() == PieceType::King {
                return "".to_string();
            }

            let mut moves = MoveList::new();

            match piece.piece_type() {
                PieceType::Pawn => (),
                PieceType::Knight => Move::knight_moves(&mut board.clone(), &mut moves),
                PieceType::Bishop => Move::bishop_moves(&mut board.clone(), &mut moves),
                PieceType::Rook => Move::rook_moves(&mut board.clone(), &mut moves),
                PieceType::Queen => Move::queen_moves(&mut board.clone(), &mut moves),
                PieceType::King => (),
            };

            let mut moves_same_target = vec![];

            for move_desc in moves.iter() {
                if move_desc.target_square == target && move_desc.start_square != start {
                    moves_same_target.push(move_desc);
                }
            }

            if moves_same_target.len() == 0 {
                return "".to_string();
            } else if moves_same_target.len() == 1 {
                // Find out if the two pieces are on the same file or rank, so we know which one we CAN'T use to disambiguate
                if start / 8 == moves_same_target[0].start_square / 8 {
                    return (start % 8).to_string();
                }
                return file_char(start).to_string();
            } else {
                return pos_to_algebraic(start);
            }
        }

        if self.is_castle {
            if self.target_square == 6 || self.target_square == 62 {
                return "O-O".to_string();
            } else {
                return "O-O-O".to_string();
            }
        }

        let capture = match self.capture {
            Some(_) => "x",
            None => "",
        };

        let pawn_file = if self.capture.is_some() {
            file_char(self.start_square).to_string()
        } else {
            "".to_string()
        };

        let promotion = match self.promotion {
            Some(prom) => format!("={}", prom.abbreviation().to_ascii_uppercase()),
            None => "".to_string(),
        };

        // TODO: Mate / Checkmate

        return match piece.piece_type() {
            PieceType::Pawn => format!(
                "{}{}{}{}",
                pawn_file,
                capture,
                pos_to_algebraic(self.target_square),
                promotion
            ),
            PieceType::Knight => format!(
                "N{}{}{}",
                disambiguation(self.start_square, self.target_square, board, piece),
                capture,
                pos_to_algebraic(self.target_square)
            ),
            PieceType::Bishop => format!(
                "B{}{}{}",
                disambiguation(self.start_square, self.target_square, board, piece),
                capture,
                pos_to_algebraic(self.target_square)
            ),
            PieceType::Rook => format!(
                "R{}{}{}",
                disambiguation(self.start_square, self.target_square, board, piece),
                capture,
                pos_to_algebraic(self.target_square)
            ),
            PieceType::Queen => format!(
                "Q{}{}{}",
                disambiguation(self.start_square, self.target_square, board, piece),
                capture,
                pos_to_algebraic(self.target_square)
            ),
            PieceType::King => format!(
                "K{}{}{}",
                disambiguation(self.start_square, self.target_square, board, piece),
                capture,
                pos_to_algebraic(self.target_square)
            ),
        };
    }

    pub fn is_legal(&self, board: &Board) -> bool {
        let is_king = board.bitboards[5 + (!board.is_white_turn() as usize) * 6]
            .has_piece_at(self.start_square);

        if !is_king {}

        return true;
    }
}

// Move generation
impl Move {
    pub fn generate_legal_moves(board: &mut Board) -> MoveList {
        let mut moves = MoveList::new();

        Move::pawn_moves(board, &mut moves);
        Move::knight_moves(board, &mut moves);
        Move::bishop_moves(board, &mut moves);
        Move::rook_moves(board, &mut moves);
        Move::queen_moves(board, &mut moves);
        Move::king_moves(board, &mut moves);

        moves
    }

    pub fn pawn_moves(board: &mut Board, move_list: &mut MoveList) {
        // Moves that involve promotions, should generate a new move for each possible promotion
        fn promote_if_possible(
            board: &Board,
            pos: u8,
            target: u8,
            capture: Option<PieceType>,
            move_list: &mut MoveList,
        ) -> bool {
            if board.is_white_turn() && target >= 56 || !board.is_white_turn() && target <= 7 {
                let queen_prom = Move {
                    capture,
                    en_passant: false,
                    is_castle: false,
                    promotion: Some(PieceType::Queen),
                    start_square: pos,
                    target_square: target,
                };
                // We only need to check legality for one move/promotion because the promoted piece type can not influence legality and the move is otherwise the same
                if !queen_prom.is_legal(board) {
                    return false;
                }

                move_list.push(queen_prom);

                move_list.push(Move {
                    capture,
                    en_passant: false,
                    is_castle: false,
                    promotion: Some(PieceType::Rook),
                    start_square: pos,
                    target_square: target,
                });

                move_list.push(Move {
                    capture,
                    en_passant: false,
                    is_castle: false,
                    promotion: Some(PieceType::Bishop),
                    start_square: pos,
                    target_square: target,
                });

                move_list.push(Move {
                    capture,
                    en_passant: false,
                    is_castle: false,
                    promotion: Some(PieceType::Knight),
                    start_square: pos,
                    target_square: target,
                });

                return true;
            } else {
                return false;
            }
        }

        let mut pawns = board.bitboards[0 + (!board.is_white_turn() as usize) * 6].get_u64();
        let white_pieces = board.bitboards[12].get_u64();
        let black_pieces = board.bitboards[13].get_u64();
        let pieces = black_pieces | white_pieces;

        let mut captureable_squares: u64;
        let attack_map: &[u64; 64];

        if !board.is_white_turn() {
            captureable_squares = white_pieces;
            attack_map = &BLACK_PAWN_ATTACK;

            let mut single_advance = (pieces | (pawns >> 8)) & !pieces;
            let mut double_advance = (pieces | ((single_advance & (0b11111111 << 24) ) >> 8)) // only the pawns that can move a single step might be able to move two, so shift these to the target squares
                & !pieces; // Double advance is only possible on the first move, so we mask the sixth rank

            while single_advance != 0 {
                let target = Move::pop_lsb(&mut single_advance);
                let pos = target + 8;
                if !promote_if_possible(board, pos, target, None, move_list) {
                    move_list.push_if_legal(
                        Move {
                            capture: None,
                            en_passant: false,
                            is_castle: false,
                            promotion: None,
                            start_square: pos,
                            target_square: target,
                        },
                        board,
                    );
                }
            }

            while double_advance != 0 {
                let target = Move::pop_lsb(&mut double_advance);
                let pos = target + 16;
                move_list.push_if_legal(
                    Move {
                        capture: None,
                        en_passant: false,
                        is_castle: false,
                        promotion: None,
                        start_square: pos,
                        target_square: target,
                    },
                    board,
                );
            }
        } else {
            captureable_squares = black_pieces;
            attack_map = &WHITE_PAWN_ATTACK;

            let mut single_advance = (pieces | (pawns << 8)) & !pieces;
            let mut double_advance = (pieces | ((single_advance & (0b11111111 << 16)) << 8)) // only the pawns that can move a single step might be able to move two, so shift these to the target squares
                & !pieces; // Double advance is only possible on the first move, so we mask the third rank

            while single_advance != 0 {
                let target = Move::pop_lsb(&mut single_advance);
                let pos = target - 8;
                if !promote_if_possible(board, pos, target, None, move_list) {
                    move_list.push_if_legal(
                        Move {
                            capture: None,
                            en_passant: false,
                            is_castle: false,
                            promotion: None,
                            start_square: pos,
                            target_square: target,
                        },
                        board,
                    );
                }
            }

            while double_advance != 0 {
                let target = Move::pop_lsb(&mut double_advance);
                let pos = target - 16;
                move_list.push_if_legal(
                    Move {
                        capture: None,
                        en_passant: false,
                        is_castle: false,
                        promotion: None,
                        start_square: pos,
                        target_square: target,
                    },
                    board,
                );
            }
        }

        if let Some(en_passant) = board.en_passant {
            captureable_squares |= 0b1 << en_passant; // We can just act like the en passant field holds another piece (pawn)
        }

        while pawns != 0 {
            let pos = Move::pop_lsb(&mut pawns);

            let mut capture_targets = attack_map[pos as usize] & captureable_squares;

            while capture_targets != 0 {
                let target = Move::pop_lsb(&mut capture_targets);
                let capture = board
                    .piece_at(target)
                    .map_or(PieceType::Pawn, |p| p.piece_type());

                if !promote_if_possible(board, pos, target, Some(capture), move_list) {
                    move_list.push_if_legal(
                        Move {
                            capture: Some(capture),
                            en_passant: board.en_passant.is_some_and(|square| square == target),
                            is_castle: false,
                            promotion: None,
                            start_square: pos,
                            target_square: target,
                        },
                        board,
                    );
                }
            }
        }
    }

    pub fn knight_moves(board: &mut Board, move_list: &mut MoveList) {
        let side: usize = if board.is_white_turn() { 0 } else { 1 };

        let mut knights = board.bitboards[1 + side * 6].get_u64();

        let friendly = board.bitboards[12 + side].get_u64();

        while knights != 0 {
            let pos = Move::pop_lsb(&mut knights);
            let mut attack_map = KNIGHT_ATTACK[pos as usize] & !friendly;
            while attack_map != 0 {
                let target = Move::pop_lsb(&mut attack_map);

                let mut found_move = Move {
                    capture: None,
                    is_castle: false,
                    en_passant: false,
                    start_square: pos,
                    target_square: target,
                    promotion: None,
                };

                found_move.capture = board.pieces[target as usize].map(|p| p.piece_type());

                move_list.push_if_legal(found_move, board);
            }
        }
    }

    pub fn bishop_moves(board: &mut Board, move_list: &mut MoveList) {
        let mut bishops = board.bitboards[2 + (!board.is_white_turn() as usize) * 6].get_u64();
    }

    pub fn rook_moves(board: &mut Board, move_list: &mut MoveList) {
        let mut rooks = board.bitboards[3 + (!board.is_white_turn() as usize) * 6].get_u64();
    }

    pub fn queen_moves(board: &mut Board, move_list: &mut MoveList) {
        let mut queens = board.bitboards[4 + (!board.is_white_turn() as usize) * 6].get_u64();
    }

    pub fn king_moves(board: &mut Board, move_list: &mut MoveList) {
        let mut king = board.bitboards[5 + (!board.is_white_turn() as usize) * 6].get_u64();
    }

    fn pop_lsb(board: &mut u64) -> u8 {
        let lsb = board.trailing_zeros() as u8;
        *board &= *board - 1;
        lsb
    }
}
