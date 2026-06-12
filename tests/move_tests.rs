use chessr::{
    board::Board,
    moves::{
        Move, MoveList,
        generator::MoveFlag,
        sliding::{SlidingAttackLookup, precompute_attacks},
    },
    piece::{Piece, PieceType},
};

#[test]
fn move_switches_player() {
    let mut board = Board::empty();
    board.add_piece(Piece::new(PieceType::Queen, true), 8);
    let move_desc = Move::simple_move(8, 16, PieceType::Queen);
    assert!(board.is_white_turn());
    board.make_move(&move_desc);
    assert!(!board.is_white_turn());
}

#[test]
fn move_moves_piece() {
    let mut board = Board::empty();
    board.add_piece(Piece::new(PieceType::Queen, true), 0);
    board.add_piece(Piece::new(PieceType::Pawn, false), 16);

    let capture = Move::new(
        0,
        16,
        MoveFlag::None,
        PieceType::Queen,
        Some(PieceType::Pawn),
        None,
    );
    board.make_move(&capture);

    println!("{}", board.to_ascii());
    assert_eq!(board.piece_at(16).unwrap().piece_type(), PieceType::Queen);
}

#[test]
fn pawn_moves() {
    let board = Board::from_fen("rnbqkbnr/pppp1ppp/8/3Pp3/8/P1r5/3P4/RNBQKBNR w - e6 0 13")
        .expect("FEN loading failed");
    let mut target_squares = vec![18u8, 19, 24, 27, 43, 44];

    let mut moves = MoveList::new();

    Move::pawn_moves(&board, &mut moves);
    for move_desc in moves.iter() {
        println!("{:?}", move_desc);
        let index = target_squares
            .iter()
            .position(|x| *x == move_desc.target_square)
            .unwrap();

        target_squares.remove(index);
    }

    assert!(
        target_squares.is_empty(),
        "Remaining target positions: {:#?}",
        target_squares
    );
}

#[test]
fn knight_moves() {
    let starting_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPP1PPPP/RNBQKBNR w KQkq - 0 1"; // Notice the removed white pawn
    let starting_board = Board::from_fen(starting_fen).expect("FEN loading failed");
    let mut target_squares = vec![11u8, 16, 18, 21, 23];

    let mut moves = MoveList::new();

    Move::knight_moves(&starting_board, &mut moves);
    for move_desc in moves.iter() {
        let index = target_squares
            .iter()
            .position(|x| *x == move_desc.target_square)
            .unwrap();
        target_squares.remove(index);
    }

    assert!(
        target_squares.is_empty(),
        "Remaining target positions: {:#?}",
        target_squares
    );

    let board2 = Board::from_fen("2K5/8/8/2k5/8/8/3N4/8 w - - 0 13").unwrap();
    let mut target_squares2 = vec![1u8, 5, 17, 21, 26, 28];

    moves.reset();

    Move::knight_moves(&board2, &mut moves);
    for move_desc in moves.iter() {
        let index = target_squares2
            .iter()
            .position(|x| *x == move_desc.target_square)
            .unwrap();
        target_squares2.remove(index);
    }

    assert!(
        target_squares2.is_empty(),
        "Remaining target positions: {:#?}",
        target_squares2
    );
}

#[test]
fn undo_promotion() {
    let mut board = Board::from_fen("8/3P4/1k6/8/2K5/8/8/8 w - - 0 1").unwrap();

    let promotion = Move::promotion(51, 59, PieceType::Queen, None);
    let copy = board.clone();
    board.make_move(&promotion);

    assert_eq!(
        board.piece_at(59).unwrap(),
        Piece::new(PieceType::Queen, true)
    );

    board.restore(copy);

    assert_eq!(
        board.piece_at(51).unwrap(),
        Piece::new(PieceType::Pawn, true)
    );
}

#[test]
fn algebraic_move_notation() {
    let lookup = precompute_attacks();
    let board = Board::from_fen("1RK3b1/RP3P2/P7/2k5/8/6N1/3N4/8 w - - 0 13").unwrap();

    let algebraic_moves = vec![
        "Nb1", "Nb3", "Nc4", "Nde4", "Nf3", "Ndf1", // Knight d-file
        "Nh1", "Ngf1", "Ne2", "Nge4", "Nf5", "Nh5",  // Knight g-file
        "Raa8", // Rook a-file
        "Rba8", // Rook b-file
        "Kd8", "Kd7", "Kc7", // King
        "f8=Q", "f8=R", "f8=B", "f8=N", "fxg8=Q", "fxg8=R", "fxg8=B", "fxg8=N", // Pawn f-file
    ];

    let moves = Move::generate_moves(&board, &lookup);
    for move_desc in moves.iter() {
        let algebraic = move_desc.to_algebraic(
            &board,
            &board.piece_at(move_desc.start_square).unwrap(),
            &lookup,
        );

        if !algebraic_moves.contains(&algebraic.as_str()) {
            panic!("Move {} was not found in premade table", algebraic);
        }
    }
}

