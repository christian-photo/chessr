use crate::{
    board::{Board, PieceType},
    moves::{Move, generator::MoveFlag, sliding::SlidingAttackLookup},
    pregen::{BISHOP_ATTACK_MASK, ROOK_ATTACK_MASK},
};

impl Move {
    pub fn legal(&self, board: &Board, lookup: &SlidingAttackLookup) -> bool {
        let side = if board.is_white_turn() { 0 } else { 1 };

        if self.get_flag() == MoveFlag::CastleKingside {
            let mask = 0b1110000 << (56 * side);
            return !board.is_attacked(mask, !board.is_white_turn(), lookup);
        } else if self.get_flag() == MoveFlag::CastleQueenside {
            let mask = 0b11100 << (56 * side);
            return !board.is_attacked(mask, !board.is_white_turn(), lookup);
        }

        if self.get_piece() == PieceType::King {
            return !board.is_attacked(0b1 << self.target_square, !board.is_white_turn(), lookup);
        }

        let king_bitboard = board.bitboards[5 + side * 6 as usize].get_u64();

        // Is the king in check?
        if board.is_attacked(king_bitboard, !board.is_white_turn(), lookup) {
            let mut new_board = board.clone();
            new_board.make_move(&self);

            // We can reuse the old bitboard, because we know that its not the king that is moving
            return !new_board.is_attacked(king_bitboard, !board.is_white_turn(), lookup);
        }

        let king_sq = king_bitboard.trailing_zeros() as u8;

        // By pretending that the king is a queen, we can easily check if the king and the moving piece share a common ray
        if ROOK_ATTACK_MASK[king_sq as usize] & (0b1 << self.start_square) != 0
            || BISHOP_ATTACK_MASK[king_sq as usize] & (0b1 << self.start_square) != 0
        {
            let mut new_board = board.clone();
            new_board.make_move(&self);

            return !new_board.is_attacked(king_bitboard, !board.is_white_turn(), lookup);
        }

        true
    }
}
