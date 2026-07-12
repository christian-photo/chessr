use chessr::{
    board::{Bitboard, BoardState},
    moves::{
        Move,
        sliding::{
            TOTAL_BISHOP_ATTACKS, TOTAL_ROOK_ATTACKS, bishop_array_length, rook_array_length,
        },
    },
    piece::{Piece, PieceType},
    search::ordering::order_moves,
    zobrist::ZobristHash,
};

#[test]
fn bitboard_has_piece_at() {
    let mut bitboard = Bitboard::empty();
    bitboard.add_piece(12);
    bitboard.add_piece(63);
    assert!(bitboard.has_piece_at(12));
    assert!(bitboard.has_piece_at(63));
    assert!(!bitboard.has_piece_at(10));
    bitboard.remove_piece(12);
    assert!(!bitboard.has_piece_at(12));
}

#[test]
fn piece_initialization() {
    let p1 = Piece::new(PieceType::Pawn, true);
    assert_eq!(p1.repr(), 0b00000000);
    assert!(p1.is_white());
    assert_eq!(p1.to_bitboard_index(), 0);

    let p2 = Piece::new(PieceType::Knight, false);
    assert_eq!(p2.repr(), 0b10000001);
    assert!(!p2.is_white());
    assert_eq!(p2.to_bitboard_index(), 7);

    let p3 = Piece::new(PieceType::Bishop, false);
    assert_eq!(p3.repr(), 0b10000010);
    assert_eq!(p3.to_bitboard_index(), 8);
}

#[test]
fn fen_reader() {
    let starting_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let starting_board = BoardState::from_fen(starting_fen).expect("FEN loading failed");
    assert!(starting_board.is_white_turn());
    assert!(starting_board.castling_rights.king_side(true));
    assert!(starting_board.castling_rights.king_side(false));
    assert!(starting_board.castling_rights.queen_side(true));
    assert!(starting_board.castling_rights.queen_side(false));

    let rook = starting_board
        .piece_at(63)
        .expect("Rook must be present in top right corner");
    let king = starting_board
        .piece_at(60)
        .expect("King must be present in specified square");
    assert_eq!(rook.piece_type(), PieceType::Rook);
    assert_eq!(king.piece_type(), PieceType::King);
    assert!(!rook.is_white());

    println!("{}", starting_board.to_ascii());

    let midgame_fen = "r1bk3r/p2pBpNp/n4n2/1p1NP2P/6P1/3P4/P1P1K3/q5b1 b - - 0 1";
    let midgame_board = BoardState::from_fen(midgame_fen).expect("FEN loading failed");

    assert!(!midgame_board.is_white_turn());
    let bishop_mid = midgame_board
        .piece_at(58)
        .expect("Bishop must be present in top row");

    assert_eq!(bishop_mid.piece_type(), PieceType::Bishop);
    println!("{}", midgame_board.to_ascii());
}

#[test]
fn rook_blocker_configurations() {
    assert_eq!(rook_array_length(), TOTAL_ROOK_ATTACKS);
}

#[test]
fn bishop_blocker_configurations() {
    assert_eq!(bishop_array_length(), TOTAL_BISHOP_ATTACKS);
}

#[test]
fn zobrist_hash() {
    let starting_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let mut starting_board = BoardState::from_fen(starting_fen).expect("FEN loading failed");

    let mut hash = ZobristHash::new();
    hash.hash_board(&starting_board);

    assert_eq!(starting_board.hash.get_u64(), hash.get_u64()); // Check that incremental hashing (used in the fen parser) matches full hash
    assert_eq!(starting_board.hash.get_u64(), 0x463b96181691fc9c);
    starting_board.make_move(&Move::simple_move(8, 16, PieceType::Pawn));
    assert_eq!(starting_board.hash.get_u64(), 0x8da7a73e5fdd72dc);
}

#[test]
fn threefold_repetition() {
    let fen = "2K5/8/2k5/8/8/8/8/4Q3 w - - 24 13";
    let mut board = BoardState::from_fen(fen).unwrap();
    let original_hash = board.hash.get_u64();

    let w_king_right = Move::simple_move(58, 59, PieceType::King);
    let w_king_left = Move::simple_move(59, 58, PieceType::King);

    let b_king_right = Move::simple_move(42, 43, PieceType::King);
    let b_king_left = Move::simple_move(43, 42, PieceType::King);

    assert!(!board.check_threefold_repetition());

    board.make_move(&w_king_right);
    board.make_move(&b_king_right);
    board.make_move(&w_king_left);
    board.make_move(&b_king_left);

    assert_eq!(board.hash.get_u64(), original_hash);
    assert!(!board.check_threefold_repetition());

    board.make_move(&w_king_right);
    board.make_move(&b_king_right);
    board.make_move(&w_king_left);
    board.make_move(&b_king_left);

    assert!(board.check_threefold_repetition());
}
