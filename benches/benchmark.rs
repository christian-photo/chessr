use chessr::{
    board::Board,
    move_generator::{Move, MoveList},
    piece::PieceType,
};
use criterion::{Criterion, criterion_group, criterion_main};

fn move_generation_benchmark(c: &mut Criterion) {
    let mut board =
        Board::from_fen("rnbqkbnr/pp2pppp/2p5/3p4/2PP4/8/PP2PPPP/RNBQKBNR w KQkq - 0 1")
            .expect("Fen is valid");

    // Add back in when all movement generators are implemented
    // c.bench_function("Move generation", |b| {
    //     b.iter(|| Move::generate_legal_moves(&mut board))
    // });
    //

    c.bench_function("Knight move generation", |b| {
        b.iter(|| Move::knight_moves(&mut board, &mut MoveList::new()))
    });

    c.bench_function("Pawn move generation", |b| {
        b.iter(|| Move::pawn_moves(&mut board, &mut MoveList::new()))
    });
}

fn move_making_benchmark(c: &mut Criterion) {
    let mut board = Board::from_fen("8/3P4/1k6/8/2K5/8/8/8 w - - 0 1").unwrap();

    let promotion = Move {
        capture: None,
        en_passant: false,
        is_castle: false,
        promotion: Some(PieceType::Queen),
        start_square: 51,
        target_square: 59,
    };

    c.bench_function("Do/Undo move", |b| {
        b.iter(|| {
            board.make_move(&promotion);

            board.undo_move(&promotion);
        })
    });
}

criterion_group!(benches, move_generation_benchmark, move_making_benchmark);
criterion_main!(benches);
