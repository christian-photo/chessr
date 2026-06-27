use chessr::generated::book::OpeningBook;
use std::io::BufRead;
use std::mem::size_of;
use vampirc_uci::UciMessage;
use vampirc_uci::parse_one;

use chessr::board::BoardState;
use chessr::engine::ChessrEngine;
use chessr::moves::generator::Move;
use chessr::uci;

macro_rules! show_size {
    (header) => {
        println!("{:<22} {:>4}    {}", "Type", "T", "Option<T>");
    };
    ($t:ty) => {
        println!(
            "{:<22} {:4} {:4}",
            stringify!($t),
            size_of::<$t>(),
            size_of::<Option<$t>>()
        )
    };
}

#[tokio::main]
async fn main() {
    let mut engine = ChessrEngine::new();

    for line in std::io::stdin().lock().lines() {
        let msg: UciMessage = parse_one(&line.unwrap());

        match msg {
            UciMessage::Uci => {
                uci::id(&ChessrEngine::name(), &ChessrEngine::author());
                engine.send_options();
                uci::acknowledge_uci();
            }
            UciMessage::SetOption { name, value } => engine.set_option(name, value),
            UciMessage::UciNewGame => engine.reset(),
            UciMessage::Position {
                startpos,
                fen,
                moves,
            } => {
                if startpos {
                    let board = BoardState::startpos();
                    engine.set_board(board);
                } else if let Some(uci_fen) = fen {
                    match BoardState::from_fen(&uci_fen.0) {
                        Ok(board) => engine.set_board(board),
                        Err(e) => eprintln!("Error while parsing fen: {}", e),
                    }
                }

                if let Some(board) = &mut engine.board {
                    moves.iter().for_each(|m| {
                        let move_desc = Move::from_uci_move(m, board)
                            .expect(&format!("UCI Move {} could not be parsed", m));
                        board.make_move(&move_desc);
                    });
                }
            }
            UciMessage::Go {
                time_control,
                search_control,
            } => engine.go(time_control, search_control),
            UciMessage::Stop => engine.stop(),
            UciMessage::IsReady => uci::engine_is_ready(),
            UciMessage::Quit => return,
            UciMessage::Unknown(cmd, _) => {
                if cmd.starts_with("go perft") {
                    let depth = cmd[9..].parse::<u8>().unwrap();
                    let count = engine.perft(depth, true);
                    println!("\nNodes searched: {}", count);
                }
            }
            _ => (),
        };
    }
}
