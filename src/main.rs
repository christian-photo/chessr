use std::io::stdin;
use std::mem::size_of;

use chessr::board::Board;
use chessr::engine::ChessrEngine;
use chessr::moves::MoveList;
use chessr::moves::generator::Move;
use chessr::piece::Piece;
use chessr::pregen::{generate_bishop_attack_mask, generate_rook_attack_mask};
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

fn main() {
    println!("{:#?}", generate_bishop_attack_mask());
    println!("{:#?}", generate_rook_attack_mask());
    show_size!(header);
    show_size!(i32);
    show_size!(u8);
    show_size!(Piece);
    show_size!(Board);
    show_size!(Move);
    show_size!(MoveList);
    show_size!(&i32);
    show_size!(Box<i32>);
    show_size!(&[i32]);
    show_size!(Vec<i32>);
    show_size!(Result<(), Box<i32>>);

    let mut engine = ChessrEngine::new();

    loop {
        let mut input = String::new();
        stdin().read_line(&mut input).expect("Could not read input");
        let split: Vec<&str> = input.trim().split(' ').collect();

        match split[0] {
            "uci" => {
                uci::id(&ChessrEngine::name(), &ChessrEngine::author());
                // UciSender::options();
                uci::acknowledge_uci();
            }
            "setoption" => (),
            "ucinewgame" => (),
            "position" => match split[1] {
                "startpos" => {
                    let board = Board::startpos();
                    engine.set_board(board);
                    if split.len() > 2 && split[2] == "moves" {
                        let moves: Vec<Move> = split[3..]
                            .iter()
                            .map(|s| {
                                Move::from_uci_move(s, &board)
                                    .expect(&format!("UCI Move {} could not be parsed", s)) // This could be improved
                            })
                            .collect();

                        engine.make_moves(&moves);
                    }
                }
                "fen" => match Board::from_fen(&split[2..6].join(" ")) {
                    Ok(board) => {
                        engine.set_board(board);
                        if split.len() > 6 && split[6] == "moves" {
                            let moves: Vec<Move> = split[7..]
                                .iter()
                                .map(|s| {
                                    Move::from_uci_move(s, &board)
                                        .expect(&format!("UCI Move {} could not be parsed", s)) // This could be improved
                                })
                                .collect();

                            engine.make_moves(&moves);
                        }
                    }
                    Err(e) => println!("Error: {}", e),
                },
                _ => (),
            },
            "go" => engine.go(),
            "stop" => engine.stop(),
            "isready" => uci::engine_is_ready(),
            "quit" => return,
            _ => (),
        }
    }
}
