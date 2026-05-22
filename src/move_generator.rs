use crate::{board::Board, piece::PieceType};

pub struct Move {
    pub start_square: u8,
    pub target_square: u8,

    pub is_castle: bool,
    pub capture: Option<PieceType>,
    pub promotion: Option<PieceType>,
}

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
        captured_piece: PieceType,
        promotion: Option<PieceType>,
    ) -> Move {
        Move {
            capture: Some(captured_piece),
            is_castle: false,
            promotion,
            start_square: from,
            target_square: to,
        }
    }

    pub fn simple(from: u8, to: u8) -> Move {
        Move {
            capture: None,
            is_castle: false,
            promotion: None,
            start_square: from,
            target_square: to,
        }
    }
}

// Move methods
impl Move {
    pub fn to_algebraic(&self, piece_type: PieceType) -> &str {
        fn pos_to_algebraic(pos: u8) -> String {
            let rank = pos / 8;
            let file = pos % 8;

            format!("{}{}", (('a' as u8) + rank) as char, file + 1)
        }

        // if self.capture.is_none() {
        //     return match self.moving_piece {
        //         PieceType::Pawn =>
        //     }
        // }
        //
        ""
    }
}

// Move generation
impl Move {
    pub fn generate_legal_moves(board: &mut Board) -> Vec<Move> {
        let mut moves = Move::generate_possible_moves(board);
        Move::filter_illegal(board, &mut moves);

        moves
    }

    fn generate_possible_moves(board: &mut Board) -> Vec<Move> {
        let mut moves = Move::knight_moves(board);

        moves
    }

    /// Filters the following kinds of illegal moves:
    /// - Moves that leave the king in check
    fn filter_illegal(board: &mut Board, moves: &mut Vec<Move>) {}

    fn castling(board: &mut Board) -> Vec<Move> {
        vec![]
    }

    fn en_passant(board: &mut Board) -> Option<Move> {
        None
    }

    fn knight_moves(board: &mut Board) -> Vec<Move> {
        let mut knights = board.bitboards[1 + (!board.is_white_turn() as usize) * 6].get_u64();

        let mut moves = Vec::<Move>::new();

        while knights != 0 {
            let pos = Move::pop_lsb(&mut knights);
            let mut attack_map = KNIGHT_ATTACK[pos as usize];
            while attack_map != 0 {
                let target = Move::pop_lsb(&mut attack_map);

                let mut found_move = Move {
                    capture: None,
                    is_castle: false,
                    start_square: pos,
                    target_square: target,
                    moving_piece: PieceType::Knight,
                    promotion: None,
                };

                // If there is a piece on the target square, we need to check if it is the same color (reject)
                if let Some(target_piece) = board.pieces[target as usize] {
                    if target_piece.is_white() == board.is_white_turn() {
                        continue;
                    } else {
                        found_move.capture = Some(target_piece.piece_type());
                    }
                }
                moves.push(found_move);
            }
        }

        moves
    }

    fn pawn_moves(board: &mut Board) -> Vec<Move> {
        // Moves that involve promotions, should generate a new move for each possible promotion
        vec![]
    }

    fn pop_lsb(board: &mut u64) -> u8 {
        let lsb = board.trailing_zeros() as u8;
        *board &= *board - 1;
        lsb
    }
}
