use chessr::{
    board::BoardState,
    engine::ChessrEngine,
    moves::{Move, MoveList, sliding::precompute_attacks},
    piece::PieceType,
};
use criterion::{Criterion, criterion_group, criterion_main};

fn perft_benchmark(c: &mut Criterion) {
    let mut engine = ChessrEngine::new();
    let state = BoardState::from_fen(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P1P1/2N2Q1p/PPPBBP1P/R3K2R b KQkq - 0 1",
    )
    .unwrap();

    engine.set_board(state);

    c.bench_function("perft(3)", |b| b.iter(|| engine.perft(3, false)));

    let state =
        BoardState::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8").unwrap();

    engine.set_board(state);

    c.bench_function("perft(3) Nr 2", |b| b.iter(|| engine.perft(3, false)));
}

fn legality_check_benchmark(c: &mut Criterion) {
    let lookup = precompute_attacks();
    let state = BoardState::from_fen(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P1P1/2N2Q1p/PPPBBP1P/R3K2R b KQkq - 0 1",
    )
    .unwrap();

    let mut list = MoveList::new();
    Move::generate_moves(&mut list, &state, &lookup);

    c.bench_function("Is Attacked", |b| {
        b.iter(|| {
            BoardState::is_attacked(
                0b11101101 << 32,
                state.bitboards[12].get_u64() | state.bitboards[13].get_u64(),
                &state.bitboards,
                true,
                &lookup,
            )
        })
    });

    c.bench_function("Legality check", |b| {
        let checked = BoardState::is_attacked(
            state.bitboards[11].get_u64(),
            state.bitboards[12].get_u64() | state.bitboards[13].get_u64(),
            &state.bitboards,
            !state.is_white_turn(),
            &lookup,
        );
        b.iter(|| {
            list.iter().into_iter().for_each(|m| {
                m.legal(&state, &lookup, checked);
            });
        })
    });
}

fn move_generation_benchmark(c: &mut Criterion) {
    let lookup = precompute_attacks();
    let board =
        BoardState::from_fen("rnbqkbnr/pp2pppp/2p5/3p4/2PP4/8/PP2PPPP/RNBQKBNR w KQkq - 0 1")
            .expect("Fen is valid");

    c.bench_function("Move generation", |b| {
        let mut list = MoveList::new();
        b.iter(|| {
            Move::generate_moves(&mut list, &board, &lookup);
            list.reset();
        })
    });

    c.bench_function("Knight move generation", |b| {
        let mut list = MoveList::new();
        b.iter(|| {
            Move::knight_moves(&board, &mut list);
            list.reset();
        })
    });

    c.bench_function("Pawn move generation", |b| {
        let mut list = MoveList::new();
        b.iter(|| {
            Move::pawn_moves(&board, &mut list);
            list.reset();
        })
    });

    c.bench_function("Bishop move generation", |b| {
        let mut list = MoveList::new();
        b.iter(|| {
            Move::bishop_moves(&board, &mut list, &lookup);
            list.reset();
        })
    });

    c.bench_function("Rook move generation", |b| {
        let mut list = MoveList::new();
        b.iter(|| {
            Move::rook_moves(&board, &mut list, &lookup);
            list.reset();
        })
    });

    c.bench_function("Queen move generation", |b| {
        let mut list = MoveList::new();
        b.iter(|| {
            Move::queen_moves(&board, &mut list, &lookup);
            list.reset();
        })
    });

    c.bench_function("King move generation", |b| {
        let mut list = MoveList::new();
        b.iter(|| {
            Move::king_moves(&board, &mut list);
            list.reset();
        })
    });
}

fn move_making_benchmark(c: &mut Criterion) {
    let mut board = BoardState::from_fen("8/3P4/1k6/8/2K5/8/8/8 w - - 0 1").unwrap();

    let promotion = Move::promotion(51, 59, PieceType::Queen, None);

    c.bench_function("Do/Undo move", |b| {
        let previous = board.clone();
        b.iter(|| {
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
    perft_benchmark,
    legality_check_benchmark,
    move_generation_benchmark,
    move_making_benchmark,
    engine_startup_benchmark
);
criterion_main!(benches);
