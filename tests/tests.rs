use chessr::{
    board::{Bitboard, Board},
    move_generator::Move,
    piece::{Piece, PieceType},
};

#[test]
fn move_switches_player() {
    let mut board = Board::empty();
    board.add_piece(Piece::new(PieceType::Queen, true), 8);
    let move_desc = Move::simple(8, 16);
    assert!(board.is_white_turn());
    board.make_move(&move_desc);
    assert!(!board.is_white_turn());
}

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
    assert_eq!(p1.repr(), 0b00000001);
    assert!(p1.is_white());
    assert_eq!(p1.to_bitboard_index(), 0);

    let p2 = Piece::new(PieceType::Knight, false);
    assert_eq!(p2.repr(), 0b10000010);
    assert!(!p2.is_white());
    assert_eq!(p2.to_bitboard_index(), 7);

    let p3 = Piece::new(PieceType::Bishop, false);
    assert_eq!(p3.repr(), 0b10000100);
    assert_eq!(p3.to_bitboard_index(), 8);
}

#[test]
fn fen_reader() {
    let starting_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let starting_board = Board::from_fen(starting_fen).expect("FEN loading failed");
    assert!(starting_board.is_white_turn());

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
    let midgame_board = Board::from_fen(midgame_fen).expect("FEN loading failed");

    assert!(!midgame_board.is_white_turn());
    let bishop_mid = midgame_board
        .piece_at(58)
        .expect("Bishop must be present in top row");

    assert_eq!(bishop_mid.piece_type(), PieceType::Bishop);
    println!("{}", midgame_board.to_ascii());
}

#[test]
fn algebraic_to_pos() {
    assert_eq!(Board::algebraic_to_u8("h8"), 63);
    assert_eq!(Board::algebraic_to_u8("a1"), 0);
    assert_eq!(Board::algebraic_to_u8("b2"), 9);
}

#[test]
fn move_moves_piece() {
    let mut board = Board::empty();
    board.add_piece(Piece::new(PieceType::Queen, true), 0);
    board.add_piece(Piece::new(PieceType::Pawn, false), 16);

    let capture = Move::capture(0, 16, PieceType::Queen, None);
    board.make_move(&capture);

    println!("{}", board.to_ascii());
    assert_eq!(board.piece_at(16).unwrap().piece_type(), PieceType::Queen);
}

fn pos_to_algebraic(pos: u8) -> String {
    let rank = pos / 8;
    let file = pos % 8;

    format!("{}{}", (('a' as u8) + rank) as char, file + 1)
}

#[test]
fn pos_conversion() {
    assert_eq!(pos_to_algebraic(0), "a1");
    assert_eq!(pos_to_algebraic(63), "h8");
    assert_eq!(pos_to_algebraic(9), "b2");
}
