use crate::{
    bit_ops::pop_lsb,
    board::{BoardState, PieceType},
    search::bonus_maps::*,
};

/// Evaluates a given board position
pub fn evaluate(board: &BoardState) -> i32 {
    let mut eval = 0i32;
    eval += count_material(board, true) - count_material(board, false);
    eval += apply_bonus_maps(board, true) - apply_bonus_maps(board, false);

    if board.is_white_turn() { eval } else { -eval }
}

fn count_material(board: &BoardState, white: bool) -> i32 {
    let offset = if white { 0 } else { 6usize };

    let pawns = board.bitboards[offset].get_u64().count_ones() as i32 * PieceType::Pawn.get_value();
    let knights =
        board.bitboards[1 + offset].get_u64().count_ones() as i32 * PieceType::Knight.get_value();
    let bishops =
        board.bitboards[2 + offset].get_u64().count_ones() as i32 * PieceType::Bishop.get_value();
    let rooks =
        board.bitboards[3 + offset].get_u64().count_ones() as i32 * PieceType::Rook.get_value();
    let queens =
        board.bitboards[4 + offset].get_u64().count_ones() as i32 * PieceType::Queen.get_value();

    pawns + knights + bishops + rooks + queens
}

fn apply_bonus_maps(board: &BoardState, white: bool) -> i32 {
    let offset = if white { 0 } else { 6usize };

    let mut bonus = 0;

    let mut pawns = board.bitboards[offset].get_u64();

    if white {
        while pawns != 0 {
            let pos = pop_lsb(&mut pawns);

            bonus += PAWN_BONUS_MAP[pos as usize];
        }
    } else {
        while pawns != 0 {
            let pos = 64 - pop_lsb(&mut pawns);

            bonus += PAWN_BONUS_MAP[pos as usize];
        }
    }

    let mut knights = board.bitboards[1 + offset].get_u64();
    while knights != 0 {
        let pos = pop_lsb(&mut knights);

        bonus += KNIGHT_BONUS_MAP[pos as usize];
    }

    let mut bishops = board.bitboards[2 + offset].get_u64();
    while bishops != 0 {
        let pos = pop_lsb(&mut bishops);

        bonus += BISHOP_BONUS_MAP[pos as usize];
    }

    bonus
}
