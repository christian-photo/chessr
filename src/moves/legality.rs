use crate::{
    board::{BoardState, Piece, PieceType},
    generated::{BISHOP_ATTACK_MASK, ROOK_ATTACK_MASK},
    moves::{Move, generator::MoveFlag, sliding::SlidingAttackLookup},
};

impl Move {
    pub fn legal(
        &self,
        board: &BoardState,
        lookup: &SlidingAttackLookup,
        king_in_check: bool,
    ) -> bool {
        let side = if board.is_white_turn() { 0 } else { 1 };
        let occupancy = board.bitboards[12].get_u64() | board.bitboards[13].get_u64();

        if self.get_flag() == MoveFlag::CastleKingside {
            let mask = 0b1110000 << (56 * side);
            return !BoardState::is_attacked(
                mask,
                occupancy,
                &board.bitboards,
                !board.is_white_turn(),
                lookup,
            );
        } else if self.get_flag() == MoveFlag::CastleQueenside {
            let mask = 0b11100 << (56 * side);
            return !BoardState::is_attacked(
                mask,
                occupancy,
                &board.bitboards,
                !board.is_white_turn(),
                lookup,
            );
        }

        // The king may not move to an attacked square
        if self.get_piece() == PieceType::King {
            return !BoardState::is_attacked(
                0b1 << self.target_square,
                occupancy ^ (0b1 << self.start_square),
                &board.bitboards,
                !board.is_white_turn(),
                lookup,
            );
        }

        let king_bitboard = board.bitboards[5 + side * 6 as usize].get_u64();

        if let Some(capture) = self.capture
            && !(self.get_flag() == MoveFlag::EnPassant)
        {
            let capture_piece = Piece::new(capture, !board.is_white_turn());
            let mut bitboards = board.bitboards.clone();
            bitboards[capture_piece.to_bitboard_index()].remove_piece(self.target_square);
            // We do not need to update the other bitboards because they are our own color. We only need to update the occupancy

            return !BoardState::is_attacked(
                king_bitboard,
                occupancy ^ (0b1 << self.start_square),
                &bitboards,
                !board.is_white_turn(),
                lookup,
            );
        }

        if self.get_flag() == MoveFlag::EnPassant {
            let mut new_board = board.clone();
            new_board.make_move(self);
            let new_occ = new_board.bitboards[12].get_u64() | new_board.bitboards[13].get_u64();
            return !BoardState::is_attacked(
                king_bitboard,
                new_occ,
                &new_board.bitboards,
                !board.is_white_turn(),
                lookup,
            );
        }

        let king_sq = king_bitboard.trailing_zeros() as u8;

        let is_pinned = ROOK_ATTACK_MASK[king_sq as usize] & (0b1 << self.start_square) != 0
            || BISHOP_ATTACK_MASK[king_sq as usize] & (0b1 << self.start_square) != 0;

        if king_in_check || is_pinned {
            let mut new_board = board.clone();
            new_board.make_move(self);
            let new_occupancy =
                new_board.bitboards[12].get_u64() | new_board.bitboards[13].get_u64();
            return !BoardState::is_attacked(
                king_bitboard,
                new_occupancy,
                &new_board.bitboards,
                !board.is_white_turn(),
                lookup,
            );
        }

        true
    }
}
