use crate::{board::Board, piece::PieceType};
use std::mem;

pub struct Move {
    pub start_square: u8,
    pub target_square: u8,

    pub is_castle: bool,
    pub en_passant: bool,

    /// The capture should be populated ON MOVE for performance
    pub capture: Option<PieceType>,
    pub promotion: Option<PieceType>,
}

#[rustfmt::skip]
const SQUARES_TO_EDGE: [u8; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 1, 1, 1, 1, 1, 1, 0,
    0, 1, 2, 2, 2, 2, 1, 0,
    0, 1, 2, 3, 3, 2, 1, 0,
    0, 1, 2, 3, 3, 2, 1, 0,
    0, 1, 2, 2, 2, 2, 1, 0,
    0, 1, 1, 1, 1, 1, 1, 0,
    0, 0, 0, 0, 0, 0, 0, 0,
];

const KNIGHT_ATTACK: [u64; 64] = [
    132096,
    329728,
    659712,
    1319424,
    2638848,
    5277696,
    10489856,
    4202496,
    33816576,
    84410368,
    168886273,
    337772546,
    675545092,
    1351090184,
    2685403152,
    1075839008,
    8657043458,
    21609054213,
    43234885898,
    86469771796,
    172939543592,
    345879087184,
    687463207072,
    275414786112,
    2216203125248,
    5531917878528,
    11068130789888,
    22136261579776,
    44272523159552,
    88545046319104,
    175990581010432,
    70506185244672,
    567348000063488,
    1416170976903168,
    2833441482211328,
    5666882964422656,
    11333765928845312,
    22667531857690624,
    45053588738670592,
    18049583422636032,
    145241088016252928,
    362539770087211008,
    725361019446099968,
    1450722038892199936,
    2901444077784399872,
    5802888155568799744,
    11533718717099671552,
    4620693356194824192,
    288230384741646336,
    576460773778259968,
    1224980241106075648,
    2449960482212151296,
    4899920964424302592,
    9799841928848605184,
    1152939783987658752,
    2305878468463689728,
    2199023255552,
    5497558138880,
    292470092988416,
    584940185976832,
    1169880371953664,
    2339760743907328,
    4679521487814656,
    9077567998918656,
];

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
    // pub fn to_algebraic(&self, board: &Board, piece: &Piece) -> String {
    //     fn pos_to_algebraic(pos: u8) -> String {
    //         let file = pos % 8;

    //         format!("{}{}", rank_char(pos), file + 1)
    //     }

    //     fn rank_char(pos: u8) -> char {
    //         let rank = pos / 8;

    //         (('a' as u8) + rank) as char
    //     }

    //     if self.capture.is_none() {
    //         return match piece.piece_type() {
    //             PieceType::Pawn => pos_to_algebraic(self.target_square),
    //             PieceType::Knight => format!("N{}", pos_to_algebraic(self.target_square)),
    //             PieceType::Bishop => format!("B{}", pos_to_algebraic(self.target_square)),
    //             PieceType::Rook => format!("R{}", pos_to_algebraic(self.target_square)),
    //             PieceType::Queen => format!("Q{}", pos_to_algebraic(self.target_square)),
    //             PieceType::King => format!("K{}", pos_to_algebraic(self.target_square)),
    //         };
    //     }

    //     ""
    // }
}

// Move generation
impl Move {
    pub fn generate_legal_moves(board: &mut Board) -> Vec<Move> {
        let mut moves = Move::pawn_moves(board);
        moves.append(&mut Move::knight_moves(board));
        moves.append(&mut Move::bishop_moves(board));
        moves.append(&mut Move::rook_moves(board));
        moves.append(&mut Move::queen_moves(board));
        moves.append(&mut Move::king_moves(board));

        moves
    }

