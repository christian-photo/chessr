use chessr::{board::Board, move_generator::Move};
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

fn criterion_benchmark(c: &mut Criterion) {
    let mut board =
        Board::from_fen("rnbqkbnr/pp2pppp/2p5/3p4/2PP4/8/PP2PPPP/RNBQKBNR w KQkq - 0 1")
            .expect("Fen is valid");

    // Add back in when all movement generators are implemented
    // c.bench_function("Move generation", |b| {
    //     b.iter(|| Move::generate_legal_moves(&mut board))
    // });

    c.bench_function("Knight move generation", |b| {
        b.iter(|| Move::knight_moves(&mut board))
    });

    c.bench_function("Pawn move generation", |b| {
        b.iter(|| Move::pawn_moves(&mut board))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
