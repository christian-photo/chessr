use std::mem::size_of;

use chessr::piece::Piece;
use chessr::pregen::generate_knight_attack_map;

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
    println!("{:#?}", generate_knight_attack_map());
    show_size!(header);
    show_size!(i32);
    show_size!(u8);
    show_size!(Piece);
    show_size!(&i32);
    show_size!(Box<i32>);
    show_size!(&[i32]);
    show_size!(Vec<i32>);
    show_size!(Result<(), Box<i32>>);
}
