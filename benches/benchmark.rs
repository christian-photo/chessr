use chessr::{
    board::Board,
    moves::{Move, MoveList, sliding::precompute_attacks},
    piece::PieceType,
};
use criterion::{Criterion, criterion_group, criterion_main};

fn move_generation_benchmark(c: &mut Criterion) {
    let mut board =
        Board::from_fen("rnbqkbnr/pp2pppp/2p5/3p4/2PP4/8/PP2PPPP/RNBQKBNR w KQkq - 0 1")
            .expect("Fen is valid");
    let lookup = precompute_attacks();

    // Add back in when all movement generators are implemented
    // c.bench_function("Move generation", |b| {
    //     b.iter(|| Move::generate_legal_moves(&mut board))
    // });
    //

    c.bench_function("Knight move generation", |b| {
        b.iter(|| Move::knight_moves(&board, &mut MoveList::new()))
    });

    c.bench_function("Pawn move generation", |b| {
        b.iter(|| Move::pawn_moves(&board, &mut MoveList::new()))
    });

    c.bench_function("Bishop move generation", |b| {
        b.iter(|| Move::bishop_moves(&board, &mut MoveList::new(), &lookup))
    });

    c.bench_function("Rook move generation", |b| {
        b.iter(|| Move::rook_moves(&board, &mut MoveList::new(), &lookup))
    });

    c.bench_function("Queen move generation", |b| {
        b.iter(|| Move::queen_moves(&board, &mut MoveList::new(), &lookup))
    });
}

fn move_making_benchmark(c: &mut Criterion) {
    let mut board = Board::from_fen("8/3P4/1k6/8/2K5/8/8/8 w - - 0 1").unwrap();

    let promotion = Move {
        capture: None,
        flag: None,
        promotion: Some(PieceType::Queen),
        start_square: 51,
        target_square: 59,
    };

    c.bench_function("Do/Undo move", |b| {
        b.iter(|| {
            let previous = board.clone();
            board.make_move(&promotion);

            board.restore(previous);
        })
    });
}

fn engine_startup_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("engine startup");
    // Configure Criterion.rs to detect smaller differences and increase sample size to improve
    // precision and counteract the resulting noise.
    group.sample_size(50);
    group.bench_function("Sliding attack lookup generation", |b| {
        b.iter(|| precompute_attacks())
    });
    group.finish();
}

criterion_group!(
    benches,
    move_generation_benchmark,
    move_making_benchmark,
    engine_startup_benchmark
);
criterion_main!(benches);
