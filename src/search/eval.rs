use crate::{
    bit_ops::pop_lsb,
    board::{BoardState, PieceType},
    search::bonus_maps::*,
};

/// Evaluates a given board position
pub fn evaluate(board: &BoardState) -> i32 {
    // White wants a high score, black a low score
    let mut eval = 0i32;
    eval += count_material(board, true) - count_material(board, false);
    eval += apply_bonus_maps(board, true) - apply_bonus_maps(board, false);

    if board.bitboards[11 + board.is_white_turn() as usize]
        .get_u64()
        .count_ones()
        < 6
    {
        if eval > 0 {
            // White is attacking (position is better for white)
            eval += endgame_bonus(board, true);
        } else {
            eval -= endgame_bonus(board, false);
        }
    }

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
    let king = board.bitboards[5 + offset].get_u64();

    if white {
        while pawns != 0 {
            let pos = pop_lsb(&mut pawns);

            bonus += PAWN_BONUS_MAP[pos as usize];
        }

        bonus += KING_BONUS_MAP[king.trailing_zeros() as usize]
            / (board.full_moves as i32 - 10).max(1) as i32;
    } else {
        while pawns != 0 {
            let pos = 63 - pop_lsb(&mut pawns);

            bonus += PAWN_BONUS_MAP[pos as usize];
        }
        bonus += KING_BONUS_MAP[63 - king.trailing_zeros() as usize]
            / (board.full_moves as i32 - 10).max(1) as i32; // Reduction should only start after 10 full moves
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

fn endgame_bonus(board: &BoardState, white_attacking: bool) -> i32 {
    let mut endgame_bonus = 0;

    endgame_bonus = endgame_bonus + 10 * force_king_to_corner(board, !white_attacking);

    let distance = king_distance(board);
    endgame_bonus = endgame_bonus + (14 - distance) * 5;

    return endgame_bonus;
}

fn force_king_to_corner(board: &BoardState, white_king: bool) -> i32 {
    let offset = if white_king { 0 } else { 6usize };
    let pos = board.bitboards[5 + offset].get_u64().trailing_zeros();

    let file = (pos % 8) as i32;
    let rank = (pos / 8) as i32;

    let file_dist = (file - 3).max(4 - file); // Distance to center files
    let rank_dist = (rank - 3).max(4 - rank);
    file_dist + rank_dist
}

fn king_distance(board: &BoardState) -> i32 {
    let white_king = board.bitboards[5].get_u64().trailing_zeros() as i32;
    let black_king = board.bitboards[11].get_u64().trailing_zeros() as i32;

    let white_file = white_king % 8;
    let white_rank = white_king / 8;
    let black_file = black_king % 8;
    let black_rank = black_king / 8;

    (white_file - black_file).abs() + (white_rank - black_rank).abs()
}
