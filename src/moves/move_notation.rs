use crate::{
    board::{Board, Piece, PieceType},
    moves::{Move, MoveList, generator::MoveFlag, sliding::SlidingAttackLookup},
};

impl Move {
    /// Not optimized for performance, but doesn't need to be performant
    /// Because it is (compared to the move generation itself) rarely used
    pub fn to_algebraic(
        &self,
        board: &Board,
        piece: &Piece,
        lookup: &SlidingAttackLookup,
    ) -> String {
        fn pos_to_algebraic(pos: u8) -> String {
            let rank = pos / 8;

            format!("{}{}", file_char(pos), rank + 1)
        }

        fn file_char(pos: u8) -> char {
            let file = pos % 8;

            (('a' as u8) + file) as char
        }

        fn disambiguation(
            start: u8,
            target: u8,
            board: &Board,
            piece: &Piece,
            lookup: &SlidingAttackLookup,
        ) -> String {
            if piece.piece_type() == PieceType::Pawn || piece.piece_type() == PieceType::King {
                return "".to_string();
            }

            let mut moves = MoveList::new();

            match piece.piece_type() {
                PieceType::Pawn => (),
                PieceType::Knight => Move::knight_moves(&mut board.clone(), &mut moves),
                PieceType::Bishop => Move::bishop_moves(&mut board.clone(), &mut moves, lookup),
                PieceType::Rook => Move::rook_moves(&mut board.clone(), &mut moves, lookup),
                PieceType::Queen => Move::queen_moves(&mut board.clone(), &mut moves, lookup),
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

        if self.get_flag() == MoveFlag::CastleKingside
            || self.get_flag() == MoveFlag::CastleQueenside
        {
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
                disambiguation(self.start_square, self.target_square, board, piece, lookup),
                capture,
                pos_to_algebraic(self.target_square)
            ),
            PieceType::Bishop => format!(
                "B{}{}{}",
                disambiguation(self.start_square, self.target_square, board, piece, lookup),
                capture,
                pos_to_algebraic(self.target_square)
            ),
            PieceType::Rook => format!(
                "R{}{}{}",
                disambiguation(self.start_square, self.target_square, board, piece, lookup),
                capture,
                pos_to_algebraic(self.target_square)
            ),
            PieceType::Queen => format!(
                "Q{}{}{}",
                disambiguation(self.start_square, self.target_square, board, piece, lookup),
                capture,
                pos_to_algebraic(self.target_square)
            ),
            PieceType::King => format!(
                "K{}{}{}",
                disambiguation(self.start_square, self.target_square, board, piece, lookup),
                capture,
                pos_to_algebraic(self.target_square)
            ),
        };
    }

    pub fn to_uci_move(&self) -> String {
        fn pos_to_algebraic(pos: u8) -> String {
            let rank = pos / 8;

            format!("{}{}", file_char(pos), rank + 1)
        }

        fn file_char(pos: u8) -> char {
            let file = pos % 8;

            (('a' as u8) + file) as char
        }

        format!(
            "{}{}{}",
            pos_to_algebraic(self.start_square),
            pos_to_algebraic(self.target_square),
            self.promotion
                .map_or("".to_string(), |prom| prom.abbreviation().to_string())
        )
    }

    pub fn from_uci_move(notation: &str, board: &Board) -> Result<Move, String> {
        let start = notation.chars().take(2).collect::<String>();
        let target = notation.chars().skip(2).take(2).collect::<String>();

        let start_pos = Board::algebraic_to_u8(&start);
        let target_pos = Board::algebraic_to_u8(&target);

        let piece = board.piece_at(start_pos).ok_or(format!(
            "Start square was {} but no piece was found on that square",
            start
        ))?;

        let diff = start_pos.abs_diff(target_pos);

        let mut flag = MoveFlag::None;
        let mut capture = None;
        let mut promotion = None;

        if piece.piece_type() == PieceType::Pawn && (diff == 7 || diff == 9) {
            flag = MoveFlag::EnPassant;
            capture = Some(PieceType::Pawn);
        } else if piece.piece_type() == PieceType::Pawn && diff == 16 {
            flag = MoveFlag::DoublePawnPush;
        } else if piece.piece_type() == PieceType::Pawn && (target_pos >= 56 || target_pos <= 7) {
            promotion =
                PieceType::from_abbreviation(notation.chars().skip(4).next().unwrap_or(' ')); // Silently fails if the conversion is not successful
            flag = MoveFlag::Promotion;
        } else if piece.piece_type() == PieceType::King && diff == 2 {
            if start_pos.saturating_sub(target_pos) == 2 {
                flag = MoveFlag::CastleQueenside;
            } else {
                flag = MoveFlag::CastleKingside;
            }
        }

        Ok(Move::new(
            start_pos,
            target_pos,
            flag,
            piece.piece_type(),
            capture,
            promotion,
        ))
    }
}
