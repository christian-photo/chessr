use crate::{
    bit_ops::pop_lsb,
    board::*,
    generated::{BLACK_PAWN_ATTACK, KING_MOVE_MAP, KNIGHT_ATTACK, WHITE_PAWN_ATTACK},
    moves::sliding::SlidingAttackLookup,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum MoveFlag {
    #[default]
    None,
    EnPassant,
    CastleKingside,
    CastleQueenside,
    Promotion,
    DoublePawnPush,
}

#[derive(Clone, Copy)]
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

    #[inline(always)]
    pub fn push(&mut self, m: Move) {
        self.moves[self.len] = m;
        self.len += 1;
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn get(&self, index: usize) -> &Move {
        &self.moves[index]
    }

    pub fn iter(&mut self) -> &mut [Move] {
        &mut self.moves[..self.len]
    }

    pub fn reset(&mut self) {
        self.len = 0;
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Move {
    pub start_square: u8,
    pub target_square: u8,

    /// Includes the move flag in the first 4 bits, the piece type in the last 4 bits
    supplement: u8,

    pub capture: Option<PieceType>,
    pub promotion: Option<PieceType>,
}

impl Move {
    #[inline(always)]
    pub fn new(
        start: u8,
        target: u8,
        flag: MoveFlag,
        piece: PieceType,
        capture: Option<PieceType>,
        promotion: Option<PieceType>,
    ) -> Self {
        Move {
            start_square: start,
            target_square: target,
            supplement: flag as u8 | ((piece as u8) << 4),
            capture,
            promotion,
        }
    }

    #[inline(always)]
    pub fn promotion(
        start: u8,
        target: u8,
        promotion: PieceType,
        capture: Option<PieceType>,
    ) -> Self {
        Move::new(
            start,
            target,
            MoveFlag::Promotion,
            PieceType::Pawn,
            capture,
            Some(promotion),
        )
    }

    #[inline(always)]
    pub fn simple_move(start: u8, target: u8, piece: PieceType) -> Self {
        Move::new(start, target, MoveFlag::None, piece, None, None)
    }

    pub fn get_flag(&self) -> MoveFlag {
        match self.supplement & 0b1111 {
            0 => MoveFlag::None,
            1 => MoveFlag::EnPassant,
            2 => MoveFlag::CastleKingside,
            3 => MoveFlag::CastleQueenside,
            4 => MoveFlag::Promotion,
            5 => MoveFlag::DoublePawnPush,
            _ => MoveFlag::None,
        }
    }

    pub fn get_piece(&self) -> PieceType {
        match self.supplement >> 4 {
            0 => PieceType::Pawn,
            1 => PieceType::Knight,
            2 => PieceType::Bishop,
            3 => PieceType::Rook,
            4 => PieceType::Queen,
            5 => PieceType::King,
            _ => PieceType::Pawn,
        }
    }
}

// Move generation
impl Move {
    /// Generates pseudo legal moves
    pub fn generate_moves(
        move_list: &mut MoveList,
        board: &BoardState,
        lookup: &SlidingAttackLookup,
    ) {
        Move::pawn_moves(board, move_list);
        Move::knight_moves(board, move_list);
        Move::bishop_moves(board, move_list, lookup);
        Move::rook_moves(board, move_list, lookup);
        Move::queen_moves(board, move_list, lookup);
        Move::king_moves(board, move_list);
    }

    pub fn pawn_moves(board: &BoardState, move_list: &mut MoveList) {
        let mut pawns = board.bitboards[0 + (!board.is_white_turn() as usize) * 6].get_u64();
        let white_pieces = board.bitboards[12].get_u64();
        let black_pieces = board.bitboards[13].get_u64();
        let pieces = black_pieces | white_pieces;

        let mut captureable_squares: u64;
        let attack_map: &[u64; 64];

        if board.is_white_turn() {
            captureable_squares = black_pieces;
            attack_map = &WHITE_PAWN_ATTACK;

            let mut single_advance = (pawns << 8) & !pieces;
            let mut promotions = single_advance & (0xFF << 56);
            single_advance = single_advance & !promotions;
            let mut double_advance = ((single_advance & (0xFF << 16)) << 8) // only the pawns that can move a single step might be able to move two, so shift these to the target squares
                & !pieces; // Double advance is only possible on the first move, so we mask the third rank

            while single_advance != 0 {
                let target = pop_lsb(&mut single_advance);
                let pos = target - 8;
                move_list.push(Move::simple_move(pos, target, PieceType::Pawn));
            }

            while promotions != 0 {
                let target = pop_lsb(&mut promotions);
                let pos = target - 8;
                move_list.push(Move::promotion(pos, target, PieceType::Queen, None));
                move_list.push(Move::promotion(pos, target, PieceType::Rook, None));
                move_list.push(Move::promotion(pos, target, PieceType::Bishop, None));
                move_list.push(Move::promotion(pos, target, PieceType::Knight, None));
            }

            while double_advance != 0 {
                let target = pop_lsb(&mut double_advance);
                let pos = target - 16;
                move_list.push(Move::new(
                    pos,
                    target,
                    MoveFlag::DoublePawnPush,
                    PieceType::Pawn,
                    None,
                    None,
                ));
            }
        } else {
            captureable_squares = white_pieces;
            attack_map = &BLACK_PAWN_ATTACK;

            // TODO: Verify
            let mut single_advance = (pawns >> 8) & !pieces;
            let mut promotions = single_advance & 0xFF;
            single_advance = single_advance & !promotions;
            let mut double_advance = ((single_advance & (0xFF << 40)) >> 8) // only the pawns that can move a single step might be able to move two, so shift these to the target squares
                & !pieces; // Double advance is only possible on the first move, so we mask the sixth rank

            while single_advance != 0 {
                let target = pop_lsb(&mut single_advance);
                let pos = target + 8;
                move_list.push(Move::simple_move(pos, target, PieceType::Pawn));
            }

            while promotions != 0 {
                let target = pop_lsb(&mut promotions);
                let pos = target + 8;
                move_list.push(Move::promotion(pos, target, PieceType::Queen, None));
                move_list.push(Move::promotion(pos, target, PieceType::Rook, None));
                move_list.push(Move::promotion(pos, target, PieceType::Bishop, None));
                move_list.push(Move::promotion(pos, target, PieceType::Knight, None));
            }

            while double_advance != 0 {
                let target = pop_lsb(&mut double_advance);
                let pos = target + 16;
                move_list.push(Move::new(
                    pos,
                    target,
                    MoveFlag::DoublePawnPush,
                    PieceType::Pawn,
                    None,
                    None,
                ));
            }
        }

        if let Some(en_passant) = board.en_passant {
            captureable_squares |= 0b1 << en_passant; // We can just act like the en passant field holds another piece (pawn)
        }

        while pawns != 0 {
            let pos = pop_lsb(&mut pawns);

            let mut capture_targets = attack_map[pos as usize] & captureable_squares;

            if capture_targets & (0b11111111 << (56 * board.is_white_turn() as u8)) != 0 {
                while capture_targets != 0 {
                    let target = pop_lsb(&mut capture_targets);
                    let capture = board.piece_at(target).map(|p| p.piece_type());

                    move_list.push(Move::promotion(pos, target, PieceType::Queen, capture));
                    move_list.push(Move::promotion(pos, target, PieceType::Rook, capture));
                    move_list.push(Move::promotion(pos, target, PieceType::Bishop, capture));
                    move_list.push(Move::promotion(pos, target, PieceType::Knight, capture));
                }
            } else {
                while capture_targets != 0 {
                    let target = pop_lsb(&mut capture_targets);
                    let capture = board
                        .piece_at(target)
                        .map_or(PieceType::Pawn, |p| p.piece_type()); // Needed because on the en passant square, there is no piece

                    move_list.push(Move::new(
                        pos,
                        target,
                        board.en_passant.map_or(MoveFlag::None, |sq| {
                            if sq == target {
                                MoveFlag::EnPassant
                            } else {
                                MoveFlag::None
                            }
                        }),
                        PieceType::Pawn,
                        Some(capture),
                        None,
                    ));
                }
            }
        }
    }

    pub fn knight_moves(board: &BoardState, move_list: &mut MoveList) {
        let side: usize = if board.is_white_turn() { 0 } else { 1 };

        let mut knights = board.bitboards[1 + side * 6].get_u64();

        let friendly = board.bitboards[12 + side].get_u64();

        while knights != 0 {
            let pos = pop_lsb(&mut knights);
            let mut attack_map = KNIGHT_ATTACK[pos as usize] & !friendly;
            while attack_map != 0 {
                let target = pop_lsb(&mut attack_map);

                move_list.push(Move::new(
                    pos,
                    target,
                    MoveFlag::None,
                    PieceType::Knight,
                    board.pieces[target as usize].map(|p| p.piece_type()),
                    None,
                ));
            }
        }
    }

    pub fn bishop_moves(
        board: &BoardState,
        move_list: &mut MoveList,
        lookup: &SlidingAttackLookup,
    ) {
        let side: usize = if board.is_white_turn() { 0 } else { 1 };
        let mut bishops = board.bitboards[2 + side * 6].get_u64();
        let friendly = board.bitboards[12 + side].get_u64();

        let occupancy = board.bitboards[12].get_u64() | board.bitboards[13].get_u64();

        while bishops != 0 {
            let pos = pop_lsb(&mut bishops);
            let mut attack_map = lookup.get_bishop_attacks(pos, occupancy) & !friendly;
            while attack_map != 0 {
                let target = pop_lsb(&mut attack_map);

                move_list.push(Move::new(
                    pos,
                    target,
                    MoveFlag::None,
                    PieceType::Bishop,
                    board.pieces[target as usize].map(|p| p.piece_type()),
                    None,
                ));
            }
        }
    }

    pub fn rook_moves(board: &BoardState, move_list: &mut MoveList, lookup: &SlidingAttackLookup) {
        let side: usize = if board.is_white_turn() { 0 } else { 1 };
        let mut rooks = board.bitboards[3 + side * 6].get_u64();
        let friendly = board.bitboards[12 + side].get_u64();

        let occupancy = board.bitboards[12].get_u64() | board.bitboards[13].get_u64();

        while rooks != 0 {
            let pos = pop_lsb(&mut rooks);
            let mut attack_map = lookup.get_rook_attacks(pos, occupancy) & !friendly;
            while attack_map != 0 {
                let target = pop_lsb(&mut attack_map);

                move_list.push(Move::new(
                    pos,
                    target,
                    MoveFlag::None,
                    PieceType::Rook,
                    board.pieces[target as usize].map(|p| p.piece_type()),
                    None,
                ));
            }
        }
    }

    pub fn queen_moves(board: &BoardState, move_list: &mut MoveList, lookup: &SlidingAttackLookup) {
        let side: usize = if board.is_white_turn() { 0 } else { 1 };
        let mut queens = board.bitboards[4 + side * 6].get_u64();
        let friendly = board.bitboards[12 + side].get_u64();

        let occupancy = board.bitboards[12].get_u64() | board.bitboards[13].get_u64();

        while queens != 0 {
            let pos = pop_lsb(&mut queens);
            let mut attack_map = lookup.get_queen_attacks(pos, occupancy) & !friendly;
            while attack_map != 0 {
                let target = pop_lsb(&mut attack_map);

                move_list.push(Move::new(
                    pos,
                    target,
                    MoveFlag::None,
                    PieceType::Queen,
                    board.pieces[target as usize].map(|p| p.piece_type()),
                    None,
                ));
            }
        }
    }

    pub fn king_moves(board: &BoardState, move_list: &mut MoveList) {
        let side: usize = if board.is_white_turn() { 0 } else { 1 };
        let mut king = board.bitboards[5 + side * 6].get_u64();
        let friendly = board.bitboards[12 + side].get_u64();

        let occupancy = board.bitboards[12].get_u64() | board.bitboards[13].get_u64();

        let pos = pop_lsb(&mut king);

        let mut attack_map = KING_MOVE_MAP[pos as usize] & !friendly;
        while attack_map != 0 {
            let target = pop_lsb(&mut attack_map);

            move_list.push(Move::new(
                pos,
                target,
                MoveFlag::None,
                PieceType::King,
                board.pieces[target as usize].map(|p| p.piece_type()),
                None,
            ));
        }

        let kingside_castle_mask = 0b11 << (5 + 56 * side);
        let queenside_castle_mask = 0b111 << (1 + 56 * side);

        if board.castling_rights.king_side(board.is_white_turn())
            && occupancy & kingside_castle_mask == 0
        {
            move_list.push(Move::new(
                pos,
                pos + 2,
                MoveFlag::CastleKingside,
                PieceType::King,
                None,
                None,
            ));
        }
        if board.castling_rights.queen_side(board.is_white_turn())
            && occupancy & queenside_castle_mask == 0
        {
            move_list.push(Move::new(
                pos,
                pos - 2,
                MoveFlag::CastleQueenside,
                PieceType::King,
                None,
                None,
            ));
        }
    }

    // fn magic_hash(masked_board: u64, pos: u8) -> u64 {
    //     (masked_board * MAGICS[pos as usize]) >> MAGIC_SHIFTS[pos as usize]
    // }
}
