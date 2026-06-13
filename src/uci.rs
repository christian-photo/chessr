pub fn id(name: &str, author: &str) {
    println!("id name {}", name);
    println!("id author {}", author);
}

pub fn acknowledge_uci() {
    println!("uciok");
}

pub fn engine_is_ready() {
    println!("readyok");
}

/// Send the best move according to the engine, in algebraic notation
pub fn best_move(best_move: &str) {
    println!("bestmove {}", best_move);
}

pub enum UciInfo {
    Depth,
    Time,
    Nodes,
    Pv,
    MultiPv,
    Score,
    Hashfull,
    Nps,
    Refutation,
    CurrentLine,
}
