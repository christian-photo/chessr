use std::mem::size_of;

use chessr::board::Board;
use chessr::moves::MoveList;
use chessr::moves::generator::Move;
use chessr::piece::Piece;

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
}