    pub fn pawn_moves(board: &mut Board) -> Vec<Move> {
        // Moves that involve promotions, should generate a new move for each possible promotion
        fn promote_if_possible(
            board: &Board,
            pos: u8,
            target: u8,
            capture: Option<PieceType>,
            moves: &mut Vec<Move>,
        ) -> bool {
            if board.is_white_turn() && pos / 8 == 6 || !board.is_white_turn() && pos / 8 == 1 {
                moves.push(Move {
                    capture,
                    en_passant: false,
                    is_castle: false,
                    promotion: Some(PieceType::Queen),
                    start_square: pos,
                    target_square: target,
                });

                moves.push(Move {
                    capture,
                    en_passant: false,
                    is_castle: false,
                    promotion: Some(PieceType::Rook),
                    start_square: pos,
                    target_square: target,
                });

                moves.push(Move {
                    capture,
                    en_passant: false,
                    is_castle: false,
                    promotion: Some(PieceType::Bishop),
                    start_square: pos,
                    target_square: target,
                });

                moves.push(Move {
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
        let direction = -1 + (board.is_white_turn() as i8 * 2);

        let mut opponent = board.black.get_u64();
        let mut friendly = board.white.get_u64();

        if !board.is_white_turn() {
            mem::swap(&mut opponent, &mut friendly);
        }

        let mut moves = Vec::<Move>::new();

        while pawns != 0 {
            let mut pawn_moves = Vec::<Move>::new();
            let pos = Move::pop_lsb(&mut pawns);

            let piece_infront = (opponent | friendly) & (0b1 << pos as i8 + 8 * direction) != 0;

            let piece_two_infront =
                (opponent | friendly) & (0b1 << pos as i8 + 16 * direction) != 0;

            let can_capture_left = (opponent & (0b1 << pos as i8 + 8 * direction - 1)) != 0
                && SQUARES_TO_EDGE[pos as usize] > 0;
            let can_capture_right = (opponent & (0b1 << pos as i8 + 8 * direction + 1)) != 0
                && SQUARES_TO_EDGE[pos as usize] > 0;

            if !piece_infront {
                // Check if the piece will be promoted
                let target = (pos as i8 + 8 * direction) as u8;
                if !promote_if_possible(board, pos, target, None, &mut moves) {
                    pawn_moves.push(Move {
                        capture: None,
                        en_passant: false,
                        is_castle: false,
                        promotion: None,
                        start_square: pos,
                        target_square: target as u8,
                    });
                }
            }

            if can_capture_left {
                let target = (pos as i8 + 8 * direction - 1) as u8;
                let capture = board.piece_at(target).unwrap().piece_type(); // We can unwrap because we checked that there is actually a opponent piece. If this still is None, we do not update this array properly

                if !promote_if_possible(board, pos, target as u8, Some(capture), &mut moves) {
                    pawn_moves.push(Move {
                        capture: Some(capture),
                        en_passant: false,
                        is_castle: false,
                        promotion: None,
                        start_square: pos,
                        target_square: target,
                    });
                }
            }

            if can_capture_right {
                let target = (pos as i8 + 8 * direction + 1) as u8;
                let capture = board.piece_at(target).unwrap().piece_type(); // We can unwrap because we checked that there is actually a opponent piece. If this still is None, we do not update this array properly

                if !promote_if_possible(board, pos, target as u8, Some(capture), &mut moves) {
                    pawn_moves.push(Move {
                        capture: Some(capture),
                        en_passant: false,
                        is_castle: false,
                        promotion: None,
                        start_square: pos,
                        target_square: target,
                    });
                }
            }

            if let Some(en_passant) = board.en_passant {
                if en_passant == (pos as i8 + 8 * direction + 1) as u8
                    || en_passant == (pos as i8 + 8 * direction - 1) as u8
                {
                    pawn_moves.push(Move {
                        capture: Some(PieceType::Pawn),
                        en_passant: true,
                        is_castle: false,
                        promotion: None,
                        start_square: pos,
                        target_square: en_passant,
                    });
                }
            }

            // These pawns can move two steps, because it is their first move
            if !piece_infront
                && !piece_two_infront
                && (board.is_white_turn() && pos / 8 == 1 || !board.is_white_turn() && pos / 8 == 6)
            {
                let move_desc = Move {
                    capture: None,
                    en_passant: false,
                    is_castle: false,
                    promotion: None,
                    start_square: pos,
                    target_square: (pos as i8 + 16 * direction) as u8,
                };

                pawn_moves.push(move_desc);
            }

            for move_desc in pawn_moves {
                if Move::check_move_legal(board, &move_desc) {
                    moves.push(move_desc);
                }
            }
        }

        moves
    }

    pub fn knight_moves(board: &mut Board) -> Vec<Move> {
        let mut knights = board.bitboards[1 + (!board.is_white_turn() as usize) * 6].get_u64();

        let mut moves = Vec::<Move>::new();

        let friendly = if board.is_white_turn() {
            board.white.get_u64()
        } else {
            board.black.get_u64()
        };

        while knights != 0 {
            let pos = Move::pop_lsb(&mut knights);
            let mut attack_map = KNIGHT_ATTACK[pos as usize];
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

                // Can't capture pieces of the same color
                if (friendly & (0b1 << target)) != 0 {
                    continue;
                }

                found_move.capture = board.pieces[target as usize].map(|p| p.piece_type());

                if Move::check_move_legal(board, &found_move) {
                    moves.push(found_move);
                }
            }
        }

        moves
    }

    pub fn bishop_moves(board: &mut Board) -> Vec<Move> {
        vec![]
    }

    pub fn rook_moves(board: &mut Board) -> Vec<Move> {
        vec![]
    }

    pub fn queen_moves(board: &mut Board) -> Vec<Move> {
        vec![]
    }

    pub fn king_moves(board: &mut Board) -> Vec<Move> {
        vec![]
    }

    fn check_move_legal(board: &mut Board, move_desc: &Move) -> bool {
        // To filter moves that would leave the king in check, we only need to look at
        // sliding pieces (Bishop, Rook, Queen), because pins only occur along rays.
        // A knight attacks by jumping - it cannot be blocked, so no pin is possible.
        // A pawn attacks only one square diagonally - it has no ray, so no pin is possible.
        board.make_move(move_desc);

        let king_pos = board.bitboards[5 + (board.is_white_turn() as usize) * 6]
            .get_u64()
            .trailing_zeros() as u8;

        let mut moves = Move::bishop_moves(board);
        for bishop_move in moves {
            if bishop_move.target_square == king_pos {
                board.undo_move(move_desc);
                return false;
            }
        }

        moves = Move::rook_moves(board);
        for rook_move in moves {
            if rook_move.target_square == king_pos {
                board.undo_move(move_desc);
                return false;
            }
        }

        moves = Move::queen_moves(board);
        for queen_move in moves {
            if queen_move.target_square == king_pos {
                board.undo_move(move_desc);
                return false;
            }
        }

        board.undo_move(move_desc);
        return true;
    }

    fn pop_lsb(board: &mut u64) -> u8 {
        let lsb = board.trailing_zeros() as u8;
        *board &= *board - 1;
        lsb
    }
}