#[test]
fn castle_legality() {
    let lookup = precompute_attacks();
    let illegal_1 =
        Board::from_fen("rn1qkb1r/p2p1ppp/b1p2n2/4p3/Pp2P3/1B3N2/1PPP1PPP/RNBQK2R w KQkq - 0 1")
            .unwrap(); // Castling kingside is illegal here for white, because the square next to the king is attacked by the black bishop

    let illegal_1_castle = Move::new(4, 6, MoveFlag::CastleKingside, PieceType::King, None, None);
    assert!(!illegal_1_castle.legal(&illegal_1, &lookup));

    let illegal_2 =
        Board::from_fen("rn1qk2r/pb1p1ppp/2p2n2/2b1p3/Pp2P3/1B3N2/1PPP2PP/RNBQK2R w KQkq - 0 1")
            .unwrap(); // Same situation as before, just the other bishop

    let illegal_2_castle = Move::new(4, 6, MoveFlag::CastleKingside, PieceType::King, None, None);
    assert!(!illegal_2_castle.legal(&illegal_2, &lookup));

    let illegal_3 =
        Board::from_fen("rn1qk2r/pb1p2pp/2p2n2/4p2B/Pp2P2b/5NP1/1PPP3P/RNBQK2R b KQkq - 0 1")
            .unwrap(); // Now it's blacks turn but the king is in check by the white bishop

    let illegal_3_castle = Move::new(
        60,
        62,
        MoveFlag::CastleKingside,
        PieceType::King,
        None,
        None,
    );
    assert!(!illegal_3_castle.legal(&illegal_3, &lookup));

    let legal =
        Board::from_fen("r3k2r/pbqp3p/n1p2np1/4p2B/Pp2P2b/5NP1/1PPP3P/RNBQK2R b KQkq - 0 1")
            .unwrap();

    let legal1 = Move::new(
        60,
        58,
        MoveFlag::CastleQueenside,
        PieceType::King,
        None,
        None,
    );
    let legal2 = Move::new(
        60,
        62,
        MoveFlag::CastleKingside,
        PieceType::King,
        None,
        None,
    );

    assert!(legal1.legal(&legal, &lookup));
    assert!(legal2.legal(&legal, &lookup));
}

#[test]
fn en_passant_pinned() {
    let lookup = precompute_attacks();
    let illegal = Board::from_fen("3k4/8/8/1KPp2r1/8/8/8/8 w - - 0 1").unwrap(); // This is illegal, because when white captures with en passant, the rook as a line of attack on the king
    let en_passant = Move::new(
        34,
        43,
        MoveFlag::EnPassant,
        PieceType::Pawn,
        Some(PieceType::Pawn),
        None,
    );

    assert!(!en_passant.legal(&illegal, &lookup));
}

#[test]
fn illegal_capture() {
    let lookup = precompute_attacks();
    let mut board = Board::from_fen("8/8/8/5p2/8/8/5QBq/1K1R2bk b - - 0 1").expect("Fen is valid");

    let king_capture_bishop = Move::new(
        7,
        14,
        MoveFlag::None,
        PieceType::King,
        Some(PieceType::Bishop),
        None,
    );

    assert!(!king_capture_bishop.legal(&board, &lookup));
}

#[test]
fn legal_move_generation() {
    fn count_legal_moves(depth: u8, board: &Board, lookup: &SlidingAttackLookup) -> u32 {
        if depth == 0 {
            return 1;
        }
        let mut counter = 0u32;

        let moves = Move::generate_moves(board, lookup);
        for m in moves.iter() {
            if m.legal(board, lookup) {
                let mut new_board = board.clone();
                new_board.make_move(m);

                counter += count_legal_moves(depth - 1, &new_board, lookup);
            }
        }

        return counter;
    }
    let lookup = precompute_attacks();
    let board = Board::from_fen("8/1B6/8/5p2/8/8/5Qrq/1K1R2bk w - - 0 1").expect("Fen is valid");

    assert_eq!(count_legal_moves(1, &board, &lookup), 43);
    assert_eq!(count_legal_moves(2, &board, &lookup), 517);
    assert_eq!(count_legal_moves(3, &board, &lookup), 19513);
    assert_eq!(count_legal_moves(4, &board, &lookup), 381753);
    assert_eq!(count_legal_moves(5, &board, &lookup), 12839499);
}
